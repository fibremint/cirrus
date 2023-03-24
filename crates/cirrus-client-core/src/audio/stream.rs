use std::{sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}}, mem::MaybeUninit};
use crossbeam_channel::Sender;
use ringbuf::{HeapRb, SharedRb, Consumer, Producer};

use cpal::traits::{DeviceTrait, StreamTrait};
use tokio::{runtime::Handle, sync::RwLock};

use super::{sample::{AudioSample, FetchBufferSpec, ProcessAudioDataStatus}, device::AudioDeviceContext, AudioPlayerRequest};
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
    StreamStatus(StreamStatus),
    CurrentStream { length: f32 },
    ResetState,
    // StreamCreated,
}

impl std::fmt::Display for UpdatedPlaybackMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UpdatedPlaybackMessage::PositionSec(_) => write!(f, "PositionSec"),
            UpdatedPlaybackMessage::StreamStatus(_) => write!(f, "StreamStatus"),
            UpdatedPlaybackMessage::CurrentStream { length: _ } => write!(f, "CurrentStream"),
            UpdatedPlaybackMessage::ResetState => write!(f, "ResetState"),
            // UpdatedPlaybackMessage::StreamCreated => write!(f, "StreamCreated"),
        }
    }
}

#[derive(Clone, serde_derive::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedStreamMessage {
    pub(crate) stream_id: String,
    pub(crate) message_type: String,
    pub(crate) message: UpdatedPlaybackMessage,
}

pub struct StreamPlaybackContext {
    pub stream_id: String,

    sample_pos: usize,
    playback_pos_sec: u32,

    stream_status: Arc<AtomicUsize>,
    host_stream_config: Arc<cpal::StreamConfig>,

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
            // stream_status: Default::default(),
            stream_status: Arc::new(AtomicUsize::new(StreamStatus::Pause as usize)),
            host_stream_config,
            notify_update_sender,
        }
    }

    fn set_playback_sec(&mut self, sec: f64) {
        self.sample_pos = convert_sec_to_sample_pos(sec, self.host_stream_config.sample_rate.0);

        self.update_playback_sec(sec as u32);
    }

    fn increase_sample_pos(&mut self, value: usize) {
        let increased_sample_pos = self.sample_pos + value;

        self.set_sample_pos(increased_sample_pos);
    }

    fn set_sample_pos(&mut self, sample_pos: usize) {
        self.sample_pos = sample_pos;
        
        self.update_playback_sec(
            convert_sample_pos_to_sec(
                self.sample_pos,
                self.host_stream_config.sample_rate.0
            )
        )
    }

    fn update_playback_sec(&mut self, sec: u32) {
        if self.playback_pos_sec == sec {
            return;
        }

        self.playback_pos_sec = sec;
        self.notify_updated_item(UpdatedPlaybackMessage::PositionSec(sec));
    }

    fn update_stream_status(&self, stream_status: StreamStatus) {
        self.stream_status.store(stream_status as usize, Ordering::SeqCst);

        self.notify_updated_item(UpdatedPlaybackMessage::StreamStatus(
            StreamStatus::from(self.stream_status.load(Ordering::SeqCst))
        ));
    }

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

fn convert_sample_pos_to_sec(sample_pos: usize, sample_rate: u32) -> u32 {
    (sample_pos as f32 / sample_rate as f32).floor() as u32
}

fn convert_sec_to_sample_pos(sec: f64, sample_rate: u32) -> usize {
    (sec * sample_rate as f64) as usize
}

pub struct AudioStream {
    stream: cpal::Stream,
    audio_sample: AudioSample,
    stream_playback_context: Arc<RwLock<StreamPlaybackContext>>,
    audio_stream_buf_consumer: Arc<Mutex<AudioStreamBufferConsumer<f32>>>,
    // request_sender: Sender<AudioPlayerRequest>,
}

impl AudioStream {
    pub fn new(
        // stream_id: String,
        audio_tag_id: &str,
        rt_handle: &Handle,
        device_context: &AudioDeviceContext,
        // source: AudioSource,
        fetch_initial_buffer_sec: Option<u32>,
        stream_buffer_len_ms: f32,
        notify_update_sender: Option<Sender<UpdatedStreamMessage>>,
        request_sender: Sender<AudioPlayerRequest>,
    ) -> Result<Self, anyhow::Error> {
        let audio_source = rt_handle.block_on(async move {
            AudioSource::new(
                "http://localhost:50000",
                &None,
                audio_tag_id
            ).await.unwrap()
        });

        let audio_stream_buf = create_audio_stream_buffer(
            stream_buffer_len_ms,
            device_context.output_stream_config.sample_rate.0 as f32,
            device_context.output_stream_config.channels as usize
        );
        let (audio_stream_buf_producer, audio_stream_buf_consumer) = audio_stream_buf.split();
        let audio_stream_buf_consumer = Arc::new(Mutex::new(audio_stream_buf_consumer));

        let stream_playback_context = Arc::new(
            RwLock::new(
                StreamPlaybackContext::new(
                    audio_source.id.clone(),
                    device_context.output_stream_config.clone(),
                    notify_update_sender,
                )
            )
        );

        let audio_sample = AudioSample::new(
            audio_source,
            &device_context.output_stream_config,
            audio_stream_buf_producer,
            FetchBufferSpec {
                init_fetch_sec: fetch_initial_buffer_sec,
                buffer_margin_sec: 2,
                fetch_packet_sec: 5,
            }
        )?;

        audio_sample.start_process_audio_data_thread(rt_handle);

        let _stream_playback_context = stream_playback_context.clone();
        let _process_sample_condvar = audio_sample.inner.lock().unwrap().context.process_sample_condvar.clone();
        let _audio_stream_buf_consumer = audio_stream_buf_consumer.clone();
        let _process_audio_data_status = audio_sample.inner.lock().unwrap().context.process_audio_data_status.clone();
        let _request_sender = request_sender.clone();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
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
                *sample = match _audio_stream_buf_consumer.lock().unwrap().pop() {
                    Some(s) => {
                        consumed_ch_samples += 1;
                        s
                    },
                    None => {
                        0.0
                    }
                };
            }

            if consumed_ch_samples == 0 && 
                ProcessAudioDataStatus::ReactEnd == ProcessAudioDataStatus::from(
                    _process_audio_data_status.load(Ordering::SeqCst)
                ) {
                    _request_sender.send(AudioPlayerRequest::StreamReactEnd).unwrap();
                    // println!("react end");
                }

            if consumed_ch_samples > 0 {
                _stream_playback_context.blocking_write().increase_sample_pos(consumed_ch_samples / 2);
            }
        };

        let stream = device_context.device.build_output_stream(
            &device_context.output_stream_config, 
            output_data_fn, 
            err_fn
        )?;

        Ok(Self {
            stream,
            audio_sample,
            stream_playback_context,
            audio_stream_buf_consumer,
            // request_sender,
        })

    }

    // pub fn get_stream_id(&self) -> String {
    //     self.stream_playback_context.blocking_read().stream_id.clone()
    // }

    pub fn play(&self) -> Result<(), anyhow::Error> {
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

        let (process_sample_mutex, _) = &*process_sample_condvar;
        let mut process_sample_guard = process_sample_mutex.lock().unwrap();
        // set condvar of process audio sample to stop
        *process_sample_guard = false;

        self.stream_playback_context.blocking_read().update_stream_status(StreamStatus::Pause);

        Ok(())
    }

    pub fn set_playback_position(
        &self, 
        position_sec: f64
    ) -> Result<(), anyhow::Error> {
        self.pause()?;

        self.audio_stream_buf_consumer.lock().unwrap().clear();
        
        self.audio_sample.set_playback_position(position_sec)?;
        self.stream_playback_context.blocking_write().set_playback_sec(position_sec as f64);

        self.play()?;

        Ok(())
    }
}

// impl Drop for AudioStream {
//     fn drop(&mut self) {
//         self.stream_playback_context.blocking_read().notify_updated_item(
//             UpdatedPlaybackMessage::PositionSec(())
//         )
//     }
// }

type AudioStreamBuffer = SharedRb<f32, Vec<MaybeUninit<f32>>>;
pub type AudioStreamBufferProducer = Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
type AudioStreamBufferConsumer<T> = Consumer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>;

fn create_audio_stream_buffer(
    length_ms: f32,
    sample_rate: f32,
    output_channels: usize
) -> AudioStreamBuffer {
    let latency_frames = (length_ms / 1_000.0) * sample_rate as f32;
    let latency_samples = latency_frames as usize * output_channels as usize;

    HeapRb::<f32>::new(latency_samples * output_channels)
}
