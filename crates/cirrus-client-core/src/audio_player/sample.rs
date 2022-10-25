use std::{
    sync::{
        Arc, 
        RwLock, 
        atomic::{AtomicUsize, Ordering}, Mutex
    }, 
    collections::VecDeque
};


use std::iter::Iterator;

use anyhow::anyhow;
use futures::StreamExt;

use rubato::Resampler;

use crate::{dto::AudioSource, request};

use super::state::AudioSampleStatus;

pub struct AudioSample {
    pub source: AudioSource,
    sample_buffer: RwLock<Vec<VecDeque<f32>>>,
    current_sample_frame: Arc<AtomicUsize>,
    pub buffer_status: Arc<AtomicUsize>,
    resampler: Mutex<rubato::FftFixedInOut<f32>>,
    pub resampler_frames_input_next: usize,
    pub resampler_frames_output_next: usize,
    remain_sample_raw: Mutex<Vec<u8>>,
    host_sample_rate: u32,
    host_output_channels: usize,
    pub content_length: f32,
    last_buf_req_pos: Arc<AtomicUsize>,
}

impl AudioSample {
    pub fn new(source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {
        let resampler = rubato::FftFixedInOut::new(
            source.metadata.sample_rate as usize, 
            host_sample_rate as usize, 
            1024, 
            2
        ).unwrap();

        let resampler_frames_input_next = resampler.input_frames_next();
        let resampler_frames_output_next = resampler.output_frames_next();

        let mut sample_buffer: Vec<VecDeque<f32>> = Vec::with_capacity(2);
        for _ in 0..host_output_channels {
            sample_buffer.push(VecDeque::new());
        }

        let resampled_sample_frames = (source.metadata.sample_frames as f32 * (host_sample_rate as f32 / source.metadata.sample_rate as f32)).ceil() as usize;
        let content_length = resampled_sample_frames as f32 / host_sample_rate as f32;

        Self {
            source,
            sample_buffer: RwLock::new(sample_buffer),
            current_sample_frame: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleStatus::DoneFillBuffer as usize)),
            resampler: Mutex::new(resampler),
            resampler_frames_input_next,
            resampler_frames_output_next,
            remain_sample_raw: Mutex::new(Vec::new()),
            host_sample_rate,
            host_output_channels,
            content_length,
            last_buf_req_pos: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn get_remain_sample_buffer_len(&self) -> usize {
        self.sample_buffer.read().unwrap()[0].len()
    }

    pub fn get_playback_sample_idx(&self) -> usize {
        self.current_sample_frame.load(Ordering::SeqCst)
    }

    pub fn get_sec_from_sample_len(&self, sample_len: usize) -> f32 {
        sample_len as f32 / self.host_sample_rate as f32
    }

    pub fn get_playback_sample_idx_from_sec(&self, sec: f32) -> usize {
        (sec * self.host_sample_rate as f32).floor() as usize
    }

    pub fn get_current_playback_position_sec(&self) -> f32 {
        self.get_sec_from_sample_len(self.get_playback_sample_idx())
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f32 {
        self.get_sec_from_sample_len(self.get_remain_sample_buffer_len())   
    }

    fn get_audio_source_sample_idx(&self, sec: f32) -> usize {
        ((sec * self.source.metadata.sample_rate as f32) / self.resampler_frames_input_next as f32).floor() as usize
    }

    pub fn set_current_sample_frame_idx(&self, sample_frame_idx: usize) {
        self.current_sample_frame.store(sample_frame_idx, Ordering::SeqCst);
    }

    // fn get_last_buf_req_pos(&self) -> usize {
    //     self.last_buf_req_pos.load(Ordering::SeqCst)
    // }

    // fn set_last_buf_req_pos(&mut self, pos: usize) {
    //     self.last_buf_req_pos.store(
    //         pos, 
    //         Ordering::SeqCst
    //     );
    // }

    pub fn set_playback_position(&self, position_sec: f32) {
        self.buffer_status.store(AudioSampleStatus::DoneFillBuffer as usize, Ordering::Relaxed);

        let position_sample_idx = self.get_playback_sample_idx_from_sec(position_sec);

        let position_sec_delta = position_sec - self.get_current_playback_position_sec();
        let drain_buffer_len = {
            if position_sec_delta > 0.0 {
                position_sample_idx - self.get_playback_sample_idx()
                // position_sec_delta * self.host_sample_rate as f32 
            } else {
                self.get_remain_sample_buffer_len()
            }
        };
        self.drain_sample_buffer(drain_buffer_len);
        
        let buf_req_start_pos = self.get_audio_source_sample_idx(position_sec);
        // self.set_last_buf_req_pos(buf_req_start_pos);
        self.last_buf_req_pos.store(
            buf_req_start_pos, 
            Ordering::SeqCst
        );

        // let position_sample_idx = self.get_playback_sample_idx_from_sec(position_sec);
        self.set_current_sample_frame_idx(position_sample_idx);

    }

    pub fn drain_sample_buffer(&self, drain_len: usize) {
        let mut sample_buffer = self.sample_buffer.write().unwrap();

        let min_drain_buffer_len = std::cmp::min(sample_buffer[0].len(), drain_len);
        let drain_delta = sample_buffer[0].len() as i32 - drain_len as i32;

        for ch_sample_buffer in sample_buffer.iter_mut() {
            ch_sample_buffer.drain(..min_drain_buffer_len);
        }

        if drain_delta < 0 {
            let mut remain_sample_raw = self.remain_sample_raw.lock().unwrap();
            remain_sample_raw.drain(..);
            // let drain_remain_sample_raw_len = std::cmp::min(remain_sample_raw.len(), (drain_delta.abs() * 2 * 2) as usize);

            // remain_sample_raw.drain(..drain_remain_sample_raw_len);
        }
    }

    pub async fn fetch_buffer(
        &self,
        playback_buffer_margin_sec: f32,
    ) -> Result<(), anyhow::Error> {
        self.buffer_status.store(AudioSampleStatus::StartFillBuffer as usize, Ordering::SeqCst);

        let fetch_buffer_sec = playback_buffer_margin_sec - self.get_remain_sample_buffer_sec();
        if let Err(_) = self.get_buffer_for(fetch_buffer_sec).await {
            // println!("batch buffer error: {:?}", e);
        }

        self.buffer_status.store(AudioSampleStatus::DoneFillBuffer as usize, Ordering::SeqCst);
        
        Ok(())
    }

    pub async fn get_buffer_for(&self, sec: f32) -> Result<(), anyhow::Error> {
        let buf_req_start_idx = self.last_buf_req_pos.load(Ordering::SeqCst) as u32;

        let mut audio_data_stream = request::get_audio_data_stream(
            &self.source.id,
            self.resampler_frames_input_next as u32,
            buf_req_start_idx,
            buf_req_start_idx + self.get_audio_source_sample_idx(sec) as u32
        ).await?;

        // ref: https://users.rust-lang.org/t/convert-slice-u8-to-u8-4/63836
        let channels = 2;

        while let Some(data) = audio_data_stream.next().await {
            let mut sample_items = Vec::with_capacity(channels);
            for ch_data in data.unwrap().audio_channel_data.into_iter() {
                sample_items.push(ch_data.content);
            }

            let mut resampler = self.resampler.lock().unwrap();
            let mut resampler_output_buffer = resampler.output_buffer_allocate();

            // zero pad for final sample data
            if sample_items[0].len() < self.resampler_frames_input_next {
                let zero_pad_len = self.resampler_frames_input_next - sample_items[0].len();
                for sample_items_ch in sample_items.iter_mut() {
                    sample_items_ch.extend_from_slice(&vec![0.; zero_pad_len]);
                }
            }

            resampler.process_into_buffer(&sample_items, &mut resampler_output_buffer, None).unwrap();

            if  AudioSampleStatus::DoneFillBuffer == AudioSampleStatus::from(self.buffer_status.load(Ordering::Relaxed)) {
                println!("stop fill buffer");
                
                return Err(anyhow!("fill buffer interrupted"));
            }

            let mut sample_buffer = self.sample_buffer.write().unwrap();
            for (ch_idx, channel_sample_buffer) in sample_buffer.iter_mut().enumerate() {
                // channel_sample_buffer.extend(resampled_wave.get(ch_idx).unwrap());
                channel_sample_buffer.extend(resampler_output_buffer.get(ch_idx).unwrap())
            }

            let buf_req_start_idx = self.last_buf_req_pos.load(Ordering::SeqCst);
            self.last_buf_req_pos.store(
                buf_req_start_idx + 1, 
                Ordering::SeqCst
            );

            // let buf_req_start_idx = self.get_last_buf_req_pos();
            // self.set_last_buf_req_pos(buf_req_start_idx+1);
        }

        Ok(())
    }
    
    pub fn play_for(&self, output: &mut [f32]) {
        for output_channel_frame in output.chunks_mut(self.host_output_channels) {  
            let mut channel_sample_read: u8 = 0;
            let mut sample_buffer = self.sample_buffer.write().unwrap();
          
            for channel_idx in 0..self.host_output_channels {
                if let Some(channel_sample) = sample_buffer[channel_idx].pop_front() {
                    output_channel_frame[channel_idx] = channel_sample;
                    channel_sample_read += 1;
                } else {
                    output_channel_frame[channel_idx] = 0.0;
                }
            }

            drop(sample_buffer);

            let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);
            self.current_sample_frame.store(
                current_sample_frame + (channel_sample_read / self.host_output_channels as u8) as usize, 
                Ordering::SeqCst
            );
        }
    }
}