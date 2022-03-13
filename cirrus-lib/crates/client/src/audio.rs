use std::{
    collections::VecDeque,
    sync::{
        Arc, 
        atomic::{AtomicUsize, Ordering},
        RwLock, Mutex
    }, rc::Rc
};

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use tokio::time::{sleep, Duration};

use crate::request;

pub struct AudioPlayer {
    ctx: AudioContext,
    streams: VecDeque<AudioStream>,
    // streams: Arc<Mutex<VecDeque<AudioStream>>>,
    // streams: VecDeque<Arc<RwLock<AudioStream>>>,
    // streams: Arc<RwLock<VecDeque<AudioStream>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let ctx = AudioContext::new().unwrap();

        Self {
            ctx,
            streams: VecDeque::new(),
        }
    }

    pub async fn add_audio(&mut self, uri: &str) -> Result<(), anyhow::Error> {
        let audio_source = AudioSource::new(uri).await.unwrap();
        let audio_stream = AudioStream::new(&self.ctx, audio_source).await?;

        self.streams.push_back(audio_stream);
        println!("done add audio");

        Ok(())
    }

    pub fn remove_audio(&self) {
        todo!()
    }

    pub fn play(&self) {
        println!("play audio");

        let current_stream = self.streams.front().unwrap();
        current_stream.stream.play().unwrap();
    }

    pub fn pause(&self) {
        println!("pause audio");

        let current_stream = self.streams.front().unwrap();
        current_stream.stream.pause().unwrap();
    }
}

struct AudioContext {
    // host: cpal::Host,
    device: cpal::Device,
    stream_config: cpal::StreamConfig,
}

impl AudioContext {
    fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
    
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
        
        println!("Output device: {}", device.name()?);
    
        let config = device.default_output_config()?.into();
        println!("Default output config: {:?}", config);

        Ok(Self {
            // host,
            device,
            stream_config: config,
        })
    }
}

#[derive(Debug, PartialEq)]
enum AudioSampleStatus {
    Init,
    FillBuffer,
    Play,
}

impl From<usize> for AudioSampleStatus {
    //ref: https://gist.github.com/polypus74/eabc7bb00873e6b90abe230f9e632989
    fn from(value: usize) -> Self {
        use self::AudioSampleStatus::*;
        match value {
            0 => Init,
            1 => FillBuffer,
            2 => Play,
            _ => unreachable!(),
        }
    }
}

struct AudioSample {
    source: AudioSource,
    sample_buffer: Arc<RwLock<VecDeque<f32>>>,
    current_sample_frame: Arc<AtomicUsize>,
    buffer_status: Arc<AtomicUsize>,
    last_buf_req_pos: Arc<AtomicUsize>,
}

// unsafe impl Send for AudioSample {}

impl AudioSample {
    pub fn new(source: AudioSource) -> Self {
        Self {
            source,
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
            current_sample_frame: Arc::new(AtomicUsize::new(0)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleStatus::Init as usize)),
            last_buf_req_pos: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn run_buffer_thread(self: Arc<Self>) {
        let buffer_margin: f32 = 20.;
        let fetch_buffer_sec: f32 = 50.;
        let content_length = self.source.metadata.sample_frames as f32 / self.source.metadata.sample_rate as f32;
        println!("content length: {}", content_length);

        tokio::spawn(async move {
            loop {
                let sample_buffer_len = self.sample_buffer.read().unwrap().len();
                let remain_sample_buffer = sample_buffer_len as f32 / self.source.metadata.sample_rate as f32 / 2.0;
                let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);
                let current_pos = current_sample_frame as f32 / self.source.metadata.sample_rate as f32;

                println!(
                    "current pos: {:.2}s\tplayed samples: {}\tremain sample buffer: {:.2}s",
                    current_pos,
                    current_sample_frame,
                    remain_sample_buffer
                );

                if content_length - current_pos > buffer_margin {
                    if buffer_margin > remain_sample_buffer {
                        println!("fetch audio sample buffer");
                        self.get_buffer_for(fetch_buffer_sec as u32 * 1000).await.unwrap();
                    } else if sample_buffer_len == 0 {
                        self.buffer_status.store(AudioSampleStatus::FillBuffer as usize, Ordering::SeqCst);
                    } else {
                        self.buffer_status.store(AudioSampleStatus::Play as usize, Ordering::SeqCst);
                    }

                    sleep(Duration::from_millis(100)).await;

                } else {
                    break;
                }
            }
        });
    }

    pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
        let sample_rate = self.source.metadata.sample_rate;
        let req_samples = ms * self.source.metadata.sample_rate / 1000;
        
        println!("request audio data part");
        let buffer_status = AudioSampleStatus::from(self.buffer_status.load(Ordering::SeqCst));
        let last_buf_req_pos = self.last_buf_req_pos.load(Ordering::SeqCst);

        let sample_res = match buffer_status {
            AudioSampleStatus::Init => {
                let resp = request::get_audio_data(
                    &self.source.id, 
                    0, 
                    req_samples).await.unwrap();

                self.last_buf_req_pos.store(req_samples as usize, Ordering::SeqCst);

                resp
            },

            AudioSampleStatus::FillBuffer | AudioSampleStatus::Play => {
                let resp = request::get_audio_data(
                    &self.source.id,
                    std::cmp::min(last_buf_req_pos as u32 + 1, self.source.metadata.sample_frames as u32),
                    std::cmp::min(last_buf_req_pos as u32 + req_samples, self.source.metadata.sample_frames as u32)).await.unwrap();

                self.last_buf_req_pos.store(req_samples as usize + last_buf_req_pos, Ordering::SeqCst);

                resp
            },
        };

        println!("parse audio data response as audio sample");
        // ref: https://users.rust-lang.org/t/convert-slice-u8-to-u8-4/63836
        let sample_res = sample_res.into_inner().content
            .chunks(2)
            .map(|chunks| i16::from_be_bytes(chunks.try_into().unwrap()) as f32 / sample_rate as f32)
            .collect::<Vec<f32>>();

        let mut sample_buffer = self.sample_buffer.write().unwrap();

        sample_buffer.extend(sample_res);
        println!("done fetch audio sample buffer");

        Ok(())
    }
    
    fn play_for(&self, output: &mut [f32]) {
        let buffer_status = self.buffer_status.load(std::sync::atomic::Ordering::SeqCst);
        let buffer_status = AudioSampleStatus::from(buffer_status);

        if buffer_status != AudioSampleStatus::Init {
            let mut sample_buffer = self.sample_buffer.write().unwrap();
            
            for frame in output.chunks_mut(2) {
                for point in 0..2 as usize {
                    match sample_buffer.pop_front() {
                        Some(sample) => frame[point] = sample,
                        None => break,
                    }
                }

                let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);
                self.current_sample_frame.store(current_sample_frame + 1, Ordering::SeqCst);
            }
        }
    }
}
struct AudioStream {
    // stream: Arc<RwLock<cpal::Stream>>,
    // stream: Arc<Mutex<cpal::Stream>>,
    // stream: &cpal::Stream,
    // stream: Rc<cpal::Stream>,
    // stream: Box<cpal::Stream>,
    stream: cpal::Stream,
    audio_sample: Arc<AudioSample>,
}

unsafe impl Send for AudioStream {}
unsafe impl Sync for AudioStream {}

impl AudioStream {
    pub async fn new(ctx: &AudioContext, source: AudioSource) -> Result<Self, anyhow::Error> {
        // let sample = ctx.stream_config.sample_rate.0 as f32;
        // let channels = ctx.stream_config.channels as usize;
        let audio_sample = Arc::new(AudioSample::new(source));

        let sample_play_err_fn = |err: cpal::StreamError| {
            println!("an error occured on stream: {}", err);
        };

        let _audio_sample = audio_sample.clone();

        let stream = ctx.device.build_output_stream(
            &ctx.stream_config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                _audio_sample.play_for(output)
            }, 
            sample_play_err_fn
        )?;

        let _audio_sample2 = audio_sample.clone();
        _audio_sample2.run_buffer_thread();

        Ok(Self {
            stream,
            audio_sample,
        })
    }
}

struct AudioSource {
    id: String,
    metadata: AudioSourceMetadata,
}

impl AudioSource {
    async fn new(uri: &str) -> Result<Self, anyhow::Error> {
        let metadata_res = request::get_audio_meta(uri).await.unwrap().into_inner();

        let metadata = AudioSourceMetadata {
            bit_rate: metadata_res.bit_rate,
            sample_rate: metadata_res.sample_rate,
            channels: metadata_res.channels as usize,
            content_bytes: metadata_res.size,
            sample_frames: metadata_res.sample_frames as usize,
        };

        Ok(Self {
            id: uri.to_string(),
            metadata,
        })
    }
}

struct AudioSourceMetadata {
    bit_rate: u32,
    sample_rate: u32,
    channels: usize,
    content_bytes: u32,
    sample_frames: usize,
}

impl AudioSourceMetadata {
    fn new(bit_rate: u32, sample_rate: u32, channels: usize, content_bytes: u32, sample_frames: usize) -> Self {
        Self {
            bit_rate,
            sample_rate,
            channels,
            content_bytes,
            sample_frames,
        }
    }
}