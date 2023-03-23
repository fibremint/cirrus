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

// #[derive(Debug)]
// pub struct PlayerStatus {
//     status: usize,
//     pos: usize,
//     remain_buf: f32,
// }

#[derive(Debug)]
pub struct AudioMeta {
    pub content_length: f64,
}


// impl TryFrom<AudioPlayerResponse> for AudioMeta {
//     type Error = AudioPlayerResponse;

//     fn try_from(value: AudioPlayerResponse) -> Result<Self, Self::Error> {
//         match value {
//             AudioPlayerResponse::AudioMeta(v) => Ok(v),
//             r => Err(r)
//         }
//     }
// }

pub struct CommonMessage {
    status: String,
}

pub enum AudioPlayerMessage {
    // ResponsePlayerStatus(PlayerStatus),
    ResponseAudioMeta(AudioMeta),
    Common(CommonMessage)
}

pub enum AudioPlayerResponse {
    // PlayerStatus(PlayerStatus),
    AudioMeta(AudioMeta),
    // Common(CommonMessage),
    None,
}

impl Into<Option<AudioMeta>> for AudioPlayerResponse {
    fn into(self) -> Option<AudioMeta> {
        match self {
            AudioPlayerResponse::AudioMeta(v) => Some(v),
            _ => None,
        }
    }
}

// impl Into<Option<AudioMeta>> for AudioPlayerResponse {
//     fn into(self) -> Option<AudioMeta> {
//         match self {
//             AudioPlayerResponse::AudioMeta(v) => Some(v),
//             _ => None,
//         }
//     }
// }

// impl Into<AudioMeta> for AudioPlayerResponse {
//     fn into(self) -> AudioMeta {
//         match self {
//             AudioPlayerResponse::AudioMeta(v) => Some(v),
//             _ => None,
//         }
//     }
// }


pub enum AudioPlayerRequest {
    AddAudio(AddAudioMessage),
    Play,
    Pause,
    Stop,
    SetPlaybackPos(SetPlaybackPosMessage),
}

// #[derive(Clone, Debug, PartialEq, Eq, Hash, Sequence)]
// pub enum RequestType {
//     LoadAudio,
//     PauseAudio,
//     StartAudio,
//     StopAudio,
//     SetListenUpdatedEvents,
//     SetPlaybackPosition,
// }

#[derive(Clone, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum RequestType {
    AddAudio,
    Play,
    Pause,
    Stop,
    SetPlaybackPosition,
}


pub struct AddAudioMessage {
    pub audio_tag_id: String,
}

pub struct SetPlaybackPosMessage {
    pub position_sec: f64,
}

fn process_request(
    audio_player: &mut AudioPlayerImpl,
    request: &AudioPlayerRequest,
    rt_handle: &Handle,
    response_channels: &ResponseChannels,
    // respones_channels: &HashMap<RequestType, (Sender<AudioPlayerResponse>, Receiver<AudioPlayerResponse>)>,
) -> Result<(), anyhow::Error> {
    match request {
        AudioPlayerRequest::AddAudio(msg) => {
            let content_length = audio_player.add_audio(&msg.audio_tag_id, &rt_handle)?;

            let (sender, _) = response_channels.get(&RequestType::AddAudio).unwrap();
            sender.send(AudioPlayerResponse::AudioMeta(
                AudioMeta {
                    content_length,
                }
            ))?;
        },
        AudioPlayerRequest::Play => {
            audio_player.play()?;

            let (sender, _) = response_channels.get(&RequestType::Play).unwrap();
            sender.send(
                AudioPlayerResponse::None
            )?;
        },
        AudioPlayerRequest::Pause => {
            audio_player.pause()?;

            let (sender, _) = response_channels.get(&RequestType::Pause).unwrap();
            sender.send(
                AudioPlayerResponse::None
            )?;
        },
        AudioPlayerRequest::Stop => {
            audio_player.stop()?;

            let (sender, _) = response_channels.get(&RequestType::Stop).unwrap();
            sender.send(
                AudioPlayerResponse::None
            )?;
        },
        AudioPlayerRequest::SetPlaybackPos(msg) => {
            audio_player.set_playback_position(msg.position_sec)?;

            let (sender, _) = response_channels.get(&RequestType::SetPlaybackPosition).unwrap();
            sender.send(
                AudioPlayerResponse::None
            )?;
        },
    }

    Ok(())
}

fn start_audio_player_thread(
    grpc_endpoint: &str,
    event_sender: Option<Sender<UpdatedStreamMessage>>,
    request_receiver: Receiver<AudioPlayerRequest>,
    response_channels: ResponseChannels,
    // response_channels: Arc<ResponseChannels>,
    // respones_channels: &HashMap<RequestType, (Sender<AudioPlayerResponse>, Receiver<AudioPlayerResponse>)>,
    rt_handle: Handle,
) -> Result<(), anyhow::Error> {

    let _grpc_endpoint = grpc_endpoint.to_string();

    thread::spawn(move || {
        let mut audio_player = AudioPlayerImpl::new(
            &_grpc_endpoint,
            event_sender,
        ).unwrap();

        loop {
            // let request = request_receiver.recv().unwrap();
            let request = match request_receiver.recv() {
                Ok(request) => request,
                Err(err) => {
                    println!("audio player manager channel disconnected, stop audio player thread");
                    break;
                },
            };

            process_request(&mut audio_player, &request, &rt_handle, &response_channels).unwrap();
        }
    });

    Ok(())
}

// struct Channel<T> {
//     sender: Sender<T>,
//     receiver: Receiver<T>
// }

type ResponseChannels = HashMap<RequestType, (Sender<AudioPlayerResponse>, Receiver<AudioPlayerResponse>)>;

// struct Channel<T>(Sender<T>, Receiver<T>);

// fn create_channels()

pub struct AudioPlayer {
    // event_sender: Sender<UpdatedStreamMessage>,
    // event_receiver: Receiver<UpdatedStreamMessage>,

    request_sender: Sender<AudioPlayerRequest>,
    // message_senders: HashMap<RequestType, Sender<AudioPlayerMessage>>,
    // request_channels: HashMap<RequestType, Channel<AudioPlayerRequest>>,
    // response_channels: HashMap<RequestType, Channel<AudioPlayerResponse>>,
    response_channels: ResponseChannels,
    // response_channels: Arc<ResponseChannels>,
}

impl AudioPlayer {
    pub fn new(
        rt_handle: Handle,
        event_sender: Option<Sender<UpdatedStreamMessage>>,
        grpc_endpoint: &str,
    ) -> Result<Self, anyhow::Error> {
        // let (event_sender, event_receiver) = crossbeam_channel::bounded::<UpdatedStreamMessage>(1);
        let (request_sender, request_receiver) = crossbeam_channel::unbounded::<AudioPlayerRequest>();
        
        // let mut request_channels = HashMap::new();
        let mut response_channels = HashMap::new();
        
        for request_type in enum_iterator::all::<RequestType>() {
            // let (request_sender, request_receiver) = crossbeam_channel::bounded::<AudioPlayerRequest>(1);
            let (response_sender, response_receiver) = crossbeam_channel::bounded::<AudioPlayerResponse>(1);

            // request_channels.insert(request_type, Channel::<AudioPlayerRequest> {
            //     sender: request_sender,
            //     receiver: request_receiver,
            // });

            response_channels.insert(request_type, (response_sender, response_receiver));

            // response_channels.insert(request_type, Channel::<AudioPlayerResponse> {
            //     sender: response_sender,
            //     receiver: response_receiver,
            // });
        }
        
        start_audio_player_thread(
            grpc_endpoint,
            event_sender,
            // None,
            request_receiver,
            response_channels.clone(),
            rt_handle
        )?;

        Ok(Self {
            // event_sender,
            // event_receiver,
            request_sender,
            // request_channels,
            response_channels,
        })
    }

    pub fn add_audio(
        &self,
        audio_tag_id: &str
    ) -> Result<AudioMeta, anyhow::Error> {
        self.request_sender.send(AudioPlayerRequest::AddAudio(
            AddAudioMessage { audio_tag_id: audio_tag_id.to_string() }
        ))?;

        let (_, receiver) = self.response_channels.get(&RequestType::AddAudio).unwrap();

        let res = receiver.recv()?;
        let inner: Option<AudioMeta> = res.into();

        Ok(inner.unwrap())
    }

    pub fn play(
        &self
    ) -> Result<(), anyhow::Error> {
        self.request_sender.send(AudioPlayerRequest::Play)?;

        let (_, receiver) = self.response_channels.get(&RequestType::Play).unwrap();
        receiver.recv()?;

        Ok(())
    }

    pub fn pause(
        &self
    ) -> Result<(), anyhow::Error> {
        self.request_sender.send(AudioPlayerRequest::Pause)?;

        let (_, receiver) = self.response_channels.get(&RequestType::Pause).unwrap();
        receiver.recv()?;

        Ok(())
    }
    
    pub fn stop(
        &self
    ) -> Result<(), anyhow::Error> {
        self.request_sender.send(AudioPlayerRequest::Stop)?;

        let (_, receiver) = self.response_channels.get(&RequestType::Stop).unwrap();
        receiver.recv()?;

        Ok(())
    }

    pub fn set_playback_position(
        &self,
        position_sec: f64
    ) -> Result<(), anyhow::Error> {
        self.request_sender.send(AudioPlayerRequest::SetPlaybackPos(
            SetPlaybackPosMessage { position_sec }
        ))?;

        let (_, receiver) = self.response_channels.get(&RequestType::SetPlaybackPosition).unwrap();
        receiver.recv()?;

        Ok(())
        // let (_, receiver) = self.response_channels.get(&RequestType::SetPlaybackPosition).unwrap();
        // // let res: Option<AudioMeta> = receiver.recv()?;
        // let res = receiver.recv()?;
        // let inner: Option<SetPlaybackPosMessage> = res.into();

        // Ok(inner.unwrap())
    }
}

// pub struct AudioPlayer {
//     inner: AudioPlayerInner,
//     message_senders: HashMap<RequestType, Sender<AudioPlayerMessage>>,
//     // event_sender: Option<Sender<UpdatedStreamMessage>>,

//     command_tx: Option<Sender<AudioPlayerRequest>>,
//     is_listen_updated_events: bool,
// }

// impl AudioPlayer {
//     pub fn new(
//         grpc_endpoint: &str,
//         event_sender: Option<Sender<UpdatedStreamMessage>>,
//     ) -> Result<Self, anyhow::Error> {
//         println!("create audio player core");

//         Ok(Self {
//             // inner: AudioPlayerInner::new(grpc_endpoint)?,
//             inner: AudioPlayerInner::new(grpc_endpoint, event_sender)?,
//             message_senders: HashMap::default(),
//             // event_sender,
//             command_tx: None,
//             is_listen_updated_events: false,
//         })
//     }

//     // pub fn enroll_event_sender(
//     //     &mut self,
//     //     event_sender: Sender<UpdatedStreamMessage>,
//     // ) {
//     //     // self.event_sender = Some(event_sender);
//     //     self.inner.enroll_event_sender(event_sender);
//     // }

//     pub fn enroll_mesage_sender(
//         &mut self,
//         name: RequestType,
//         message_sender: Sender<AudioPlayerMessage>,
//     ) {
//         self.message_senders.insert(name, message_sender);
//     }

//     pub fn dispatch_message(
//         &mut self,
//         rt_handle: Handle,
//         message: AudioPlayerRequest,
//         // command_rx: Receiver<AudioPlayerRequest>,
//         command_tx: Sender<AudioPlayerRequest>,
//     ) {
//         match message {
//             // TODO: match method name
//             AudioPlayerRequest::LoadAudio(value) => {
//                 thread::spawn(move || {
//                     rt_handle.block_on(async move {
//                         let audio_source = AudioSource::new(
//                             "http://localhost:50000", 
//                             &None, 
//                             &value.audio_tag_id,
//                         ).await.unwrap();

//                         command_tx.send(AudioPlayerRequest::AddAudioSource(audio_source)).unwrap();
//                     });
//                 });
//             },
//             AudioPlayerRequest::SetPlaybackPos(value) => {
//                 let res = self.inner.set_playback_position(value.position_sec).unwrap();

//                 let sender = self.message_senders
//                     .get(&RequestType::SetPlaybackPosition)
//                     .unwrap();

//                 sender.send(AudioPlayerMessage::Common(
//                     CommonMessage { status: "ok".to_string() }
//                 )).unwrap();
//             },
//             // AudioPlayerRequest::SetListenUpdatedEvents(value) => {
//             //     self.set_listen_updated_events(value);

//             //     let sender = self.message_senders.get_mut("set_listen_updated_events").unwrap();
//             //     sender.send(AudioPlayerMessage::Common(
//             //         CommonMessage { status: "ok".to_string() }
//             //     )).unwrap();

//             //     // let player_status = self.inner.get_player_status();

//             //     // sender.send(AudioPlayerMessage::ResponsePlayerStatus(player_status)).unwrap();
//             // },
//             AudioPlayerRequest::StartAudio => {
//                 self.inner.play().unwrap();

//                 let sender = self.message_senders
//                     .get(&RequestType::StartAudio)
//                     .unwrap();

//                 sender.send(AudioPlayerMessage::Common(
//                     CommonMessage { status: "ok".to_string() }
//                 )).unwrap();
//             },
//             AudioPlayerRequest::StopAudio => {
//                 self.inner.stop().unwrap();

//                 let sender = self.message_senders
//                     .get(&RequestType::StopAudio)
//                     .unwrap();

//                 sender.send(AudioPlayerMessage::Common(
//                     CommonMessage { status: "ok".to_string() }
//                 )).unwrap();
//             },
//             AudioPlayerRequest::PauseAudio => {
//                 self.inner.pause().unwrap();

//                 let sender = self.message_senders
//                     .get(&RequestType::PauseAudio)
//                     .unwrap();

//                 sender.send(AudioPlayerMessage::Common(
//                     CommonMessage { status: "ok".to_string() }
//                 )).unwrap();
//             },
//             AudioPlayerRequest::AddAudioSource(value) => {
//                 let content_length = self.inner.add_audio(&rt_handle, value).unwrap();
                
//                 let sender = self.message_senders
//                     .get(&RequestType::LoadAudio)
//                     .unwrap();

//                 sender.send(AudioPlayerMessage::ResponseAudioMeta(
//                     AudioMeta { content_length }
//                 )).unwrap();
//             }

//             _ => println!("got unexpected message"),
//         }
//     }

//     // fn set_listen_updated_events(&mut self, is_listen: bool) {
//     //     if is_listen && !self.is_listen_updated_events {

//     //         let handle = std::thread::spawn(move || loop {
//     //             let playback_payload = PlaybackPayload {
//     //                 status: audio_player.get_status(),
//     //                 pos: audio_player.get_playback_position(),
//     //                 remain_buf: audio_player.get_remain_sample_buffer_sec(),
//     //             };

//     //             if let Err(e) = window.emit("update-playback-pos", playback_payload) {
//     //                 println!("{:?}", e);
//     //             }

//     //             std::thread::sleep(std::time::Duration::from_millis(200));
//     //         });
//     //     } else if !is_listen && self.is_listen_updated_events {

//     //     }
//     // }
// }

pub struct AudioPlayerImpl {
    device_context: AudioDeviceContext,
    streams: VecDeque<AudioStream>,
    status: usize,
    event_sender: Option<Sender<UpdatedStreamMessage>>
}

impl AudioPlayerImpl {
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

    // fn enroll_event_sender(event_sender: Sender<UpdatedStreamMessage>) {

    // }

    // fn get_current_stream(&self) {
    //     todo!()
    // }

    pub fn add_audio(
        &mut self,
        audio_tag_id: &str,
        // sender: &Sender<AudioPlayerMessage>,
        rt_handle: &Handle,
        // audio_source: AudioSource,
    ) -> Result<f64, anyhow::Error> {
        // println!("process add audio request, params: {:?}", audio_source);
        // let audio_source = rt_handle.block_on(async move {
        //     AudioSource::new(
        //         "http://localhost:50000",
        //         &None,
        //         audio_tag_id
        //     ).await.unwrap()
        // });

        let audio_stream = AudioStream::new(
            // audio_source.id.clone(),
            audio_tag_id,
            rt_handle,
            &self.device_context,
            // audio_source,
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

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        println!("process stop request");

        self.streams.clear();

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