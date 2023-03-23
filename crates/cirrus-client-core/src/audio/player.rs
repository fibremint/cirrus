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
use enum_iterator::Sequence;
use tokio::{
    time::Duration, 
    sync::{RwLock}, runtime::Handle,
};
use tonic::transport::ClientTlsConfig;

use crate::{audio_player::state::PlaybackStatus, tls};
use crate::dto::AudioSource;

use crate::audio::{device::AudioDeviceContext, stream::AudioStream};

use super::stream::{UpdatedStreamMessage, UpdatedPlaybackMessage};

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
    SetListenUpdatedEvents(bool),
    StartAudio,
    StopAudio,
    PauseAudio,
    AddAudioSource(AudioSource),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum RequestType {
    LoadAudio,
    PauseAudio,
    StartAudio,
    StopAudio,
    SetListenUpdatedEvents,
    SetPlaybackPosition,
}

pub struct LoadAudioMessage {
    pub audio_tag_id: String,
}

pub struct SetPlaybackPosMessage {
    pub position_sec: f64,
}

pub struct AudioPlayer {
    inner: AudioPlayerInner,
    message_senders: HashMap<RequestType, Sender<AudioPlayerMessage>>,
    // event_sender: Option<Sender<UpdatedStreamMessage>>,

    command_tx: Option<Sender<AudioPlayerRequest>>,
    is_listen_updated_events: bool,
}

impl AudioPlayer {
    pub fn new(
        grpc_endpoint: &str,
        event_sender: Option<Sender<UpdatedStreamMessage>>,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            // inner: AudioPlayerInner::new(grpc_endpoint)?,
            inner: AudioPlayerInner::new(grpc_endpoint, event_sender)?,
            message_senders: HashMap::default(),
            // event_sender,
            command_tx: None,
            is_listen_updated_events: false,
        })
    }

    // pub fn enroll_event_sender(
    //     &mut self,
    //     event_sender: Sender<UpdatedStreamMessage>,
    // ) {
    //     // self.event_sender = Some(event_sender);
    //     self.inner.enroll_event_sender(event_sender);
    // }

    pub fn enroll_mesage_sender(
        &mut self,
        name: RequestType,
        message_sender: Sender<AudioPlayerMessage>,
    ) {
        self.message_senders.insert(name, message_sender);
    }

    pub fn dispatch_message(
        &mut self,
        rt_handle: Handle,
        message: AudioPlayerRequest,
        // command_rx: Receiver<AudioPlayerRequest>,
        command_tx: Sender<AudioPlayerRequest>,
    ) {
        match message {
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
                let res = self.inner.set_playback_position(value.position_sec).unwrap();

                let sender = self.message_senders
                    .get(&RequestType::SetPlaybackPosition)
                    .unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            // AudioPlayerRequest::SetListenUpdatedEvents(value) => {
            //     self.set_listen_updated_events(value);

            //     let sender = self.message_senders.get_mut("set_listen_updated_events").unwrap();
            //     sender.send(AudioPlayerMessage::Common(
            //         CommonMessage { status: "ok".to_string() }
            //     )).unwrap();

            //     // let player_status = self.inner.get_player_status();

            //     // sender.send(AudioPlayerMessage::ResponsePlayerStatus(player_status)).unwrap();
            // },
            AudioPlayerRequest::StartAudio => {
                self.inner.play().unwrap();

                let sender = self.message_senders
                    .get(&RequestType::StartAudio)
                    .unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::StopAudio => {
                self.inner.stop().unwrap();

                let sender = self.message_senders
                    .get(&RequestType::StopAudio)
                    .unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::PauseAudio => {
                self.inner.pause().unwrap();

                let sender = self.message_senders
                    .get(&RequestType::PauseAudio)
                    .unwrap();

                sender.send(AudioPlayerMessage::Common(
                    CommonMessage { status: "ok".to_string() }
                )).unwrap();
            },
            AudioPlayerRequest::AddAudioSource(value) => {
                let content_length = self.inner.add_audio(&rt_handle, value).unwrap();
                
                let sender = self.message_senders
                    .get(&RequestType::LoadAudio)
                    .unwrap();

                sender.send(AudioPlayerMessage::ResponseAudioMeta(
                    AudioMeta { content_length }
                )).unwrap();
            }

            _ => println!("got unexpected message"),
        }
    }

    // fn set_listen_updated_events(&mut self, is_listen: bool) {
    //     if is_listen && !self.is_listen_updated_events {

    //         let handle = std::thread::spawn(move || loop {
    //             let playback_payload = PlaybackPayload {
    //                 status: audio_player.get_status(),
    //                 pos: audio_player.get_playback_position(),
    //                 remain_buf: audio_player.get_remain_sample_buffer_sec(),
    //             };

    //             if let Err(e) = window.emit("update-playback-pos", playback_payload) {
    //                 println!("{:?}", e);
    //             }

    //             std::thread::sleep(std::time::Duration::from_millis(200));
    //         });
    //     } else if !is_listen && self.is_listen_updated_events {

    //     }
    // }
}

pub struct AudioPlayerInner {
    device_context: AudioDeviceContext,
    streams: VecDeque<AudioStream>,
    status: usize,
    event_sender: Option<Sender<UpdatedStreamMessage>>
}

impl AudioPlayerInner {
    pub fn new(
        grpc_endpoint: &str,
        event_sender: Option<Sender<UpdatedStreamMessage>>,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            device_context: AudioDeviceContext::new()?,
            streams: VecDeque::default(),
            status: 0,
            event_sender,
        })
    }

    fn enroll_event_sender(event_sender: Sender<UpdatedStreamMessage>) {

    }

    fn get_current_stream(&self) {
        todo!()
    }

    pub fn add_audio(
        &mut self,
        // audio_tag_id: &str,
        // sender: &Sender<AudioPlayerMessage>,
        rt_handle: &Handle,
        audio_source: AudioSource,
    ) -> Result<f64, anyhow::Error> {
        println!("process add audio request, params: {:?}", audio_source);

        let audio_stream = AudioStream::new(
            audio_source.id.clone(),
            rt_handle,
            &self.device_context,
            audio_source,
            Some(5),
            150.,
            self.event_sender.clone(),
        )?;

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

        Ok(())    
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        println!("process pause request");

        self.streams.get(0).unwrap().pause()?;
        
        Ok(())
    }

    pub fn set_playback_position(&self, position_sec: f64) -> Result<(), anyhow::Error> {
        println!("process set_playback_position request, params: {}", position_sec);

        self.streams.get(0).unwrap().set_playback_position(position_sec)?;

        Ok(())
    }

    // pub fn get_player_status(&self) -> PlayerStatus {
    //     println!("process get_player_status");

    //     PlayerStatus {
    //         status: 0,
    //         pos: 1000,
    //         remain_buf: 2000.0,
    //     }
    // }
}