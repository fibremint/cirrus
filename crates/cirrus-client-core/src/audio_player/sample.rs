use std::{
    sync::{
        Arc, 
        atomic::{AtomicUsize, Ordering, AtomicBool}, Condvar, Mutex
    }, 
    collections::{VecDeque, HashMap},
};
use std::iter::Iterator;

use cirrus_protobuf::api::AudioDataRes;
// use anyhow::anyhow;
use futures::StreamExt;
use tokio::sync::mpsc;
use opus;
use audio::{Channels, AsInterleavedMut};

use rand::Rng;

use crate::{dto::AudioSource, request};
use super::state::AudioSampleBufferStatus;

pub struct AudioSample {
    pub inner: Arc<AudioSampleInner>,
    pub tx: mpsc::Sender<f64>,
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
                    inner_1_clone.fetch_buffer(fetch_buf_sec).await.unwrap();
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

    pub fn get_current_playback_position_sec(&self) -> f64 {
        self.inner.get_current_playback_position_sec()
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f64 {
        self.inner.get_remain_sample_buffer_sec()
    }

    pub fn set_playback_position(&self, position_sec: f64) {
        self.inner.set_playback_position(position_sec)
    }

    pub fn get_buffer_status(&self) -> AudioSampleBufferStatus {
        self.inner.get_buffer_status()
    }

    pub fn get_content_length(&self) -> f64 {
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

struct BufChunkInfoNode {
    pub id: u32,
    pub start_idx: u32,
    pub end_idx: u32,

    pub prev_info: Option<Arc<Mutex<BufChunkInfoNode>>>,
    pub next_info: Option<Arc<Mutex<BufChunkInfoNode>>>,
}

impl BufChunkInfoNode {
    pub fn new(
        idx_from: u32, 
        prev_info: Option<Arc<Mutex<BufChunkInfoNode>>>, 
        next_info: Option<Arc<Mutex<BufChunkInfoNode>>>
    ) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();

        Self { 
            id,
            start_idx: idx_from,
            end_idx: idx_from+1,
            prev_info, 
            next_info 
        }
    }
}

struct EncodedBuffer {
    pub frame_buf: HashMap<u32, AudioDataRes>, // packet idx, packet
    pub buf_chunk_info: HashMap<u32, Arc<Mutex<BufChunkInfoNode>>>,
    pub seek_buf_chunk_node_idx: u32,
}

impl Default for EncodedBuffer {
    fn default() -> Self {
        let mut buf_chunk_info = HashMap::new();

        let bci_node = BufChunkInfoNode::new(0, None, None);
        let bci_node_id = bci_node.id;
        let bci_node = Arc::new(Mutex::new(bci_node));

        buf_chunk_info.insert(bci_node_id.try_into().unwrap(), bci_node);

        Self { 
            frame_buf: Default::default(), 
            buf_chunk_info: buf_chunk_info,
            seek_buf_chunk_node_idx: bci_node_id,
        }
    }
}

impl EncodedBuffer {
    fn check_buf_chunk(&mut self, audio_data: &AudioDataRes) {
        {
            let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
            let mut bc = bci_node.lock().unwrap();
    
            if audio_data.packet_idx > bc.end_idx {
                let next_bci_node = BufChunkInfoNode::new(
                    audio_data.packet_idx,
                    Some(bci_node.clone()),
                    None
                );
                let next_bci_node_id = next_bci_node.id; 
    
                let next_bci_node = Arc::new(Mutex::new(next_bci_node));
                bc.next_info = Some(Arc::clone(&next_bci_node));
    
                self.seek_buf_chunk_node_idx = next_bci_node_id;
                self.buf_chunk_info.insert(next_bci_node_id, next_bci_node);
            } 
            
            if audio_data.packet_idx < bc.start_idx {
                let prev_bci_node = BufChunkInfoNode::new(
                    audio_data.packet_idx, 
                    None, 
                    Some(bci_node.clone())
                );
                let prev_bci_node_id = prev_bci_node.id;

                let prev_bci_node = Arc::new(Mutex::new(prev_bci_node));
                bc.prev_info = Some(Arc::clone(&prev_bci_node));

                self.seek_buf_chunk_node_idx = prev_bci_node_id;
                self.buf_chunk_info.insert(prev_bci_node_id, prev_bci_node);
            }
        }

        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let mut bc = bci_node.lock().unwrap();

        if bc.next_info.is_none() {
            return;
        }

        let next_node = bc.next_info.clone().unwrap();
        let nn = next_node.lock().unwrap();

        if audio_data.packet_idx < nn.start_idx {
            return;
        }

        bc.next_info = nn.next_info.clone();
        bc.end_idx = nn.end_idx;
        let nn_id = nn.id;

        self.buf_chunk_info.remove(&nn_id);
    }

    fn push(&mut self, audio_data: AudioDataRes) {
        self.check_buf_chunk(&audio_data);

        let bci_node = self.buf_chunk_info.get(&self.seek_buf_chunk_node_idx).unwrap().to_owned();        
        let mut bc = bci_node.lock().unwrap();

        self.frame_buf.insert(audio_data.packet_idx, audio_data);
        bc.end_idx += 1;
    }
}

pub struct AudioSampleInner {
    pub source: AudioSource,

    playback_sample_frame_pos: Arc<AtomicUsize>,
    pub buffer_status: Arc<AtomicUsize>,
    host_sample_rate: u32,
    host_output_channels: usize,

    done_fill_buf_condvar: Arc<(Mutex<bool>, Condvar)>,
    buf_req_start_idx: Arc<AtomicUsize>,
    packet_playback_idx: Arc<AtomicUsize>,

    packet_buf: Arc<Mutex<EncodedBuffer>>,
    decoded_sample_frame_buf: Arc<Mutex<Vec<VecDeque<f32>>>>,
    opus_decoder: Arc<Mutex<opus::Decoder>>,
}

fn get_packet_idx_from_sec(sec: f64, packet_dur: f64) -> usize {
    (sec / packet_dur).floor() as usize
}

impl AudioSampleInner {
    pub fn new(source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {
        let mut decoded_sample_frame_buf: Vec<VecDeque<f32>> = Vec::with_capacity(2);

        for _ in 0..host_output_channels {
            decoded_sample_frame_buf.push(VecDeque::new());
        }

        let od = opus::Decoder::new(48_000, opus::Channels::Stereo).unwrap();

        Self {
            source,

            playback_sample_frame_pos: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleBufferStatus::StartFillBuffer as usize)),
            host_sample_rate,
            host_output_channels,
            done_fill_buf_condvar: Arc::new((Mutex::new(true), Condvar::new())),
            buf_req_start_idx: Arc::new(AtomicUsize::new(0)),

            packet_playback_idx: Arc::new(AtomicUsize::new(0)),
            packet_buf: Arc::new(Mutex::new(EncodedBuffer::default())),

            decoded_sample_frame_buf: Arc::new(Mutex::new(decoded_sample_frame_buf)),
            opus_decoder: Arc::new(Mutex::new(od)),
        }
    }

    fn get_remain_sample_buffer_sec(&self) -> f64 {
        let p_pos = self.packet_playback_idx.load(Ordering::SeqCst) as i32;

        let packet_buf = self.packet_buf.lock().unwrap();
        let buf_chunk_info = packet_buf.buf_chunk_info.get(&packet_buf.seek_buf_chunk_node_idx).unwrap();

        let bci = buf_chunk_info.lock().unwrap();

        let remain_packets: i32 = bci.end_idx as i32 - p_pos - 1;

        if p_pos < bci.start_idx.try_into().unwrap() || remain_packets < 0 {
            return 0.
        }

        remain_packets as f64 * 0.06
    }

    fn get_playback_sample_frame_pos(&self) -> usize {
        self.playback_sample_frame_pos.load(Ordering::SeqCst)
    }

    fn convert_sample_frame_idx_to_sec(&self, sample_frame_idx: usize, sample_rate: u32) -> f64 {
        sample_frame_idx as f64 / sample_rate as f64
    }

    fn convert_sec_to_sample_frame_idx(&self, sec: f64, sample_rate: u32) -> usize {
        (sec * sample_rate as f64).floor() as usize
    }

    fn get_current_playback_position_sec(&self) -> f64 {
        self.convert_sample_frame_idx_to_sec(self.get_playback_sample_frame_pos(), self.host_sample_rate)
    }

    fn get_playback_sample_frame_pos_from_sec(&self, sec: f64) -> usize {
        self.convert_sec_to_sample_frame_idx(sec, self.host_sample_rate)
    }

    fn set_playback_sample_frame_pos(&self, sample_frame_idx: usize) {
        self.playback_sample_frame_pos.store(sample_frame_idx, Ordering::SeqCst);
    }

    fn get_buffer_status(&self) -> AudioSampleBufferStatus {
        AudioSampleBufferStatus::from(self.buffer_status.load(Ordering::Relaxed))
    } 

    fn set_buffer_status(&self, status: AudioSampleBufferStatus) {
        self.buffer_status.store(status as usize, Ordering::Relaxed);
    }

    fn set_playback_position(&self, position_sec: f64) {
        self.set_buffer_status(AudioSampleBufferStatus::StopFillBuffer);
        
        let done_fill_buf_condvar_clone = self.done_fill_buf_condvar.clone();
        let (done_fill_buf_condvar_lock, done_fill_buf_cv) = &*done_fill_buf_condvar_clone;
        let mut done_fill_buf = done_fill_buf_condvar_lock.lock().unwrap();

        while !*done_fill_buf {
            done_fill_buf = done_fill_buf_cv.wait(done_fill_buf).unwrap();
        }

        let position_sample_idx = self.get_playback_sample_frame_pos_from_sec(position_sec);
        let position_sec_delta = position_sec - self.get_current_playback_position_sec();
        let pos = get_packet_idx_from_sec(position_sec, 0.06);

        self.packet_playback_idx.store(pos, Ordering::SeqCst);

        let sample_req_start_sec = 
            if position_sec_delta > 0.0 { position_sec + self.get_remain_sample_buffer_sec() }
            else { position_sec };

        let buf_req_idx = get_packet_idx_from_sec(sample_req_start_sec, 0.06);

        self.buf_req_start_idx.store(buf_req_idx, Ordering::SeqCst);
        self.set_playback_sample_frame_pos(position_sample_idx);

        let mut ds_buf = self.decoded_sample_frame_buf.lock().unwrap();
        for channel_sample_buffer in ds_buf.iter_mut() {
            channel_sample_buffer.clear();
        }

        self.set_buffer_status(AudioSampleBufferStatus::StartFillBuffer);
    }

    async fn fetch_buffer(
        &self,
        playback_buffer_margin_sec: f64,
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
        if let Err(_err) = self.get_buffer_for(&self.source.server_address, fetch_buffer_sec).await {
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

    async fn get_buffer_for(&self, server_address: &str, duration_sec: f64) -> Result<(), anyhow::Error> {
        let packet_start_idx = self.buf_req_start_idx.load(Ordering::SeqCst);
        let packet_num = get_packet_idx_from_sec(duration_sec, 0.06);

        let mut audio_data_stream = request::get_audio_data_stream(
            server_address.to_string(),
            &self.source.id,
            packet_start_idx.try_into().unwrap(),
            packet_num.try_into().unwrap(),
            2,
        ).await?;

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

            let mut t = self.packet_buf.lock().unwrap();
            t.push(audio_data);

            let buf_req_start_idx = self.buf_req_start_idx.load(Ordering::SeqCst);
            self.buf_req_start_idx.store(buf_req_start_idx+1, Ordering::SeqCst);
        }

        Ok(())
    }
    
    pub fn play_for(&self, output: &mut [f32]) {
        let mut ds_buf = self.decoded_sample_frame_buf.lock().unwrap();
        
        while ds_buf[0].len() < output.len() {
            let enc_buf = self.packet_buf.lock().unwrap();
            let mut od = self.opus_decoder.lock().unwrap();
            let p_pos = self.packet_playback_idx.load(Ordering::SeqCst) as u32;

            if let Some(eb) = enc_buf.frame_buf.get(&p_pos) {
                let mut decoded_samples = vec![0.; (eb.sp_frame_num*2).try_into().unwrap()];
                let mut decoded_samples = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);

                if let Err(err) = od.decode_float(
                    &eb.encoded_samples, 
                    &mut decoded_samples.as_interleaved_mut(),
                    false
                ) {
                      println!("{:?}", err);
                }

                let r = audio::io::Read::new(decoded_samples);

                for (ch_idx, channel_sample_buffer) in ds_buf.iter_mut().enumerate() {
                    channel_sample_buffer.extend(r.channel(ch_idx));
                }

                self.packet_playback_idx.store(p_pos as usize +1, Ordering::SeqCst);

            } else {
                break;
            }
        }

        for output_channel_frame in output.chunks_mut(self.source.channels) {  
            let mut channel_sample_read: u8 = 0;
          
            for channel_idx in 0..self.host_output_channels {
                if let Some(channel_sample) = ds_buf[channel_idx].pop_front() {
                    output_channel_frame[channel_idx] = channel_sample;
                    channel_sample_read += 1;
                } else {
                    output_channel_frame[channel_idx] = 0.0;
                }
            }

            let playback_sample_pos = self.get_playback_sample_frame_pos();
            let next_playback_frame_idx = playback_sample_pos + (channel_sample_read / self.source.channels as u8) as usize;
            self.set_playback_sample_frame_pos(next_playback_frame_idx);
        }
    }
}
