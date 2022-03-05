use std::{iter::Copied, any::Any, collections::{HashMap, VecDeque}, sync::{Arc, Mutex, Weak, atomic::AtomicUsize}, borrow::{Borrow, BorrowMut}, rc::Rc, thread};
use tokio::sync::RwLock;
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

#[derive(Debug)]
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
    current_sample_frame: usize,
    // buffer_status: Arc<RwLock<AudioSampleStatus>>,
    buffer_status: Arc<AtomicUsize>,

}

impl AudioSample {
    pub fn new(source: AudioSource) -> Self {
        Self {
            source,
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
            current_sample_frame: 0,
            // buffer_status: Arc::new(RwLock::new(AudioSampleStatus::FillBuffer)),
            buffer_status: Arc::new(AtomicUsize::new(AudioSampleStatus::Init as usize)),
        }
    }

    // pub async fn run_buffer_thread(&self) {
    //     // let sample_buffer = Arc::clone(&self.sample_buffer);

    //     // thread::spawn(move || {
    //     //     loop {
    //     //         println!("run buffer thread");
    //     //     }
    //     // });

    //     loop {
    //         println!("run buffer thread");

    //         let sample_buffer = self.sample_buffer.read().await;

    //         if self.current_sample_frame < sample_buffer.len() + (self.source.metadata.sample_rate * 15) as usize {
    //             println!("fill buffer");
    //             self.get_buffer_for(30 * 1000).await.unwrap();
    //             println!("done fill buffer");
    //         } else {
    //             println!("buffer enough");
    //         }
    //         // let buf_len = sample_buffer.len();

    //         // println!("buf len: {}", buf_len);
    //     }

    //     // println!("run buffer thread");

    //     // let sample_buffer = self.sample_buffer.read().await;

    //     // if self.current_sample_frame < sample_buffer.len() + (self.source.metadata.sample_rate * 15) as usize {
    //     //     println!("fill buffer");
    //     //     self.get_buffer_for(30 * 1000).await.unwrap();
    //     //     println!("done fill buffer");
    //     // } else {
    //     //     println!("buffer enough");
    //     // }
    // }

    pub fn run_buffer_thread(self: Arc<Self>) {
        // let _self = self.clone();
        // let _self = Arc::get_mut(&mut self).unwrap();

        tokio::spawn(async move {
            loop {
                // println!("run buffer fill task");

                let sample_buffer_len = {
                    self.sample_buffer.read().await.len()
                };
    
                if self.current_sample_frame + (self.source.metadata.sample_rate * 15) as usize > sample_buffer_len  {
                    // let mut buffer_status = self.buffer_status.write().await.borrow_mut();
                    // let buffer_status = self.buffer_status.borrow_mut();
                    // buffer_status = RwLock::new(AudioSampleStatus::FillBuffer);
                    self.buffer_status.store(AudioSampleStatus::FillBuffer as usize, std::sync::atomic::Ordering::SeqCst);
                    
                    println!("fill buffer");
                    self.get_buffer_for(30 * 1000).await.unwrap();
                } else {
                    self.buffer_status.store(AudioSampleStatus::Play as usize, std::sync::atomic::Ordering::SeqCst);

                    println!("enough buffer");
                }
    
                sleep(Duration::from_millis(100)).await;
            }
        });

        // loop {
        //     let _self = self.clone();
        //     // let _self = Arc::clone(&self);

        //     tokio::task::spawn(async move {
        //         println!("run buffer fill task");

        //         let sample_buffer_len = {
        //             _self.sample_buffer.read().await.len()
        //         };

        //         if _self.current_sample_frame + (_self.source.metadata.sample_rate * 15) as usize > sample_buffer_len  {
        //             println!("fill buffer");
        //             _self.get_buffer_for(30 * 1000).await.unwrap();
        //         } else {
        //             println!("enough buffer");
        //         }

        //         sleep(Duration::from_millis(100)).await;
        //     });

        //     // tokio::join!(task());

        //     // task.
        // }
    }

    // pub fn run_buffer_thread(&self) {
    //     loop {
    //         println!("run buffer thread");

    //         tokio::spawn(async move {
    //             let sample_buffer = _self.sample_buffer.write().await;
    //             if self.current_sample_frame < sample_buffer.len() + (self.source.metadata.sample_rate * 15) as usize {
    //                 println!("fill buffer");
    //                 // _self.get_buffer_for(30 * 1000).await.unwrap();
    //             } else {
    //                 println!("buffer enough");
    //             }
    //         });
    //     } 

    // }

    // pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
    pub async fn get_buffer_for(&self, ms: u32) -> Result<(), anyhow::Error> {
        println!("get buffer fn");
        let sample_rate = self.source.metadata.sample_rate;
        let req_samples = ms * self.source.metadata.sample_rate / 1000;
        
        println!("req sample");
        let sample_res = api::get_audio_data(
            &self.source.id, 
            self.current_sample_frame as u32, self.current_sample_frame as u32 + req_samples
        ).await.unwrap();

        println!("transform sample res");
        let sample_res = sample_res.into_inner().content
            .iter()
            .map(|item| *item as f32 / sample_rate as f32)
            .collect::<Vec<f32>>();
        println!("get sample res");
        // sample_res.get_ref().content
        let mut sample_buffer = self.sample_buffer.write().await;
        println!("done sample buffer write await");

        // sample_buffer.extend(sample_res.get_ref().content);
        sample_buffer.extend(sample_res);
        println!("done extend sample buffer");

        // drop(sample_buffer);

        Ok(())
    }
    
    fn play_for(&self, output: &mut [f32]) {
        println!("run play for function");

        let buffer_status = self.buffer_status.load(std::sync::atomic::Ordering::SeqCst);
        let buffer_status = AudioSampleStatus::from(buffer_status);

        println!("buf status: {:?}", buffer_status);

        // for frame in output.chunks_mut(self.source.metadata.channels) {
        //     // let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        //     // let sample_buffer = self.sample_buffer.lock().unwrap();
        //     let sample_buffer = self.sample_buffer.write().unwrap();
        //     let sample_data = match self.sample_buffer. {
        //         Some(s) => s,
        //         None => {
        //             println!("end of sample");
        //             self.pause();
        //             &0.0
        //         }
        //     };

        //     drop(sample_buffer);

        //     for sample in frame.iter_mut() {
        //         *sample = sample_data;
        //     }
        // }
    }

    // pub fn sample_play(self: Arc<Self>, output: &mut [f32]) {
    //     // for frame in output.chunks_mut(self.source.metadata.channels) {
    //     //     // let value: T = cpal::Sample::from::<f32>(&on_sample(request));
    //     //     // let sample_buffer = self.sample_buffer.lock().unwrap();
    //     //     let sample_buffer = self.sample_buffer.write().unwrap();
    //     //     let sample_data = match self.sample_buffer. {
    //     //         Some(s) => s,
    //     //         None => {
    //     //             println!("end of sample");
    //     //             self.pause();
    //     //             &0.0
    //     //         }
    //     //     };

    //     //     drop(sample_buffer);

    //     //     for sample in frame.iter_mut() {
    //     //         *sample = sample_data;
    //     //     }
    //     // }
    // }
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
        // let metadata = AudioRequest::get_audio_source_metadata();
        // let metadata = AudioSourceMetadata::new(
        //     16,
        //     44100,
        //     2,
        //     70_000_000
        // );

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