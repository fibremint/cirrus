use std::{
    collections::VecDeque,
    sync::{
        Arc, 
        atomic::{AtomicUsize, Ordering, AtomicBool},
        mpsc,
    },
    thread,
};

use anyhow::anyhow;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use tokio::{
    time::{sleep, Duration}, 
    sync::Mutex,
};

use crate::audio_player::state::{AudioSampleStatus, PlaybackStatus};
use crate::dto::AudioSource;

use super::sample::AudioSample;

pub struct AudioPlayer {
    inner: Arc<Mutex<AudioPlayerInner>>,
    thread_run_states: Vec<Arc<AtomicBool>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let inner = Arc::new(
            Mutex::new(
                AudioPlayerInner::new(tx.clone())
            )
        );

        let mut thread_run_states: Vec<Arc<AtomicBool>> = Vec::new();

        let _inner_1 = inner.clone();

        // ref: https://www.reddit.com/r/rust/comments/nwbtsz/help_understanding_how_to_start_and_stop_threads/
        let thread_run_state_1 = Arc::new(AtomicBool::new(true));
        let thread_run_state_1_clone = thread_run_state_1.clone();
        thread::spawn(move || loop {
            println!("start receiver");
            if !thread_run_state_1_clone.load(Ordering::Relaxed) {
                println!("terminate receiver");
                break;
            }

            // while let Some(data) = rx.blocking_recv() {
            while let Ok(data) = rx.recv() {
                println!("received message: {:?}", data);
                match data {
                    "stop" => _inner_1.blocking_lock().remove_audio(),
                    _ => (),
                }
            }
        });
        
        thread_run_states.push(thread_run_state_1);

        Self {
            inner,
            thread_run_states
        }
    }

    pub async fn add_audio(&self, audio_tag_id: &str) -> Result<f32, anyhow::Error> {
        self.inner.lock().await.add_audio(audio_tag_id).await
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        self.inner.blocking_lock().play()
    }

    pub fn stop(&self) {
        self.inner.blocking_lock().remove_audio()
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        self.inner.blocking_lock().pause()
    }

    pub fn get_playback_position(&self) -> f32 {
        self.inner.blocking_lock().get_playback_position()
    }

    pub fn set_playback_position(&self, position_sec: f32) -> Result<(), anyhow::Error> {
        self.inner.blocking_lock().set_playback_position(position_sec)
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f32 {
        self.inner.blocking_lock().get_remain_sample_buffer_sec()
    }

    pub fn get_status(&self) -> PlaybackStatus {
        self.inner.blocking_lock().get_status()
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        for thread_run_state in &self.thread_run_states {
            thread_run_state.store(false, Ordering::Relaxed);
        }
    }
}

pub struct AudioPlayerInner {
    ctx: Arc<AudioContext>,
    streams: VecDeque<AudioStream>,
    tx: mpsc::Sender<&'static str>,
    status: Arc<AtomicUsize>
}

impl AudioPlayerInner {
    pub fn new(
        tx: mpsc::Sender<&'static str>,
    ) -> Self {
        let ctx = AudioContext::new().unwrap();
        let ctx = Arc::new(ctx);

        Self {
            ctx,
            streams: VecDeque::new(),
            tx,
            status: Arc::new(AtomicUsize::new(PlaybackStatus::Stop as usize))
        }
    }

    pub async fn add_audio(&mut self, audio_tag_id: &str) -> Result<f32, anyhow::Error> {
        let audio_source = AudioSource::new(audio_tag_id).await?;

        let audio_ctx_1_clone = self.ctx.clone();
        let tx_1_clone = self.tx.clone();
        let status_1_clone = self.status.clone();
        let audio_stream = tokio::task::spawn_blocking(move || {
            AudioStream::new(
                audio_ctx_1_clone,
                audio_source,
                tx_1_clone,
                status_1_clone
            )
        }).await?.unwrap();

        println!("done add audio stream");

        let content_length = audio_stream.inner.lock().await.get_content_length();
        println!("content length: {}", content_length);

        self.streams.push_back(audio_stream);
        println!("done add audio");

        Ok(content_length)
    }

    pub fn remove_audio(&mut self) {
        // self.streams.remove(0).unwrap();
        match self.streams.remove(0) {
            Some(_) => self.status.store(PlaybackStatus::Stop as usize, Ordering::SeqCst),
            None => (),
        }
    }

    pub fn play(&mut self) -> Result<(), anyhow::Error> {
        println!("play audio");

        let current_stream = self.streams.front().unwrap();

        match current_stream.play() {
            Ok(_) => self.status.store(PlaybackStatus::Play as usize, Ordering::SeqCst),
            Err(e) => {
                self.status.store(PlaybackStatus::Error as usize, Ordering::SeqCst);

                return Err(anyhow!(e))
            }
        }

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), anyhow::Error> {
        println!("pause audio");

        let current_stream = self.streams.front().unwrap();
        // current_stream.pause()?;
        match current_stream.pause() {
            Ok(_) => self.status.store(PlaybackStatus::Pause as usize, Ordering::SeqCst),
            Err(e) => {
                self.status.store(PlaybackStatus::Error as usize, Ordering::SeqCst);

                return Err(anyhow!(e))
            }
        }

        Ok(())
    }

    pub fn get_playback_position(&self) -> f32 {
        match self.streams.front() {
            Some(stream) => return stream.inner.blocking_lock().audio_sample.get_current_playback_position_sec(),
            None => return 0.0,
        }
    }

    pub fn get_remain_sample_buffer_sec(&self) -> f32 {
        match self.streams.front() {
            Some(stream) => return stream.inner.blocking_lock().audio_sample.get_remain_sample_buffer_sec(),
            None => return 0.0,
        }
    }

    pub fn set_playback_position(&self, position_sec: f32) -> Result<(), anyhow::Error> {
        match self.streams.front() {
            Some(stream) => stream.inner.blocking_lock().set_playback_position(position_sec)?,
            None => todo!(),
        }

        Ok(())
    }

    pub fn get_status(&self) -> PlaybackStatus {
        PlaybackStatus::from(self.status.load(Ordering::Relaxed))
    }
}

struct AudioContext {
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
    
        let config: cpal::StreamConfig = device.default_output_config()?.into();

        println!("Output stream properties: sample_rate: {}, channel(s): {}", 
                 config.sample_rate.0, config.channels);

        Ok(Self {
            device,
            stream_config: config,
        })
    }
}

struct AudioStream {
    inner: Arc<Mutex<AudioStreamInner>>,
    thread_run_states: Vec<Arc<AtomicBool>>,
}

impl AudioStream {
    pub fn new(
        ctx: Arc<AudioContext>, 
        source: AudioSource, 
        tx: mpsc::Sender<&'static str>,
        audio_player_status: Arc<AtomicUsize>
    ) -> Result<Self, anyhow::Error> {
        let source_id = source.id.clone();
        let inner = AudioStreamInner::new(
            ctx, 
            source, 
            tx,
            audio_player_status
        )?;
        let inner = Arc::new(Mutex::new(inner));

        let mut thread_run_states: Vec<Arc<AtomicBool>> = Vec::new();

        let inner_guard = inner.blocking_lock();

        let audio_sample_1_clone = inner_guard.audio_sample.clone();
        let thread_run_state_1 = Arc::new(AtomicBool::new(true));
        let thread_run_state_1_clone = thread_run_state_1.clone();
        let source_id_1 = source_id.clone();
        tokio::spawn(async move {
            loop {
                if !thread_run_state_1_clone.load(Ordering::Relaxed) {
                    println!("stop thread: fetch buffer, source id: {}", source_id_1);
                    break;
                }

                if let Err(e) = audio_sample_1_clone.fetch_buffer(180., 10.).await {
                // if let Err(e) = audio_sample_1_clone.fetch_buffer(20., 50.).await {
                    println!("failed to fetch buffer: {}", e);
                }

                sleep(Duration::from_millis(500)).await;
            }
        });
        thread_run_states.push(thread_run_state_1);

        let inner_1_clone = inner.clone();
        let thread_run_state_2 = Arc::new(AtomicBool::new(true));
        let thread_run_state_2_clone = thread_run_state_2.clone();
        let source_id_2 = source_id.clone();
        thread::spawn(move || loop {
            if !thread_run_state_2_clone.load(Ordering::Relaxed) {
                println!("stop thread: audio stream playback management, source id: {}", source_id_2);
                break;
            }

            let mut inner_guard = inner_1_clone.blocking_lock();
            match inner_guard.update_stream_playback() {
                Ok("stop") => thread_run_state_2_clone.store(false, Ordering::Relaxed),
                Ok(&_) => (),
                Err(e) => println!("error at manage playback: {}, source id: {}", e, source_id_2),
            }

            drop(inner_guard);

            thread::sleep(Duration::from_millis(10));
        });
        thread_run_states.push(thread_run_state_2);

        drop(inner_guard);

        Ok(Self {
            inner,
            thread_run_states
        })
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        self.inner.blocking_lock().set_stream_playback(PlaybackStatus::Play)
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        self.inner.blocking_lock().set_stream_playback(PlaybackStatus::Pause)
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        for thread_run_state in &self.thread_run_states {
            thread_run_state.store(false, Ordering::Relaxed);
        }
    }
}

struct AudioStreamInner {
    stream: cpal::Stream,
    audio_sample: Arc<AudioSample>,
    tx: mpsc::Sender<&'static str>,
    audio_player_status: Arc<AtomicUsize>,
    stream_playback_status: PlaybackStatus
}

unsafe impl Send for AudioStreamInner {}
unsafe impl Sync for AudioStreamInner {}

impl AudioStreamInner {
    pub fn new(
        ctx: Arc<AudioContext>, 
        source: AudioSource, 
        tx: mpsc::Sender<&'static str>,
        audio_player_status: Arc<AtomicUsize>,
    ) -> Result<Self, anyhow::Error> {
        let host_output_sample_rate = ctx.stream_config.sample_rate.0;
        let host_output_channels = ctx.stream_config.channels;

        let audio_sample = Arc::new(
            AudioSample::new(
                source,
                host_output_sample_rate,
                host_output_channels as usize
            )
        );

        let sample_play_err_fn = |err: cpal::StreamError| {
            println!("an error occured on stream: {}", err);
        };

        let audio_sample_1_clone = audio_sample.clone();

        let stream = ctx.device.build_output_stream(
            &ctx.stream_config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                audio_sample_1_clone.play_for(output)
            }, 
            sample_play_err_fn
        )?;
        stream.pause().unwrap();

        let stream_playback_status = PlaybackStatus::Pause;

        let audio_stream = Self {
            stream,
            audio_sample,
            tx,
            audio_player_status,
            stream_playback_status
        };

        Ok(audio_stream)
    }

    pub fn get_content_length(&self) -> f32 {
        self.audio_sample.content_length
    }

    pub fn set_playback_position(&mut self, position_sec: f32) -> Result<(), anyhow::Error> {
        println!("set stream playback position: {}", position_sec);

        self.audio_sample.buffer_status.store(AudioSampleStatus::StopFillBuffer as usize, Ordering::Relaxed);
        self.audio_player_status.store(PlaybackStatus::Pause as usize, Ordering::Relaxed);
        self.set_stream_playback(PlaybackStatus::Pause)?;

        let position_sample_idx = self.audio_sample.get_sample_idx_from_sec(position_sec);
        let drain_buffer_len = {
            if position_sec - self.audio_sample.get_current_playback_position_sec() > 0.0 {
                position_sample_idx - self.audio_sample.get_current_sample_idx()
            } else {
                self.audio_sample.get_remain_sample_buffer_len()
            }
        };

        self.audio_sample.drain_sample_buffer(drain_buffer_len);
        self.audio_sample.set_current_sample_frame_idx(position_sample_idx);

        self.audio_player_status.store(PlaybackStatus::Play as usize, Ordering::Relaxed);
        self.set_stream_playback(PlaybackStatus::Play)?;

        Ok(())
    }

    pub fn set_stream_playback(&mut self, status: PlaybackStatus) -> Result<(), anyhow::Error> {
        match status {
            PlaybackStatus::Play => self.stream.play()?,
            PlaybackStatus::Pause => self.stream.pause()?,
            PlaybackStatus::Stop => self.stream.pause()?,
            PlaybackStatus::Error => self.stream.pause()?,
        }

        self.stream_playback_status = status;
        println!("set audio stream status: {:?}", status);

        Ok(())
    }

    fn update_stream_playback(&mut self) -> Result<&'static str, anyhow::Error> {
        match PlaybackStatus::from(self.audio_player_status.load(Ordering::SeqCst)) {
            PlaybackStatus::Play => {
                // reach end of the content
                if self.audio_sample.content_length - self.audio_sample.get_current_playback_position_sec() <= 0.5 {
                    println!("reach end of content");
                    self.set_stream_playback(PlaybackStatus::Pause)?;

                    self.tx.send("stop").unwrap();

                    return Ok("stop");
                }
                
                if self.audio_sample.get_remain_sample_buffer_sec() < 0.01 && 
                    self.stream_playback_status == PlaybackStatus::Play {
                    // remain buffer is insufficient

                    println!("remain buffer is insufficient for playing audio, buf: {}", self.audio_sample.get_remain_sample_buffer_sec());

                    self.set_stream_playback(PlaybackStatus::Pause)?;
                } else if self.audio_sample.get_remain_sample_buffer_sec() > 0.01 && 
                    self.stream_playback_status == PlaybackStatus::Pause {
                    // remain buffer is enough for playing (check sufficient of a buffer at above code)
                    // and play stream if is paused

                    println!("remain buffer is enough for playing audio, buf: {}", self.audio_sample.get_remain_sample_buffer_sec());

                    self.set_stream_playback(PlaybackStatus::Play)?;
                }
            },
            PlaybackStatus::Pause => {
                if self.stream_playback_status == PlaybackStatus::Play {

                    self.set_stream_playback(PlaybackStatus::Pause)?;
                }
            },
            _ => (),
        }

        Ok("")
    }
}

impl Drop for AudioStreamInner {
    fn drop(&mut self) {
        self.audio_sample.buffer_status.store(AudioSampleStatus::StopFillBuffer as usize, Ordering::Relaxed);
        self.set_stream_playback(PlaybackStatus::Stop).unwrap();
    }
}
