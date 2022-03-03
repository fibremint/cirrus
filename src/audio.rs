use std::{iter::Copied, any::Any, collections::{HashMap, VecDeque}, sync::{Arc, Mutex, Weak, RwLock}, borrow::Borrow, rc::Rc};

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use crate::request::AudioRequest;

trait Playable {
    fn play(&self);
    fn pause(&self);
}

pub struct AudioPlayer {
    ctx: AudioContext,
    // audio_source_lists: Arc<Vec<String>>,
    streams: HashMap<String, AudioStream>,
    // streams: HashMap<String, AudioStream<Box<dyn Send>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        // let audio_source_lsits = audio_source_lists.clone();
        let ctx = AudioContext::new().unwrap();

        Self {
            ctx,
            streams: HashMap::new(),
        }
    }

    pub fn add_audio(&mut self, uri: String) -> Result<(), anyhow::Error> {
        let audio_source = AudioSource::new(uri.clone()).unwrap();
        let audio_stream = AudioStream::new(&self.ctx, audio_source)?;

        self.streams.insert(uri.clone(), audio_stream);

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

    pub fn play(&self) {
        println!("play audio");
    }

    pub fn pause(&self) {
        println!("pause audio");
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

struct AudioSample {
    source: AudioSource,
    // sample_buffer: VecDeque<f32>,
    sample_buffer: Arc<RwLock<VecDeque<f32>>>,

}

impl AudioSample {
    pub fn new(source: AudioSource) -> Self {
        Self {
            source,
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    fn play_for(&self, output: &mut [f32]) {
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

struct AudioStream {
    stream: cpal::Stream,
    // source: AudioSource, 
    // sample_buffer: Arc<RwLock<VecDeque<f32>>>,
    // sample_buffer: VecDeque<f32>,
    audio_sample: Arc<AudioSample>,
    // audio_sample: Arc<RwLock<AudioSample>>,
    play_status: PlayStatus,
}

// unsafe impl Send for AudioStream {}

impl AudioStream {
    pub fn new(ctx: &AudioContext, source: AudioSource) -> Result<Self, anyhow::Error> {
        let sample = ctx.stream_config.sample_rate.0 as f32;
        let channels = ctx.stream_config.channels as usize;
        // let audio_sample = AudioSample::new(source);
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

        Ok(Self {
            stream,
            audio_sample,
            play_status: PlayStatus::Pause,
        })
    }
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
    fn new(uri: String) -> Result<Self, anyhow::Error> {
        // let metadata = AudioRequest::get_audio_source_metadata();
        let metadata = AudioSourceMetadata::new(
            16,
            44100,
            2,
            70_000_000
        );

        Ok(Self {
            id: String::from("test-source"),
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
}

impl AudioSourceMetadata {
    fn new(bit_rate: u32, sample_rate: u32, channels: usize, content_bytes: u32) -> Self {
        // let content_ms = 
        Self {
            bit_rate,
            sample_rate,
            channels,
            content_bytes,
            content_ms: 0,
        }
    }

    fn get_byte_address_by_ms(&self, ms: u32) -> u32 {
        return ms / self.content_ms * self.content_bytes  
    }
}