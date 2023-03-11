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
    ctx: AudioContext,
    streams: VecDeque<AudioStream>,
    status: usize,
    // pub command_tx: Option<mpsc::Sender<String>>,
    // command_rx: mpsc::Receiver<String>,

    // message_tx: Option<mpsc::Sender<AudioPlayerMessage>>,

    message_senders: HashMap<String, Sender<AudioPlayerMessage>>,
}

impl AudioPlayer {
    pub fn new(
        grpc_endpoint: &str,
        // message_tx: Option<mpsc::Sender<AudioPlayerMessage>>,
    ) -> Result<Self, anyhow::Error> {
        // let (command_tx, mut command_rx) = mpsc::channel();

        // thread::spawn(move || {
        //     loop {
        //         let msg = command_rx.recv().unwrap();
        //     }
        // });

        println!("create audio player core");

        Ok(Self {
            ctx: AudioContext::new()?,
            streams: VecDeque::default(),
            status: 0,
            // command_tx: None,
            // command_rx,
            // message_tx,
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
        // command_rx: mpsc::Receiver<String>,
        command_rx: Receiver<String>,
    ) {
        // let (command_tx, command_rx) = mpsc::channel::<String>();

        // self.command_tx = Some(command_tx);
        // let _self = Arc::new(self);
        // _self.foo();

        // let handle = thread::spawn(move || {
        //     loop {
        //         let msg = command_rx.recv().unwrap();
        //         ;
        //     }
        // });
        // loop {
        //     let msg = command_rx.recv().unwrap();
        //     self.dispatch_message(msg);
        // }

        loop {
            while let Ok(value) = command_rx.try_recv() {
                println!("cmd value: {}", value);
                self.dispatch_message(&value);
            }
        }
    }

    pub fn dispatch_message(&self, message: &str) {
        match message {
            "load_audio" => {
                let sender = self.message_senders.get("load_audio").unwrap();
                let test_audio_meta = AudioMeta {
                    content_length: 100.
                };

                sender.send(AudioPlayerMessage::ResponseAudioMeta(test_audio_meta));
            },

            _ => println!("got message: {}", message),
        }
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

    // pub fn start_message_handler(&self) {
        
    //     let handle = thread::spawn(move || {
    //         loop {
    //             let msg = self.command_rx.recv().unwrap();

    //         }
    //     });
    // }
}