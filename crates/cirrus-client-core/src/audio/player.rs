use std::{
    collections::{VecDeque, HashMap},
    sync::{
        Arc, 
        atomic::{AtomicUsize, Ordering, AtomicBool}, mpsc,
    },
    thread, path::PathBuf,
};

use anyhow::anyhow;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use crossbeam_channel::{Sender, Receiver};
use tokio::{
    time::Duration, 
    sync::{RwLock}, runtime::Handle,
};
use tonic::transport::ClientTlsConfig;

use crate::{audio_player::state::PlaybackStatus, tls};
use crate::dto::AudioSource;

use crate::audio::{context::AudioContext, stream::AudioStream};

// use super::sample::AudioSample;

#[derive(Clone)]
pub struct ServerState {
    pub grpc_endpoint: String,
    pub tls_config: Option<ClientTlsConfig>,
}

#[derive(Debug)]
pub struct PlayerStatus {
    status: usize,
    pos: usize,
    remain_buf: f32,
}

#[derive(Debug)]
pub struct AudioMeta {
    content_length: f64,
}

pub enum AudioPlayerMessage {
    ResponsePlayerStatus(PlayerStatus),
    ResponseAudioMeta(AudioMeta),
}

pub struct AudioPlayer {
    inner: AudioPlayerInner,
    message_senders: HashMap<String, Sender<AudioPlayerMessage>>,
}

impl AudioPlayer {
    pub fn new(
        grpc_endpoint: &str,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            inner: AudioPlayerInner::new(grpc_endpoint)?,
            message_senders: HashMap::default(),
        })
    }

    pub fn enroll_mesage_sender(
        &mut self,
        name: String,
        message_sender: Sender<AudioPlayerMessage>,
    ) {
        self.message_senders.insert(name, message_sender);
    }

    pub fn start_command_handler(
        &mut self,
        command_rx: Receiver<String>,
    ) {
        loop {
            while let Ok(value) = command_rx.try_recv() {
                println!("cmd value: {}", value);
                self.dispatch_message(&value);
            }
        }
    }

    pub fn dispatch_message(&mut self, message: &str) {
        match message {
            // TODO: match method name
            "load_audio" => {
                let sender = self.message_senders.get(message).unwrap();
                let content_length = self.inner.add_audio().unwrap();

                sender.send(AudioPlayerMessage::ResponseAudioMeta(
                    AudioMeta { content_length }
                )).unwrap();
            },

            _ => println!("got message: {}", message),
        }
    }
}

pub struct AudioPlayerInner {
    ctx: AudioContext,
    streams: VecDeque<AudioStream>,
    status: usize,
}

impl AudioPlayerInner {
    pub fn new(
        grpc_endpoint: &str,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            ctx: AudioContext::new()?,
            streams: VecDeque::default(),
            status: 0,
        })
    }

    pub fn add_audio(&mut self) -> Result<f64, anyhow::Error> {
        Ok(120.)
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        todo!()
    }

    pub fn stop(&self) {

    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        todo!()

    }

    pub fn get_playback_position(&self) -> f64 {
        todo!()

    }

    pub fn set_playback_position(&self, position_sec: f64) -> Result<(), anyhow::Error> {
        todo!()

    }

    pub fn get_remain_sample_buffer_sec(&self) -> f64 {
        todo!()

    }

    pub fn get_status(&self) -> PlaybackStatus {
        todo!()

    }
}