use std::{sync::{Arc, mpsc::{self}, Mutex, atomic::{AtomicUsize, Ordering}}, mem::MaybeUninit, fmt::Display};
use crossbeam_channel::Sender;
use ringbuf::{HeapRb, SharedRb, Consumer, Producer};

use cpal::traits::{DeviceTrait, StreamTrait};
use cirrus_protobuf::api::AudioDataRes;
use tokio::{runtime::Handle, sync::RwLock};
// use tokio::sync::mpsc;

use super::{sample::{AudioSample, FetchBufferSpec}, device::AudioDeviceContext};
use crate::dto::AudioSource;

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum StreamStatus {
    Play,
    Pause,
    Stop,
    BufferNotEnough,
    Error,
}

impl From<usize> for StreamStatus {
    fn from(value: usize) -> Self {
        use self::StreamStatus::*;
        match value {
            0 => Play,
            1 => Pause,
            2 => Stop,
            3 => BufferNotEnough,
            4 => Error,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, serde_derive::Serialize)]
pub enum UpdatedPlaybackMessage {
    PositionSec(u32),
    // RemainSampleBufferSec(u32),
    StreamStatus(StreamStatus),
    CurrentStream { length: f32 },
    // StreamCreated,
}

impl std::fmt::Display for UpdatedPlaybackMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // std::fmt::Debug::fmt(self, f)
        match self {
            UpdatedPlaybackMessage::PositionSec(_) => write!(f, "PositionSec"),
            UpdatedPlaybackMessage::StreamStatus(_) => write!(f, "StreamStatus"),
            UpdatedPlaybackMessage::CurrentStream { length: _ } => write!(f, "CurrentStream"),
            // UpdatedPlaybackMessage::StreamCreated => write!(f, "StreamCreated"),
        }
    }
}

#[derive(Clone, serde_derive::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedStreamMessage {
    stream_id: String,
    message_type: String,
    message: UpdatedPlaybackMessage,
    // message: T,
}

// pub fn create_atomic_status<T>()

pub struct StreamPlaybackContext {
    pub stream_id: String,
    // sample_pos: Arc<AtomicUsize>,
    sample_pos: usize,
    playback_pos_sec: u32,

    remain_audio_sample_buffer: u32,
    stream_status: Arc<AtomicUsize>,
    // pub sample_rate: u32,
    // host_stream_config: Arc<HostStreamConfig>,
    host_stream_config: Arc<cpal::StreamConfig>,

    // notify_update_sender: Option<Sender<UpdatedStreamMessage<UpdatedPlaybackMessage>>>,
    notify_update_sender: Option<Sender<UpdatedStreamMessage>>,
}

impl StreamPlaybackContext {
    pub fn new(
        stream_id: String,
        host_stream_config: Arc<cpal::StreamConfig>,
        notify_update_sender: Option<Sender<UpdatedStreamMessage>>,
    ) -> Self {

        Self {
            stream_id,
            sample_pos: Default::default(),
            playback_pos_sec: Default::default(),
            remain_audio_sample_buffer: Default::default(),
            // stream_status: Default::default(),
            stream_status: Arc::new(AtomicUsize::new(StreamStatus::Pause as usize)),
            host_stream_config,
            notify_update_sender,
        }
    }

    fn set_playback_pos_sec(&mut self, sample_pos: usize) {
        let updated_sec = convert_from_sample_pos_to_sec(
            sample_pos,
            self.host_stream_config.sample_rate.0
        );

        if self.playback_pos_sec == updated_sec {
            return;
        }

        self.playback_pos_sec = updated_sec;
        self.notify_updated_item(UpdatedPlaybackMessage::PositionSec(updated_sec));
    }

    pub fn set_sample_pos(&mut self, sample_pos: usize) {
        // self.sample_pos.store(sample_pos, Ordering::SeqCst);
        self.sample_pos = sample_pos;
        
        self.set_playback_pos_sec(sample_pos);
    }

    pub fn increase_sample_pos(&mut self, value: usize) {
        // let increated_sample_pos = self.sample_pos.load(Ordering::SeqCst) + value;
        // self.sample_pos.store(increated_sample_pos, Ordering::SeqCst);

        let increased_sample_pos = self.sample_pos + value;
        self.sample_pos = increased_sample_pos;
        
        self.set_playback_pos_sec(increased_sample_pos);
    }

    pub fn get_stream_status(&self) -> StreamStatus {
        StreamStatus::from(self.stream_status.load(Ordering::SeqCst))
    }

    pub fn update_stream_status(&self, stream_status: StreamStatus) {
        self.stream_status.store(stream_status as usize, Ordering::SeqCst);

        self.notify_updated_item(UpdatedPlaybackMessage::StreamStatus(
            self.get_stream_status()
        ));
    }

    // pub fn notify_current_stream_info(&self) {
    //     self.notify_updated_item(UpdatedPlaybackMessage::CurrentStream { 
    //         length: () 
    //     })
    // }

    fn notify_updated_item(&self, message: UpdatedPlaybackMessage) {
        if let Some(sender) = &self.notify_update_sender {
            sender.send(UpdatedStreamMessage { 
                stream_id: self.stream_id.clone(),
                message_type: message.to_string(),
                message,
            }).unwrap();
        }
    }
}

// impl<T> Notify for StreamPlaybackContext<T> {
//     // fn notify_update<T>(&self, message: T) {
//     //     if let Some(sender) = &self.notify_update_sender {
//     //         sender.send(UpdatedStreamMessage { 
//     //             stream_id: self.stream_id.clone(),
//     //             message,
//     //         }).unwrap();
//     //     }    
//     // }

//     fn get_sender(&self) -> Option<Sender<T>> {
//         self.notify_update_sender.clone()
//     }
// }

fn convert_from_sample_pos_to_sec(sample_pos: usize, sample_rate: u32) -> u32 {
    (sample_pos as f32 / sample_rate as f32).floor() as u32
}

pub struct AudioStream {
    stream: cpal::Stream,
    audio_sample: AudioSample,
    // status: usize,
    stream_playback_context: Arc<RwLock<StreamPlaybackContext>>,
}

impl AudioStream {
    pub fn new(
        stream_id: String,
        rt_handle: &Handle,
        device_context: &AudioDeviceContext,
        source: AudioSource,
        fetch_initial_buffer_sec: Option<u32>,
        stream_buffer_len_ms: f32,
        notify_update_sender: Option<Sender<UpdatedStreamMessage>>,
    ) -> Result<Self, anyhow::Error> {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let audio_stream_buf = create_audio_stream_buffer(
            stream_buffer_len_ms,
            device_context.output_stream_config.sample_rate.0 as f32,
            device_context.output_stream_config.channels as usize
        );
        let (audio_stream_buf_prod, mut audio_stream_buf_con) = audio_stream_buf.split();
        
        let stream_playback_context = Arc::new(
            RwLock::new(
                StreamPlaybackContext::new(
                    stream_id,
                    device_context.output_stream_config.clone(),
                    notify_update_sender,
                )
            )
        );

        let audio_sample = AudioSample::new(
            source,
            // audio_data_tx,
            &device_context.output_stream_config,
            audio_stream_buf_prod,
            &stream_playback_context,
            FetchBufferSpec {
                init_fetch_sec: fetch_initial_buffer_sec,
                buffer_margin_sec: 2,
                fetch_packet_sec: 5,
            }
        )?;

        // fetch intial buffer
        // if let Some(fetch_sec) = fetch_initial_buffer_sec {
        //     audio_sample.fetch_buffer(rt_handle, fetch_sec).unwrap();
        // }
        audio_sample.start_process_audio_data_thread(rt_handle);

        let _stream_playback_context = stream_playback_context.clone();
        let _process_sample_condvar = audio_sample.inner.lock().unwrap().context.process_sample_condvar.clone();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut input_buf_fell_behind = false;
            let mut consumed_ch_samples = 0;
            // Notify to audio sample processer
            {
                let (process_sample_mutex, process_sample_cv) = &*_process_sample_condvar;
                let mut process_sample_guard = process_sample_mutex.lock().unwrap();

                *process_sample_guard = true;
                process_sample_cv.notify_one();
            }

            // consume audio samples from stream buffer
            for sample in data {
                *sample = match audio_stream_buf_con.pop() {
                    Some(s) => {
                        consumed_ch_samples += 1;
                        s
                    },
                    None => {
                        input_buf_fell_behind = true;
                        0.0
                    }
                };
            }

            // _stream_playback_context.increase_sample_pos(consumed_ch_samples / 2);
            if consumed_ch_samples > 0 {
                _stream_playback_context.blocking_write().increase_sample_pos(consumed_ch_samples / 2);
            }

            // _sample_pos.store(_sample_pos.load(Ordering::SeqCst) + consumed_samples, Ordering::SeqCst);

            if input_buf_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");

                // set status buffer not enough
                // let s = _stream_playback_context.blocking_read().update_stream_status();
            } 
            
            // else {
            //     let (process_sample_mutex, process_sample_cv) = &*_process_sample_condvar;
            //     let mut process_sample_guard = process_sample_mutex.lock().unwrap();

            //     *process_sample_guard = true;
            //     process_sample_cv.notify_one();
            // }
        };

        let stream = device_context.device.build_output_stream(
            &device_context.output_stream_config, 
            output_data_fn, 
            err_fn
        )?;

        // stream_playback_context.blocking_read().notify_updated_item(UpdatedPlaybackMessage::StreamCreated);

        Ok(Self {
            stream,
            audio_sample,
            // status: 0,
            stream_playback_context,
        })

    }

    pub fn play(&mut self) -> Result<(), anyhow::Error> {
        self.stream.play()?;
        self.stream_playback_context.blocking_read().notify_updated_item(
            UpdatedPlaybackMessage::CurrentStream { 
                length: self.audio_sample.inner.lock().unwrap().source.length as f32 
            }
        );
        self.stream_playback_context.blocking_read().update_stream_status(StreamStatus::Play);

        Ok(())
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        self.stream.pause()?;

        let process_sample_condvar = self.audio_sample.inner.lock().unwrap().context.process_sample_condvar.clone();
        // let process_sample_condvar = self.audio_sample.context.process_sample_condvar.clone();
        let (process_sample_mutex, process_sample_cv) = &*process_sample_condvar;
        let mut process_sample_guard = process_sample_mutex.lock().unwrap();
        // set condvar of process audio sample to stop
        *process_sample_guard = false;
        // // notify 
        // process_sample_cv.notify_one();
        self.stream_playback_context.blocking_read().update_stream_status(StreamStatus::Pause);


        Ok(())
    }

    // pub fn set_playback_position(&self) {
    //     let prev_playback_status = self.stream_playback_context.get_stream_status();

    //     self.pause();
    //     self.audio_sample.set_playback_position();

    //     if prev_playback_status == StreamStatus::Play {
    //         self.play();
    //     }
    // }

    pub fn update_audio_stream_buffer(&mut self) {
        todo!()
    }
}

type AudioStreamBuffer = SharedRb<f32, Vec<MaybeUninit<f32>>>;
pub type AudioStreamBufferProducer = Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
// type AudioStreamBufferConsumer<T> = Consumer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>;

fn create_audio_stream_buffer(
    length_ms: f32,
    sample_rate: f32,
    output_channels: usize
) -> AudioStreamBuffer {
    let latency_frames = (length_ms / 1_000.0) * sample_rate as f32;
    let latency_samples = latency_frames as usize * output_channels as usize;

    HeapRb::<f32>::new(latency_samples * output_channels)
}

// #[derive(Debug, PartialEq)]
// pub enum StreamPlaybackStatus {

// }

// fn increase_audio_sample_pos(as_pos: Arc<AtomicUsize>, value: usize) {
    
// }

// fn output_data_fn_test(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    
// }

// fn audio_stream_pipeline(
//     output: &mut [f32],
//     // audio_sample_tx: mpsc::Sender<&'static str>,
//     // audio_sample_tx: SyncSender<&'static str>,
//     // audio_sample_tx: Arc<Mutex<Sender<&'static str>>>,
//     consumer: Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
// ) {

// }

// fn get_packet(audio_sample_tx: mpsc::Sender<&'static str>) -> AudioDataRes {
//     todo!()
// }