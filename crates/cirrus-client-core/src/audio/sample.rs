use std::{sync::{Arc, Mutex, Condvar, atomic::{AtomicUsize, Ordering, AtomicU32}}};
use audio::InterleavedBuf;
use cirrus_protobuf::api::AudioDataRes;
use tokio::{runtime::Handle, sync::RwLock};
use tokio_stream::StreamExt;

use crate::{dto::AudioSource, request};

use super::{packet::PacketBuffer, stream::AudioStreamBufferProducer, resampler::AudioResampler, decoder::PacketDecoder};

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum FetchBufferStatus {
    Init,
    Filling,
    Filled,
    Interrupted,
    Error,
}

impl From<usize> for FetchBufferStatus {
    fn from(value: usize) -> Self {
        use self::FetchBufferStatus::*;
        match value {
            0 => Init,
            1 => Filling,
            2 => Filled,
            3 => Interrupted,
            4 => Error,
            _ => unreachable!(),
        }
    }
}


#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum FetchBufferRequest {
    None,
    Stop,
}

impl From<usize> for FetchBufferRequest {
    fn from(value: usize) -> Self {
        use self::FetchBufferRequest::*;
        match value {
            0 => None,
            1 => Stop,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum ProcessAudioDataStatus {
    Init,
    Processing,
    Paused,
    WaitConsume,
    DataNotExist,
    ReactEnd,
    Error,
}

impl From<usize> for ProcessAudioDataStatus {
    fn from(value: usize) -> Self {
        use self::ProcessAudioDataStatus::*;
        match value {
            0 => Init,
            1 => Processing,
            2 => Paused,
            3 => WaitConsume,
            4 => DataNotExist,
            5 => ReactEnd,
            6 => Error,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum Action {
    Start,
    Stop,
}

impl From<usize> for Action {
    fn from(value: usize) -> Self {
        use self::Action::*;
        match value {
            0 => Start,
            1 => Stop,
            _ => unreachable!(),
        }
    }
}

// #[derive(Clone, serde_derive::Serialize)]
// pub enum UpdatedBufferMessage {
//     RemainSampleBufferSec(f32),
//     BufferStatus(FetchBufferStatus),
// }

pub struct FetchBufferSpec {
    pub init_fetch_sec: Option<u32>,
    pub buffer_margin_sec: u32,
    pub fetch_packet_sec: u32,
}

pub struct AudioSample {
    pub inner: Arc<Mutex<AudioSampleInner>>,
    
    fetch_buffer_action: Arc<AtomicUsize>,
}

impl AudioSample {
    pub fn new(
        source: AudioSource,
        // host_stream_config: HostStreamConfig,
        output_stream_config: &cpal::StreamConfig,
        audio_stream_buf_producer: AudioStreamBufferProducer,
        fetch_buffer_spec: FetchBufferSpec,
    ) -> Result<Self, anyhow::Error> {

        Ok(Self {
            inner: Arc::new(
                Mutex::new(
                    AudioSampleInner::new(
                        source,
                        output_stream_config,
                        audio_stream_buf_producer,
                        fetch_buffer_spec,
                    )?
                )
            ),

            fetch_buffer_action: Arc::new(AtomicUsize::new(Action::Start as usize)),
        })
    }

    pub fn start_process_audio_data_thread(
        &self,
        rt_handle: &Handle,
    ) {
        let _rt_handle = rt_handle.clone();
        let _inner = self.inner.clone();
        let _process_sample_condvar = _inner.lock().unwrap().context.process_sample_condvar.clone();
        let _fetch_buffer_action = self.fetch_buffer_action.clone();

        std::thread::spawn(move || loop {
            {
                if Action::Stop == Action::from(
                    _fetch_buffer_action.load(Ordering::SeqCst)
                ) {
                    break;
                }

                let (process_sample_mutex, process_sample_cv) = &*_process_sample_condvar;
                let mut process_sample_guard = process_sample_mutex.lock().unwrap();
    
                // Wait until the value of condvar is changed to true and notified
                while !*process_sample_guard {
                    process_sample_guard = process_sample_cv.wait(process_sample_guard).unwrap();
                }
    
            }

            _inner.lock().unwrap().process_audio_data(&_rt_handle);
            // if let Err(e) = _inner.lock().unwrap().process_audio_data(&_rt_handle) {
            //     match e {
                    
            //     }
            // }
        });
    }

    pub fn set_playback_position(
        &self,
        position_sec: f64
    ) -> Result<(), anyhow::Error> {
        self.inner.lock().unwrap().set_playback_position(position_sec)?;

        Ok(())
    }

    // pub fn update_from_output_stream_config(&self) {
    //     todo!()
    // }
}

impl Drop for AudioSample {
    fn drop(&mut self) {
        self.fetch_buffer_action.store(Action::Stop as usize, Ordering::SeqCst);
    }
}

pub struct AudioSampleInner {
    pub source: AudioSource,
    pub context: AudioSampleContext,

    packet_buffer: Arc<RwLock<PacketBuffer>>,

    packet_decoder: PacketDecoder,
    resampler: AudioResampler,

    audio_stream_buf_producer: AudioStreamBufferProducer,

    fetch_buffer_spec: FetchBufferSpec,

}

impl AudioSampleInner {
    pub fn new(
        source: AudioSource,
        output_stream_config: &cpal::StreamConfig,
        audio_stream_buf_producer: AudioStreamBufferProducer,
        fetch_buffer_spec: FetchBufferSpec,
    ) -> Result<Self, anyhow::Error> {
        let packet_buffer = PacketBuffer::new(source.content_packets);
        let packet_decoder = PacketDecoder::new()?;

        Ok(Self {
            source,
            packet_buffer: Arc::new(RwLock::new(packet_buffer)),
            packet_decoder,
            resampler: AudioResampler::new(
                output_stream_config.sample_rate.0.try_into()?,
                output_stream_config.channels.into(),
            )?,
            context: AudioSampleContext::default(),
            audio_stream_buf_producer,
            fetch_buffer_spec,
        })
    }

    pub fn set_playback_position(
        &mut self,
        position_sec: f64
    ) -> Result<(), anyhow::Error> {
        self.set_fetch_buffer_action(Action::Stop, None)?;

        let new_position_idx = position_sec as u32 * 50;

        self.context.playback_sample_frame_pos.store(new_position_idx, Ordering::SeqCst);

        self.context.fetch_buffer_status.store(FetchBufferStatus::Filled as usize, Ordering::SeqCst);

        Ok(())
    }

    pub fn set_fetch_buffer_action(
        &mut self,
        action: Action,
        rt_handle: Option<&Handle>,
    ) -> Result<(), anyhow::Error> {
        let fetch_buffer_status = FetchBufferStatus::from(
            self.context.fetch_buffer_status.load(Ordering::SeqCst)
        );
        
        match action {
            Action::Start => {
                if fetch_buffer_status == FetchBufferStatus::Filling || 
                    fetch_buffer_status == FetchBufferStatus::Interrupted {
                    
                    return Ok(())
                }

                let remain_buffer_sec = self.packet_buffer.blocking_read().get_remain_buffer(
                    self.context.playback_sample_frame_pos.load(Ordering::SeqCst)
                ) / 50;

                if remain_buffer_sec > self.fetch_buffer_spec.buffer_margin_sec {
                    return Ok(())
                }

                if self.packet_buffer.blocking_read().is_reached_last_fetch_index() {
                    return Ok(())
                }
                
                self.context.fetch_buffer_status.store(FetchBufferStatus::Filling as usize, Ordering::SeqCst);

                self.start_fetch_buffer_task(rt_handle.unwrap(), self.fetch_buffer_spec.fetch_packet_sec)?;

            },
            Action::Stop => {
                if fetch_buffer_status != FetchBufferStatus::Filling {
                    return Ok(())
                } 

                self.context.fetch_buffer_request.store(FetchBufferRequest::Stop as usize, Ordering::SeqCst);
        
                let (fetch_buffer_mutex, fetch_buffer_cv) = &*self.context.fetch_buffer_condvar;
                let mut fetch_buffer_guard = fetch_buffer_mutex.lock().unwrap();
    
                while *fetch_buffer_guard {
                    fetch_buffer_guard = fetch_buffer_cv.wait(fetch_buffer_guard).unwrap();
                }
            },
        }

        Ok(())
    }

    fn start_fetch_buffer_task(
        &mut self,
        rt_handle: &Handle,
        fetch_sec: u32,
    ) -> Result<(), anyhow::Error> {
        let audio_tag_id = self.source.id.clone();
        let _fetch_buffer_status = self.context.fetch_buffer_status.clone();
        let _packet_buffer = self.packet_buffer.clone();
        let _content_packets = self.source.content_packets;

        let _fetch_buffer_condvar = self.context.fetch_buffer_condvar.clone();
        let _fetch_buffer_request = self.context.fetch_buffer_request.clone();

        let _playback_sample_frame_pos = self.context.playback_sample_frame_pos.clone();
        
        let mut fetch_packet_cnt = 0;
        let fetch_required_packet_num = fetch_sec * 50;

        let mut is_interrupted = false;

        rt_handle.spawn(async move {
            let (fetch_buffer_mutex, fetch_buffer_condvar) = &*_fetch_buffer_condvar;

            // start fetch buffer
            _fetch_buffer_status.store(FetchBufferStatus::Filling as usize, Ordering::SeqCst);
            {
                let mut fetch_buffer_guard = fetch_buffer_mutex.lock().unwrap();
                *fetch_buffer_guard = true;
            }

            loop {
                if fetch_packet_cnt == fetch_required_packet_num {
                    _fetch_buffer_status.store(FetchBufferStatus::Filled as usize, Ordering::SeqCst);
                    break;
                }

                let (fetch_start_idx, fetch_size) = _packet_buffer.write().await.fetch_buffer_guidance(
                    _playback_sample_frame_pos.load(Ordering::SeqCst),
                    fetch_required_packet_num - fetch_packet_cnt
                );

                if fetch_size == 0 {
                    _fetch_buffer_status.store(FetchBufferStatus::Filled as usize, Ordering::SeqCst);
                    break;
                }

                if is_interrupted {
                    _fetch_buffer_status.store(FetchBufferStatus::Interrupted as usize, Ordering::SeqCst);
                    break;
                }

                let mut audio_data_stream = match request::get_audio_data_stream(
                    "http://localhost:50000", 
                    &None, 
                    &audio_tag_id,
                    fetch_start_idx,
                    fetch_size, 
                    2
                ).await {
                    Ok(stream) => stream,
                    Err(err) => {
                        eprintln!("{}", err);
                        _fetch_buffer_status.store(FetchBufferStatus::Error as usize, Ordering::SeqCst);

                        {
                            let mut fetch_buffer_guard = fetch_buffer_mutex.lock().unwrap();
                            *fetch_buffer_guard = false;
                            fetch_buffer_condvar.notify_one();
                        }

                        return;
                    },
                };

                println!("fetch packet: {}..{}", fetch_start_idx, fetch_start_idx+fetch_size);
        
                while let Some(res) = audio_data_stream.next().await {
                    if FetchBufferRequest::Stop == FetchBufferRequest::from(_fetch_buffer_request.load(Ordering::SeqCst)) {
                        _fetch_buffer_request.store(FetchBufferRequest::None as usize, Ordering::SeqCst);
                        is_interrupted = true;

                        break;
                    }

                    let audio_data = match res {
                        Ok(data) => data,
                        Err(e) => {
                            println!("err: {}", e);
                            _fetch_buffer_status.store(FetchBufferStatus::Error as usize, Ordering::SeqCst);

                            break;
                        }
                    };

                    if let Err(e) = _packet_buffer.write().await.insert(audio_data) {
                        eprintln!("failed to insert audio data: {}", e.to_string());
                    }

                    fetch_packet_cnt += 1;
                }
            }

            {
                let mut fetch_buffer_guard = fetch_buffer_mutex.lock().unwrap();
                *fetch_buffer_guard = false;
                fetch_buffer_condvar.notify_one();
            }

            println!("done fetch buffer, fetch cnt: {}", fetch_packet_cnt )
        });

        Ok(())
    }

    fn check_process_available(
        &self,
    ) -> Result<AudioDataRes, anyhow::Error> {
        let (process_sample_mutex, _) = &*self.context.process_sample_condvar;
        let playback_sample_frame_pos = self.context.playback_sample_frame_pos.load(Ordering::SeqCst);
        let mut process_sample_guard = process_sample_mutex.lock().unwrap();
        
        // Buffer has not enough spaces to fill buffer
        // Wait for consumer consumes buffer
        // let processed_sample_len = self.resampler.resampler.output_frames_max() * 2;
        let processed_sample_len = self.resampler.get_processed_sample_len();

        if processed_sample_len > self.audio_stream_buf_producer.free_len() {
            // Set status to wait for audio stream output function consumes buffer 
            self.context.process_audio_data_status.store(ProcessAudioDataStatus::WaitConsume as usize, Ordering::SeqCst);
            // Set wait status
            *process_sample_guard = false;

            return Err(anyhow::anyhow!(ProcessAudioDataStatus::WaitConsume as usize))
        }

        // if self.context.playback_sample_frame_pos == self.source.content_packets -1 {
        if playback_sample_frame_pos == self.source.content_packets -1 {
            self.context.process_audio_data_status.store(ProcessAudioDataStatus::ReactEnd as usize, Ordering::SeqCst);
            
            *process_sample_guard = false;
            
            return Err(anyhow::anyhow!(ProcessAudioDataStatus::ReactEnd as usize))
        }

        // audio data is not fetched  
        let packet_buf_guard = self.packet_buffer.blocking_read();

        let data = packet_buf_guard.get_data(playback_sample_frame_pos);
        if data.is_none() {
            // Set status that audio sample packet trying to process does not exist
            self.context.process_audio_data_status.store(ProcessAudioDataStatus::DataNotExist as usize, Ordering::SeqCst);
            // Set wait status
            *process_sample_guard = false;

            return Err(anyhow::anyhow!(ProcessAudioDataStatus::DataNotExist as usize))
        }

        Ok(data.unwrap().to_owned())
    }

    pub fn process_audio_data(
        &mut self,
        rt_handle: &Handle,
    ) -> Result<(), anyhow::Error> {
        // Request fetch buffer
        self.set_fetch_buffer_action(Action::Start, Some(rt_handle))?;           
        
        // Check a processing of audio data is required
        let data = self.check_process_available()?;

        // Process audio data
        let samples = self.packet_decoder.decode(&data.encoded_samples)?;
        let samples = self.resampler.resample(samples)?;

        // Push audio samples into the stream buffer
        self.audio_stream_buf_producer.push_slice(samples.as_interleaved());

        self.context.playback_sample_frame_pos.store(
            self.context.playback_sample_frame_pos.load(Ordering::SeqCst) +1,
            Ordering::SeqCst
        );

        Ok(())
    }
}

// fn create_

pub struct AudioSampleContext {
    pub playback_sample_frame_pos: Arc<AtomicU32>,
    pub buffer_status: usize,

    pub packet_playback_idx: usize,

    pub fetch_buffer_status: Arc<AtomicUsize>,
    pub process_audio_data_status: Arc<AtomicUsize>,
    pub fetch_buffer_condvar: Arc<(Mutex<bool>, Condvar)>,
    pub process_sample_condvar: Arc<(Mutex<bool>, Condvar)>,

    pub fetch_buffer_request: Arc<AtomicUsize>,
}

impl Default for AudioSampleContext {
    fn default() -> Self {
        Self { 
            playback_sample_frame_pos: Arc::new(AtomicU32::new(0)), 
            buffer_status: Default::default(), 
            packet_playback_idx: Default::default(),

            fetch_buffer_status: Arc::new(AtomicUsize::new(FetchBufferStatus::Init as usize)),
            process_audio_data_status: Arc::new(AtomicUsize::new(ProcessAudioDataStatus::Init as usize)),

            fetch_buffer_condvar: Arc::new((Mutex::new(false), Condvar::new())),
            process_sample_condvar: Arc::new((Mutex::new(false), Condvar::new())),

            fetch_buffer_request: Arc::new(AtomicUsize::new(FetchBufferRequest::None as usize)),
        }
    }
}