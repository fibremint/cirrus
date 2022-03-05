use std::{iter::Copied, any::Any, collections::{HashMap, VecDeque}, sync::{Arc, Mutex, Weak, atomic::{AtomicUsize, Ordering}, RwLock}, borrow::{Borrow, BorrowMut}, rc::Rc, thread};
// std::sync::atomic::Ordering;
// use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

// use crate::request::AudioRequest;
use crate::client as api;
// use api::audio_proto::AudioMetaRes;

trait Playable {
    fn play(&self);
    fn pause(&self);
}

pub struct AudioPlayer {
    ctx: AudioContext,
    // audio_source_lists: Arc<Vec<String>>,
    // streams: HashMap<String, AudioStream>,
    streams: VecDeque<AudioStream>,
    // streams: HashMap<String, AudioStream<Box<dyn Send>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        // let audio_source_lsits = audio_source_lists.clone();
        let ctx = AudioContext::new().unwrap();

        Self {
            ctx,
            streams: VecDeque::new(),
            // streams: HashMap::new(),

        }
    }

    pub async fn add_audio(&mut self, uri: &str) -> Result<(), anyhow::Error> {
        let audio_source = AudioSource::new(uri).await.unwrap();
        let audio_stream = AudioStream::new(&self.ctx, audio_source).await?;

        // self.streams.insert(uri.to_string(), audio_stream);
        self.streams.push_back(audio_stream);
        println!("done add audio");

        Ok(())
    }

    pub fn remove_audio(&self) {

    }

    pub fn set_next(&self, uri: String) {

    }

    pub fn set_previous(&self, uri: String) {

    }

    pub fn add_stream(&self, uri: String) {
        // let stream = Arc::new(
        //     AudioStream::new(uri)
        // );

        // let stream = AudioStream::new(uri);

        // self.streams.insert(uri, stream);
        // self.streams.extend_one((uri, stream));
    }

    pub fn remove_stream(&self, target_uri: String) {

    }

    pub async fn play(&self) {
        println!("play audio");

        let current_stream = self.streams.front().unwrap();
        // let current_stream = self.streams.front().unwrap().
        current_stream.stream.play().unwrap();
        // current_stream.play();
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

enum PlayStatus {
    Play,
    Pause,
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
    // sample_buffer: VecDeque<f32>,
    sample_buffer: Arc<RwLock<VecDeque<f32>>>,
    // current_sample_frame: usize,
    current_sample_frame: Arc<AtomicUsize>,
    // buffer_status: Arc<RwLock<AudioSampleStatus>>,
    buffer_status: Arc<AtomicUsize>,
    last_buf_req_pos: Arc<AtomicUsize>,
}

impl AudioSample {
    pub fn new(source: AudioSource) -> Self {
        Self {
            source,
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
            current_sample_frame: Arc::new(AtomicUsize::new(0)),
            // current_sample_frame: 0,
            // buffer_status: Arc::new(RwLock::new(AudioSampleStatus::FillBuffer)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleStatus::Init as usize)),
            last_buf_req_pos: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn run_buffer_thread(self: Arc<Self>) {
        let buffer_margin: f32 = 20.;
        let fetch_buffer_sec: f32 = 50.;
        // let sample_frames = self.source.metadata.sample_frames;
        let content_length = self.source.metadata.sample_frames as f32 / self.source.metadata.sample_rate as f32;
        println!("content length: {}", content_length);

        tokio::spawn(async move {
            loop {
                let sample_buffer_len = self.sample_buffer.read().unwrap().len();
                let remain_sample_buffer = sample_buffer_len as f32 / self.source.metadata.sample_rate as f32 / 2.0;
                let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);
                let current_pos = (current_sample_frame as f32 / self.source.metadata.sample_rate as f32);

                println!(
                    "current pos: {:.2}s\tplayed samples: {}\tremain sample buffer: {:.2}s",
                    current_pos,
                    current_sample_frame,
                    remain_sample_buffer
                );
                // if current_sample_frame + (self.source.metadata.sample_rate * 15) as usize > sample_buffer_len  {

                if content_length - current_pos > buffer_margin {
                // if current_pos + remain_sample_buffer < content_length {
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

                // if current_sample_frame + sample_buffer_len < sample_frames {
                //     if (self.source.metadata.sample_rate * buffer_margin) as usize > sample_buffer_len {
                //         println!("fetch audio sample buffer");
                //         self.get_buffer_for(fetch_buffer_sec * 1000).await.unwrap();
                //     } else if sample_buffer_len == 0 {
                //         self.buffer_status.store(AudioSampleStatus::FillBuffer as usize, Ordering::SeqCst);
                //     } else {
                //         self.buffer_status.store(AudioSampleStatus::Play as usize, Ordering::SeqCst);
                //     }
                // } else {
                //     break;
                // }

                // if (self.source.metadata.sample_rate * buffer_margin) as usize > sample_buffer_len {
                //     println!("fetch audio sample buffer");
                //     self.get_buffer_for(fetch_buffer_sec * 1000).await.unwrap();
                // } else if sample_buffer_len == 0 {
                //     self.buffer_status.store(AudioSampleStatus::FillBuffer as usize, Ordering::SeqCst);
                // } else {
                //     self.buffer_status.store(AudioSampleStatus::Play as usize, Ordering::SeqCst);
                // }
    
                // sleep(Duration::from_millis(100)).await;
            }
        });
    }

    // pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
    pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
        let sample_rate = self.source.metadata.sample_rate;
        let req_samples = ms * self.source.metadata.sample_rate / 1000;
        
        println!("request audio data part");
        let buffer_status = AudioSampleStatus::from(self.buffer_status.load(Ordering::SeqCst));
        // let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst) as u32;
        let last_buf_req_pos = self.last_buf_req_pos.load(Ordering::SeqCst);

        let sample_res = match buffer_status {
            AudioSampleStatus::Init => {
                let resp = api::get_audio_data(
                    &self.source.id, 
                    0, 
                    req_samples).await.unwrap();

                self.last_buf_req_pos.store(req_samples as usize, Ordering::SeqCst);

                resp
            },

            AudioSampleStatus::FillBuffer | AudioSampleStatus::Play => {
                

                let resp = api::get_audio_data(
                    &self.source.id,
                    std::cmp::min(last_buf_req_pos as u32 + 1, self.source.metadata.sample_frames as u32),
                    std::cmp::min(last_buf_req_pos as u32 + req_samples, self.source.metadata.sample_frames as u32)).await.unwrap();
                // let resp = api::get_audio_data(
                //     &self.source.id, 
                //     last_buf_req_pos as u32 + 1, 
                //     last_buf_req_pos as u32 + req_samples).await.unwrap();

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
        // println!("run play for function");

        let buffer_status = self.buffer_status.load(std::sync::atomic::Ordering::SeqCst);
        let buffer_status = AudioSampleStatus::from(buffer_status);

        // println!("buf status: {:?}", buffer_status);

        // if buffer_status == AudioSampleStatus::Play {
        if buffer_status != AudioSampleStatus::Init {
            // println!("play content function");

            let buf_len = self.sample_buffer.read().unwrap().len();
            // println!("buf len: {}", buf_len);

            // let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);

            let mut sample_buffer = self.sample_buffer.write().unwrap();
            
            for frame in output.chunks_mut(2) {
                for point in 0..2 as usize {
                    // frame[point] = sample_buffer.pop_front().unwrap();
                    match sample_buffer.pop_front() {
                        Some(sample) => frame[point] = sample,
                        None => break,
                    }
                    // self.current_sample_frame.store(self.current_sample_frame.load(Ordering::SeqCst), Ordering::SeqCst);
                }

                let current_sample_frame = self.current_sample_frame.load(Ordering::SeqCst);
                self.current_sample_frame.store(current_sample_frame + 1, Ordering::SeqCst);
            }
            // for frame in output.chunks_mut(2) {
            //     if let Some(sample) = sample_buffer.pop_front() {
            //         println!("current_sample: {}, sample_frame_pos: {}", sample, current_sample_frame);
            //         let value: f32 = cpal::Sample::from::<f32>(&sample);

            //         for sample in frame.iter_mut() {
            //             *sample = value;
            //         }

            //         self.current_sample_frame.store(current_sample_frame + 1, Ordering::SeqCst);
            //     } else {
            //         println!("empty sample buffer");
            //         break;
            //     }

            // }
            // if let Some(sample) = sample_buffer.pop_front() {
            //     println!("current_sample: {}, sample_frame_pos: {}", sample, current_sample_frame);

            //     self.current_sample_frame.store(current_sample_frame + 1, Ordering::SeqCst);
            // } else {
            //     println!("empty sample buffer");
            // }

        }
    }
}

// struct StreamPlayStatus {
//     samples: usize,
//     current_sample: usize,
// }

// impl StreamPlayStatus {
//     fn new(samples: usize) -> Self {
//         Self {
//             samples,
//             current_sample: 0 as usize,
//         }
//     }
// }

struct AudioStream {
    stream: cpal::Stream,
    // source: AudioSource, 
    // sample_buffer: Arc<RwLock<VecDeque<f32>>>,
    // sample_buffer: VecDeque<f32>,
    audio_sample: Arc<AudioSample>,
    // audio_sample: Arc<RwLock<AudioSample>>,
    play_status: PlayStatus,
    // stream_play_status: StreamPlayStatus,
}

// unsafe impl Send for AudioStream {}

impl AudioStream {
    pub async fn new(ctx: &AudioContext, source: AudioSource) -> Result<Self, anyhow::Error> {
        let sample = ctx.stream_config.sample_rate.0 as f32;
        let channels = ctx.stream_config.channels as usize;
        // let audio_sample = AudioSample::new(source);
        let audio_sample = Arc::new(AudioSample::new(source));

        let sample_play_err_fn = |err: cpal::StreamError| {
            println!("an error occured on stream: {}", err);
        };

        let _audio_sample = audio_sample.clone();

        // let stream = tokio::spawn(async move {
        //     ctx.device.build_output_stream(
        //         &ctx.stream_config, 
        //         move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {

        //         }, 
        //         sample_play_err_fn,
        //     )
        // }).await.unwrap().unwrap();

        let stream = ctx.device.build_output_stream(
            &ctx.stream_config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                _audio_sample.play_for(output)
            }, 
            sample_play_err_fn
        )?;

        // _audio_sample.run_buffer_thread();

        // stream.play().unwrap();

        // audio_sample.clone().run_buffer_thread().await;

    
        // thread::spawn(move || {

        // });

        let _audio_sample2 = audio_sample.clone();
        _audio_sample2.run_buffer_thread();

        // thread::spawn(async move || {
        //     _audio_sample2.run_buffer_thread().await;
        //     // _audio_sample2.run_buffer_thread().await;
        // });

        // tokio::spawn(async {
        //     _audio_sample2.run_buffer_thread().await;
        // });

        // tokio::spawn(async move {
        //     _audio_sample2.run_buffer_thread().await
        // });

        // tokio::spawn(async move {
        //     loop {
        //         // println!("run buffer thread loop");
        //         _audio_sample2.run_buffer_thread().await;
        //     }
        // });

        // audio_sample.run_buffer_thread();

        Ok(Self {
            stream,
            audio_sample,
            play_status: PlayStatus::Pause,
        })
    }

    pub async fn play(&self) {
        self.stream.play().unwrap();
    }

    pub fn pause(&self) {
        self.stream.pause().unwrap();
    }

    // pub fn run_buffer_fill(&self) {
    //     self.audio_sample.clone().run_buffer_thread();
    // }
}

// impl Playable for AudioStream {
//     fn play(&self) {
//         self.stream.play();
//     }

//     fn pause(&self) {
//         self.stream.pause();
//     }
// }

struct AudioSource {
    id: String,
    metadata: AudioSourceMetadata,
    // source_buffer: Arc<RwLock<VecDeque<u8>>>,
}

impl AudioSource {
    async fn new(uri: &str) -> Result<Self, anyhow::Error> {
        let metadata_res = api::get_audio_meta(uri).await.unwrap().into_inner();

        let metadata = AudioSourceMetadata {
            bit_rate: metadata_res.bit_rate,
            sample_rate: metadata_res.sample_rate,
            channels: metadata_res.channels as usize,
            content_bytes: metadata_res.size,
            content_ms: 0,
            sample_frames: metadata_res.sample_frames as usize,
        };

        Ok(Self {
            id: uri.to_string(),
            metadata,
            // source_buffer: Arc::new(RwLock::new(VecDeque::new())),
        })
    }

    fn get_source_data(length: f32) {

    }
}

struct AudioSourceMetadata {
    bit_rate: u32,
    sample_rate: u32,
    channels: usize,
    content_bytes: u32,
    content_ms: u32,
    sample_frames: usize,
}

impl AudioSourceMetadata {
    fn new(bit_rate: u32, sample_rate: u32, channels: usize, content_bytes: u32, sample_frames: usize) -> Self {
        // let content_ms = 
        Self {
            bit_rate,
            sample_rate,
            channels,
            content_bytes,
            content_ms: 0,
            sample_frames,
        }
    }

    fn get_byte_address_by_ms(&self, ms: u32) -> u32 {
        return ms / self.content_ms * self.content_bytes  
    }
}