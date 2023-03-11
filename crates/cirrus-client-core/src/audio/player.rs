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

pub struct CommonMessage {
    status: String,
}

pub enum AudioPlayerMessage {
    ResponsePlayerStatus(PlayerStatus),
    ResponseAudioMeta(AudioMeta),
    Common(CommonMessage)
}

pub enum AudioPlayerRequest {
    LoadAudio(LoadAudioMessage),
    SetPlaybackPos(SetPlaybackPosMessage),
    GetPlayerStatus,
    StartAudio,
    StopAudio,
    PauseAudio,
    AddAudioSource(AudioSource),
}

pub struct LoadAudioMessage {
    pub audio_tag_id: String,
}

pub struct SetPlaybackPosMessage {
    pub position_sec: f64,
}

pub struct AudioPlayer {
    inner: AudioPlayerInner,
    message_senders: HashMap<String, Sender<AudioPlayerMessage>>,

    command_tx: Option<Sender<AudioPlayerRequest>>,
}

impl AudioPlayer {
    pub fn new(
        grpc_endpoint: &str,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            inner: AudioPlayerInner::new(grpc_endpoint)?,
            message_senders: HashMap::default(),
            command_tx: None,
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
        rt_handle: Arc<Handle>,
        command_tx: Sender<AudioPlayerRequest>,
        command_rx: Receiver<AudioPlayerRequest>,
    ) {
        // self.command_tx = Some(command_tx);
        loop {
            while let Ok(value) = command_rx.try_recv() {
                self.dispatch_message(rt_handle.clone(), value, command_tx.clone());
            }
        }
    }

    pub fn dispatch_message(
        &mut self,
        rt_handle: Arc<Handle>,
        message_type: AudioPlayerRequest,
        // command_rx: Receiver<AudioPlayerRequest>,
        command_tx: Sender<AudioPlayerRequest>,
    ) {
        match message_type {
            // TODO: match method name
            AudioPlayerRequest::LoadAudio(value) => {
                thread::spawn(move || {
                    rt_handle.block_on(async move {
                        let audio_source = AudioSource::new(
                            "http://localhost:50000", 
                            &None, 
                            &value.audio_tag_id,
                        ).await.unwrap();

                        command_tx.send(AudioPlayerRequest::AddAudioSource(audio_source)).unwrap();
                    });
                });
            },
            AudioPlayerRequest::SetPlaybackPos(value) => {
                let sender = self.message_senders.get("set_playback_pos").unwrap();
                let res = self.inner.set_playback_position(value.position_sec).unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::GetPlayerStatus => {
                let sender = self.message_senders.get("get_player_status").unwrap();
                let player_status = self.inner.get_player_status();

                sender.send(AudioPlayerMessage::ResponsePlayerStatus(player_status)).unwrap();
            },
            AudioPlayerRequest::StartAudio => {
                let sender = self.message_senders.get("start_audio").unwrap();
                self.inner.play().unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::StopAudio => {
                let sender = self.message_senders.get("stop_audio").unwrap();
                self.inner.stop().unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::PauseAudio => {
                let sender = self.message_senders.get("pause_audio").unwrap();
                self.inner.pause().unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::AddAudioSource(value) => {
                let sender = self.message_senders.get("load_audio").unwrap();

                let content_length = self.inner.add_audio(&rt_handle, value).unwrap();
  
                sender.send(AudioPlayerMessage::ResponseAudioMeta(
                    AudioMeta { content_length }
                )).unwrap();
            }

            _ => println!("got unexpected message"),
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

    pub fn add_audio(
        &mut self,
        // audio_tag_id: &str,
        // sender: &Sender<AudioPlayerMessage>,
        rt_handle: &Arc<Handle>,
        audio_source: AudioSource,
    ) -> Result<f64, anyhow::Error> {
        println!("process add audio request, params: {:?}", audio_source);

        let audio_stream = AudioStream::new(rt_handle, &self.ctx, audio_source)?;

        self.streams.push_back(audio_stream);

        Ok(0.)
    }

    pub fn play(&mut self) -> Result<(), anyhow::Error> {
        println!("process play request");
        
        let audio_stream = self.streams.get_mut(0).unwrap();
        audio_stream.play()?;
        // self.streams.get(0).unwrap().play()?;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), anyhow::Error> {
        println!("process stop request");

        Ok(())    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        println!("process pause request");

        Ok(())
    }

    pub fn set_playback_position(&self, position_sec: f64) -> Result<(), anyhow::Error> {
        println!("process set_playback_position request, params: {}", position_sec);

        Ok(())
    }

    pub fn get_player_status(&self) -> PlayerStatus {
        println!("process get_player_status");

        PlayerStatus {
            status: 0,
            pos: 1000,
            remain_buf: 2000.0,
        }
    }
}