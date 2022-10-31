use std::{
    sync::{
        Arc, 
        RwLock, 
        atomic::{AtomicUsize, Ordering, AtomicBool}, Condvar, Mutex
    }, 
    collections::VecDeque
};
use std::iter::Iterator;

// use anyhow::anyhow;
use futures::StreamExt;
use tokio::sync::mpsc;
use opus;
use audio::{io, wrap, WriteBuf, ExactSizeBuf, ChannelMut, Channels, Channel, ReadBuf};


use crate::{dto::AudioSource, request};
use super::state::AudioSampleBufferStatus;

pub struct AudioSample {
    pub inner: Arc<AudioSampleInner>,
    pub tx: mpsc::Sender<f32>,
    thread_run_states: Vec<Arc<AtomicBool>>,
}

impl AudioSample {
    pub fn new(audio_source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {
        let (tx, mut rx) = mpsc::channel(64);

        let inner = Arc::new(
            AudioSampleInner::new(
                audio_source,
                host_sample_rate, 
                host_output_channels
            )
        );

        let inner_1_clone = inner.clone();

        let mut thread_run_states: Vec<Arc<AtomicBool>> = Vec::new();
        let thread_run_state_1 = Arc::new(AtomicBool::new(true));
        let thread_run_state_1_clone = thread_run_state_1.clone();

        tokio::spawn(async move {
            loop {
                if !thread_run_state_1_clone.load(Ordering::Relaxed) {
                    println!("stop thread: fetch buffer, source id: {}", inner_1_clone.source.id);

                    break;
                }

                while let Some(fetch_buf_sec) = rx.recv().await {
                    inner_1_clone.fetch_buffer(&inner_1_clone.source.server_address, fetch_buf_sec).await.unwrap();
                }
            }
        });

        thread_run_states.push(thread_run_state_1);

        Self {
            inner,
            tx,
            thread_run_states
        }
    }

    pub fn get_current_playback_position_sec(&self) -> f32 {
        self.inner.get_current_playback_position_sec()
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f32 {
        self.inner.get_remain_sample_buffer_sec()
    }

    pub fn set_playback_position(&self, position_sec: f32) {
        self.inner.set_playback_position(position_sec)
    }

    pub fn get_buffer_status(&self) -> AudioSampleBufferStatus {
        self.inner.get_buffer_status()
    }

    pub fn get_content_length(&self) -> f32 {
        self.inner.source.length
    }

}

impl Drop for AudioSample {
    fn drop(&mut self) {
        self.inner.set_buffer_status(AudioSampleBufferStatus::StopFillBuffer);

        for thread_run_state in &self.thread_run_states {
            thread_run_state.store(false, Ordering::Relaxed);
        }
    }
}

pub struct AudioSampleInner {
    pub source: AudioSource,
    sample_buffer: RwLock<Vec<VecDeque<f32>>>,
    playback_sample_frame_idx: Arc<AtomicUsize>,
    pub buffer_status: Arc<AtomicUsize>,
    host_sample_rate: u32,
    host_output_channels: usize,
    buf_req_pos: Arc<AtomicUsize>,
    done_fill_buf_condvar: Arc<(Mutex<bool>, Condvar)>,
    // opus_decoder: Arc<opus::Decoder>,
}

impl AudioSampleInner {
    pub fn new(source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {
        let mut sample_buffer: Vec<VecDeque<f32>> = Vec::with_capacity(2);
        for _ in 0..host_output_channels {
            sample_buffer.push(VecDeque::new());
        }

        Self {
            source,
            sample_buffer: RwLock::new(sample_buffer),
            playback_sample_frame_idx: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleBufferStatus::StartFillBuffer as usize)),
            host_sample_rate,
            host_output_channels,
            buf_req_pos: Arc::new(AtomicUsize::new(0)),
            done_fill_buf_condvar: Arc::new((Mutex::new(true), Condvar::new())),
            // opus_decoder: Arc::new(opus_decoder),
        }
    }

    fn get_remain_sample_buffer_len(&self) -> usize {
        self.sample_buffer.read().unwrap()[0].len()
    }

    fn get_playback_sample_frame_idx(&self) -> usize {
        self.playback_sample_frame_idx.load(Ordering::SeqCst)
    }

    fn convert_sample_frame_idx_to_sec(&self, sample_frame_idx: usize, sample_rate: u32) -> f32 {
        sample_frame_idx as f32 / sample_rate as f32
    }

    fn convert_sec_to_sample_frame_idx(&self, sec: f32, sample_rate: u32) -> usize {
        (sec * sample_rate as f32).floor() as usize
    }

    fn get_current_playback_position_sec(&self) -> f32 {
        self.convert_sample_frame_idx_to_sec(self.get_playback_sample_frame_idx(), self.host_sample_rate)
    }

    fn get_remain_sample_buffer_sec(&self) -> f32 {
        self.convert_sample_frame_idx_to_sec(self.get_remain_sample_buffer_len(), self.host_sample_rate)
    }

    fn get_playback_sample_frame_idx_from_sec(&self, sec: f32) -> usize {
        self.convert_sec_to_sample_frame_idx(sec, self.host_sample_rate)
    }

    fn set_playback_sample_frame_idx(&self, sample_frame_idx: usize) {
        self.playback_sample_frame_idx.store(sample_frame_idx, Ordering::SeqCst);
    }

    fn get_buf_req_pos(&self) -> usize {
        self.buf_req_pos.load(Ordering::SeqCst)
    }

    fn set_buf_req_pos(&self, pos: usize) {
        self.buf_req_pos.store(
            pos, 
            Ordering::SeqCst
        );
    }

    fn get_buffer_status(&self) -> AudioSampleBufferStatus {
        AudioSampleBufferStatus::from(self.buffer_status.load(Ordering::Relaxed))
    } 

    fn set_buffer_status(&self, status: AudioSampleBufferStatus) {
        self.buffer_status.store(status as usize, Ordering::Relaxed);
    }

    fn set_playback_position(&self, position_sec: f32) {
        self.set_buffer_status(AudioSampleBufferStatus::StopFillBuffer);
        
        let done_fill_buf_condvar_clone = self.done_fill_buf_condvar.clone();
        let (done_fill_buf_condvar_lock, done_fill_buf_cv) = &*done_fill_buf_condvar_clone;
        let mut done_fill_buf = done_fill_buf_condvar_lock.lock().unwrap();

        while !*done_fill_buf {
            done_fill_buf = done_fill_buf_cv.wait(done_fill_buf).unwrap();
        }

        let position_sample_idx = self.get_playback_sample_frame_idx_from_sec(position_sec);
        let position_sec_delta = position_sec - self.get_current_playback_position_sec();

        let drain_buffer_len = 
            if position_sec_delta > 0.0 { position_sample_idx - self.get_playback_sample_frame_idx() } 
            else { self.get_remain_sample_buffer_len() };

        self.drain_sample_buffer(drain_buffer_len);

        let sample_req_start_sec = 
            if position_sec_delta > 0.0 { position_sec + self.get_remain_sample_buffer_sec() }
            else { position_sec };

        let buf_req_start_pos = (sample_req_start_sec * self.host_sample_rate as f32) as usize;
        self.set_buf_req_pos(buf_req_start_pos);
        self.set_playback_sample_frame_idx(position_sample_idx);

        self.set_buffer_status(AudioSampleBufferStatus::StartFillBuffer);
    }

    fn drain_sample_buffer(&self, drain_len: usize) {
        let mut sample_buffer = self.sample_buffer.write().unwrap();

        let min_drain_buffer_len = std::cmp::min(sample_buffer[0].len(), drain_len);

        for ch_sample_buffer in sample_buffer.iter_mut() {
            ch_sample_buffer.drain(..min_drain_buffer_len);
        }
    }

    async fn fetch_buffer(
        &self,
        server_address: &str,
        playback_buffer_margin_sec: f32,
    ) -> Result<(), anyhow::Error> {
        if self.get_buffer_status() != AudioSampleBufferStatus::StartFillBuffer {
            return Ok(());
        }

        println!("start fill buffer");

        self.set_buffer_status(AudioSampleBufferStatus::StartedFillBuffer);

        let done_fill_buf_condvar_clone = self.done_fill_buf_condvar.clone();
        let (done_fill_buf_condvar_lock, done_fill_buf_cv) = &*done_fill_buf_condvar_clone;

        {
            let mut done_fill_buf = done_fill_buf_condvar_lock.lock().unwrap();
            *done_fill_buf = false;
        }

        let fetch_buffer_sec = playback_buffer_margin_sec - self.get_remain_sample_buffer_sec();
        if let Err(_err) = self.get_buffer_for(server_address, fetch_buffer_sec).await {
            // println!("fetch buffer error: {:?}", err);
        }

        let mut done_fill_buf = done_fill_buf_condvar_lock.lock().unwrap();
        *done_fill_buf = true;
        done_fill_buf_cv.notify_one();

        if self.get_buffer_status() == AudioSampleBufferStatus::StartedFillBuffer {
            self.set_buffer_status(AudioSampleBufferStatus::StartFillBuffer);        
        }

        println!("done fill buffer");
        
        Ok(())
    }

    async fn get_buffer_for(&self, server_address: &str, sec: f32) -> Result<(), anyhow::Error> {
        let req_sample_frames_len = (sec * self.host_sample_rate as f32) as u32;

        let mut t = Vec::with_capacity(2);
        for _ in 0..2 {
            t.push(vec![0u8; 0]);
        }

        let mut audio_data_stream = request::get_audio_data_stream(
            server_address.to_string(),
            &self.source.id,
            self.host_sample_rate,
            self.host_output_channels.try_into().unwrap(),
            self.get_buf_req_pos() as u32,
            req_sample_frames_len,
        ).await?;

        let mut opus_decoder = opus::Decoder::new(self.host_sample_rate, opus::Channels::Stereo).unwrap();

        while let Some(res) = audio_data_stream.next().await {
            let audio_data = match res {
                Ok(data) => data,
                Err(e) => {
                    println!("err: {}", e);
                    break;
                },
            };

            if AudioSampleBufferStatus::StopFillBuffer == self.get_buffer_status() {
                self.set_buffer_status(AudioSampleBufferStatus::StoppedFillBuffer);
                println!("stopped fill buffer");

                break;
            }

            let mut sample_buffer = self.sample_buffer.write().unwrap();

            let mut decoded_samples = vec![0.; (audio_data.num_frames*2).try_into().unwrap()];

            if let Err(err) = opus_decoder.decode_float(
                &audio_data.encoded_samples, 
                &mut decoded_samples, 
                false) {
                    println!("{:?}", err);
                }
            
            let data = audio::wrap::interleaved(decoded_samples.as_slice(), 2);
            let r = audio::io::Read::new(data);
            
            for (ch_idx, channel_sample_buffer) in sample_buffer.iter_mut().enumerate() {
                channel_sample_buffer.extend(r.channel(ch_idx));
            }

            let curr_buf_req_start_idx = self.get_buf_req_pos();
            self.set_buf_req_pos(curr_buf_req_start_idx + audio_data.num_frames as usize);
        }

        Ok(())
    }
    
    pub fn play_for(&self, output: &mut [f32]) {
        for output_channel_frame in output.chunks_mut(self.source.channels) {  
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

            let playback_sample_frame_idx = self.get_playback_sample_frame_idx();
            let next_playback_frame_idx = playback_sample_frame_idx + (channel_sample_read / self.source.channels as u8) as usize;
            self.set_playback_sample_frame_idx(next_playback_frame_idx);
        }
    }
}