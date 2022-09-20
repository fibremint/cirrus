use std::{
    sync::{
        Arc, 
        RwLock, 
        atomic::{AtomicUsize, Ordering}
    }, 
    collections::VecDeque
};
use rubato::Resampler;

use crate::{dto::AudioSource, request};

use super::state::AudioSampleStatus;

pub struct AudioSample {
    pub source: AudioSource,
    sample_buffer: Arc<RwLock<Vec<VecDeque<f32>>>>,
    current_sample_frame: Arc<AtomicUsize>,
    pub buffer_status: Arc<AtomicUsize>,
    pub last_buf_req_pos: Arc<AtomicUsize>,
    resampler: Arc<RwLock<rubato::FftFixedInOut<f32>>>,
    pub resampler_frames_input_next: usize,
    pub resampler_frames_output_next: usize,
    remain_sample_raw: Arc<RwLock<Vec<u8>>>,
    // resampled_sample_frames: usize,
    host_sample_rate: u32,
    host_output_channels: usize,
    pub content_length: f32,
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
            sample_buffer: Arc::new(RwLock::new(sample_buffer)),
            current_sample_frame: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleStatus::FillBuffer as usize)),
            last_buf_req_pos: Arc::new(AtomicUsize::new(0)),
            resampler: Arc::new(
                RwLock::new(
                    resampler
                )
            ),
            resampler_frames_input_next,
            resampler_frames_output_next,
            remain_sample_raw: Arc::new(RwLock::new(Vec::new())),
            // resampled_sample_frames,
            host_sample_rate,
            host_output_channels,
            content_length,
        }
    }

    pub fn get_remain_sample_buffer_len(&self) -> usize {
        self.sample_buffer.read().unwrap()[0].len()
    }

    pub fn get_current_sample_idx(&self) -> usize {
        self.current_sample_frame.load(Ordering::SeqCst)
    }

    pub fn get_sec_from_sample_len(&self, sample_len: usize) -> f32 {
        sample_len as f32 / self.host_sample_rate as f32
    }

    pub fn get_sample_idx_from_sec(&self, sec: f32) -> usize {
        (sec * self.host_sample_rate as f32).floor() as usize
    }

    pub fn get_current_playback_position_sec(&self) -> f32 {
        self.get_sec_from_sample_len(self.get_current_sample_idx())
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f32 {
        self.get_sec_from_sample_len(self.get_remain_sample_buffer_len())   
    }

    pub fn set_current_sample_frame_idx(&self, sample_frame_idx: usize) {
        self.current_sample_frame.store(sample_frame_idx, Ordering::SeqCst);
    }

    pub fn drain_sample_buffer(&self, drain_len: usize) {
        let mut sample_buffer = self.sample_buffer.write().unwrap();

        let min_drain_buffer_len = std::cmp::min(sample_buffer[0].len(), drain_len);

        for ch_sample_buffer in sample_buffer.iter_mut() {
            ch_sample_buffer.drain(..min_drain_buffer_len);
        }
    }

    pub async fn fetch_buffer(
        &self, 
        buffer_margin: f32,
        fetch_buffer_sec: f32,
    ) -> Result<(), anyhow::Error> {
        if AudioSampleStatus::StopFillBuffer == 
            AudioSampleStatus::from(self.buffer_status.load(Ordering::Relaxed)) {
                self.buffer_status.store(AudioSampleStatus::FillBuffer as usize, Ordering::Relaxed);
            }

        // while self.get_remain_sample_buffer_sec() < buffer_margin && 
        //     AudioSampleStatus::FillBuffer == AudioSampleStatus::from(self.buffer_status.load(Ordering::Relaxed)) {

        //     self.get_buffer_for(fetch_buffer_sec as u32 * 1000).await?;
        // }

        if self.get_remain_sample_buffer_sec() < buffer_margin {
            println!("fetch audio sample buffer");
            self.get_buffer_for(fetch_buffer_sec as u32 * 1000).await?;
        }

        Ok(())
    }

    pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
        let req_samples = ms * self.source.metadata.sample_rate / 1000;
        
        println!("request audio data part");
        let last_buf_req_pos = self.last_buf_req_pos.load(Ordering::SeqCst);

        let sample_res = request::get_audio_data(
            &self.source.id,
            std::cmp::min(last_buf_req_pos as u32, self.source.metadata.sample_frames as u32),
            std::cmp::min(last_buf_req_pos as u32 + req_samples, self.source.metadata.sample_frames as u32)
        ).await?;

        println!("parse audio data response as audio sample");
        // ref: https://users.rust-lang.org/t/convert-slice-u8-to-u8-4/63836
        let sample_res = sample_res.into_inner();
        let chunks_per_channel = 2;
        let channel = 2;

        let mut remain_sample_raw = self.remain_sample_raw.write().unwrap();
        
        let mut p_samples: Vec<u8> = remain_sample_raw.drain(..).collect();
        p_samples.extend_from_slice(&sample_res.content);
        
        let mut chunks_items_iter = p_samples.chunks_exact(chunks_per_channel * channel * self.resampler_frames_input_next);
        let mut channel_sample_buf_extend_cnt: usize = 0;

        while let Some(chunk_items) = chunks_items_iter.next() {
            let mut input_buf: Vec<Vec<f32>> = Vec::with_capacity(channel);

            for _ in 0..channel {
                input_buf.push(Vec::with_capacity(self.resampler_frames_input_next));
            }

            let sample_items = chunk_items
                .chunks(2)
                .map(|item| i16::from_be_bytes(item.try_into().unwrap()) as f32 / self.host_sample_rate as f32)
                .collect::<Vec<f32>>();

            for sample_item in sample_items.chunks(2) {
                for channel_idx in 0..channel {
                    input_buf[channel_idx].push(sample_item[channel_idx]);
                }
            }

            let mut resampler = self.resampler.write().unwrap();
            let resampled_wave = resampler.process(input_buf.as_ref(), None).unwrap();

            if  AudioSampleStatus::StopFillBuffer == AudioSampleStatus::from(self.buffer_status.load(Ordering::Relaxed)) {
                println!("stop fill buffer");

                self.last_buf_req_pos.store(last_buf_req_pos + channel_sample_buf_extend_cnt, Ordering::SeqCst);

                return Ok(());
            }

            let mut sample_buffer = self.sample_buffer.write().unwrap();
            for (ch_idx, channel_sample_buffer) in sample_buffer.iter_mut().enumerate() {
                channel_sample_buffer.extend(resampled_wave.get(ch_idx).unwrap());
            }

            channel_sample_buf_extend_cnt += resampled_wave[0].len();
        }

        let remain_samples = chunks_items_iter.remainder();
        remain_sample_raw.extend(remain_samples);

        println!("done resampling wave data");
        self.last_buf_req_pos.store(last_buf_req_pos + req_samples as usize, Ordering::SeqCst);

        Ok(())
    }
    
    pub fn play_for(&self, output: &mut [f32]) {
        // let mut sample_buffer = self.sample_buffer.write().unwrap();

        for output_channel_frame in output.chunks_mut(self.host_output_channels) {  
            let mut channel_sample_read: u8 = 0;
            let mut sample_buffer = self.sample_buffer.write().unwrap();
          
            for channel_idx in 0..self.host_output_channels {
                if let Some(channel_sample) = sample_buffer[channel_idx].pop_front() {
                    output_channel_frame[channel_idx] = channel_sample;
                    channel_sample_read += 1;
                } else {
                    output_channel_frame[channel_idx] = 0.0;
                    // break;
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