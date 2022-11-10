use std::{
    sync::{
        Arc, 
        atomic::{AtomicUsize, Ordering}, Condvar, Mutex
    }, 
    collections::VecDeque
};
use std::iter::Iterator;

// use anyhow::anyhow;
use futures::StreamExt;
use tokio::runtime::Handle;
use opus;
use audio::{Channels, AsInterleavedMut};

use crate::{dto::AudioSource, request};
use super::{state::AudioSampleBufferStatus, packet::{EncodedBuffer, get_packet_idx_from_sec, NodeSearchDirection}};

pub struct AudioSample {
    pub inner: Arc<AudioSampleInner>,
}

impl AudioSample {
    pub fn new(audio_source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {

        let inner = Arc::new(
            AudioSampleInner::new(
                audio_source,
                host_sample_rate, 
                host_output_channels
            )
        );

        Self {
            inner,
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

    pub fn fetch_buffer(&self, min_avail_buf_sec: f64, fetch_start_margin_sec: f64, rt_handle: Arc<Handle>) {
        if self.get_buffer_status() != AudioSampleBufferStatus::StartFillBuffer {
            return;
        }

        // check fetch requried
        let remain_buf_sec = self.inner.get_remain_sample_buffer_sec();

        if remain_buf_sec > min_avail_buf_sec ||
            remain_buf_sec - min_avail_buf_sec > fetch_start_margin_sec {
            return;
        }

        let packet_buf = self.inner.packet_buf.lock().unwrap();
        let last_buf_chunk = packet_buf.buf_chunk_info.get(&packet_buf.last_node_id).unwrap();

        let content_packets = packet_buf.content_packets;
        let last_chunk_end_idx = last_buf_chunk.lock().unwrap().end_idx;
        let chunks_num = packet_buf.get_chunks_num_from_current();

        if last_chunk_end_idx == content_packets && chunks_num == 1 {
            return;
        }
        
        // fetch start
        let inner = Arc::clone(&self.inner);
          
        rt_handle.spawn(async move {
            println!("start fill buffer thread. source id: {}", inner.source.id);

            inner.fetch_buffer(min_avail_buf_sec).await.unwrap();

            println!("stopped fill buffer thread. source id: {}", inner.source.id);
        });
    }
}

impl Drop for AudioSample {
    fn drop(&mut self) {
        self.inner.set_buffer_status(AudioSampleBufferStatus::StopFillBuffer);
    }
}

pub struct AudioSampleInner {
    pub source: AudioSource,

    playback_sample_frame_pos: Arc<AtomicUsize>,
    pub buffer_status: Arc<AtomicUsize>,
    host_sample_rate: u32,
    host_output_channels: usize,

    done_fill_buf_condvar: Arc<(Mutex<bool>, Condvar)>,
    packet_playback_idx: Arc<AtomicUsize>,

    packet_buf: Arc<Mutex<EncodedBuffer>>,
    decoded_sample_frame_buf: Arc<Mutex<Vec<VecDeque<f32>>>>,
    opus_decoder: Arc<Mutex<opus::Decoder>>,
}

impl AudioSampleInner {
    pub fn new(source: AudioSource, host_sample_rate: u32, host_output_channels: usize) -> Self {
        let mut decoded_sample_frame_buf: Vec<VecDeque<f32>> = Vec::with_capacity(2);

        for _ in 0..host_output_channels {
            decoded_sample_frame_buf.push(VecDeque::new());
        }

        let od = opus::Decoder::new(48_000, opus::Channels::Stereo).unwrap();

        let content_pckts = source.content_packets;

        Self {
            source,

            playback_sample_frame_pos: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleBufferStatus::StartFillBuffer as usize)),
            host_sample_rate,
            host_output_channels,
            done_fill_buf_condvar: Arc::new((Mutex::new(true), Condvar::new())),

            packet_playback_idx: Arc::new(AtomicUsize::new(0)),
            packet_buf: Arc::new(Mutex::new(EncodedBuffer::new(content_pckts))),

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

        remain_packets as f64 * 0.02
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
        let updated_playback_pkt_idx = get_packet_idx_from_sec(position_sec, 0.02);
        self.packet_playback_idx.store(updated_playback_pkt_idx, Ordering::SeqCst);
        println!("updated playback packet index: {}", updated_playback_pkt_idx);
        
        let position_sec_delta = position_sec - self.get_current_playback_position_sec();

        let packet_buf_update_dir = 
            if position_sec_delta > 0.0 
                { NodeSearchDirection::Forward } 
            else 
                { NodeSearchDirection::Backward };

        self.packet_buf.lock().unwrap().update_seek_position(position_sec, packet_buf_update_dir);
        self.set_playback_sample_frame_pos(position_sample_idx);

        let mut ds_buf = self.decoded_sample_frame_buf.lock().unwrap();
        for channel_sample_buffer in ds_buf.iter_mut() {
            channel_sample_buffer.clear();
        }

        self.set_buffer_status(AudioSampleBufferStatus::StartFillBuffer);
    }

    async fn fetch_buffer(
        &self,
        fetch_buf_sec: f64,
    ) -> Result<(), anyhow::Error> {
        if self.get_buffer_status() != AudioSampleBufferStatus::StartFillBuffer {
            return Ok(());
        }

        self.set_buffer_status(AudioSampleBufferStatus::StartedFillBuffer);

        let done_fill_buf_condvar_clone = self.done_fill_buf_condvar.clone();
        let (done_fill_buf_condvar_lock, done_fill_buf_cv) = &*done_fill_buf_condvar_clone;
        
        {
            let mut done_fill_buf = done_fill_buf_condvar_lock.lock().unwrap();
            *done_fill_buf = false;
        }

        if let Err(_err) = self.get_buffer_for(fetch_buf_sec).await {
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

    async fn get_buffer_for(
        &self, 
        duration_sec: f64
    ) -> Result<(), anyhow::Error> {
        let fetch_start_pkt_idx = self.packet_buf.lock().unwrap().next_packet_idx;
        let fetch_packet_num = self.packet_buf
            .lock()
            .unwrap()
            .get_fetch_required_packet_num(
                fetch_start_pkt_idx,
                duration_sec,
            );
        // let fetch_start_pkt_idx = pb.next_packet_idx;
        // let fetch_packet_num = pb
        //     .get_fetch_required_packet_num(
        //         fetch_start_pkt_idx,
        //         duration_sec,
        //     );

        if fetch_packet_num == 0 {
            println!("warn: attempted to fetch 0 packets");
            // workaround
            self.packet_buf.lock().unwrap().merge_node_from_current();
            return Ok(());
        }

        // let (pkt_seek_start_pkt_idx, next_pkt_start_ts) = self.packet_buf.lock().unwrap().get_next_packet_start_ts_from_current();

        let mut audio_data_stream = request::get_audio_data_stream(
            &self.source.server.grpc_endpoint,
            &self.source.server.tls_config,
            &self.source.id,
            fetch_start_pkt_idx.try_into().unwrap(),
            fetch_packet_num.try_into().unwrap(),
            2,
            0,
            0,
            // pkt_seek_start_pkt_idx,
            // next_pkt_start_ts.into(),
        ).await?;

        println!("fetch packet: ({}..{})", fetch_start_pkt_idx, fetch_start_pkt_idx+fetch_packet_num);
        // println!("seek buf chunk id: {}", self.packet_buf.lock().unwrap().seek_buf_chunk_node_idx);
        let mut last_idx = 0;

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
                println!("buffer status is stop. interrupting fill buffer");

                break;
            }

            let mut packet_buf = self.packet_buf.lock().unwrap();
            last_idx = audio_data.packet_idx;
            packet_buf.insert(audio_data);

            // // for test 
            // break;
        }

        println!("last pushed packet id: {}", last_idx);

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

                // let t = od.get_nb_samples(&eb.encoded_samples).unwrap();
                // let d = od.get_last_packet_duration().unwrap();
                // let t2 = opus::packet::parse(&eb.encoded_samples).unwrap();

                // let a = t2.frames;

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
                println!("err: packet {} is missing", p_pos);

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
