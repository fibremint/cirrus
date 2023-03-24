use std::{
    collections::{VecDeque, HashMap},
    thread
};

use crossbeam_channel::{Sender, Receiver};
use enum_iterator::Sequence;
use tokio::runtime::Handle;
use tonic::transport::ClientTlsConfig;

use crate::audio::{device::AudioDeviceContext, stream::AudioStream};

use super::stream::UpdatedStreamMessage;

// use super::sample::AudioSample;

#[derive(Clone)]
pub struct ServerState {
    pub grpc_endpoint: String,
    pub tls_config: Option<ClientTlsConfig>,
}

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
    StreamReactEnd,
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
        AudioPlayerRequest::StreamReactEnd => {
            audio_player.next_stream();
        }
    }

    Ok(())
}

fn start_audio_player_thread(
    grpc_endpoint: &str,
    event_sender: Option<Sender<UpdatedStreamMessage>>,
    request_sender: Sender<AudioPlayerRequest>,
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
            request_sender,
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


type ResponseChannels = HashMap<RequestType, (Sender<AudioPlayerResponse>, Receiver<AudioPlayerResponse>)>;

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
        event_sender: Option<Sender<UpdatedStreamMessage>>,
        grpc_endpoint: &str,
    ) -> Result<Self, anyhow::Error> {
        let rt_handle = tokio::runtime::Handle::current();

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
            request_sender.clone(),
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

pub struct AudioPlayerImpl {
    device_context: AudioDeviceContext,
    streams: VecDeque<AudioStream>,
    status: usize,
    event_sender: Option<Sender<UpdatedStreamMessage>>,
    request_sender: Sender<AudioPlayerRequest>,
}

impl AudioPlayerImpl {
    pub fn new(
        grpc_endpoint: &str,
        event_sender: Option<Sender<UpdatedStreamMessage>>,
        request_sender: Sender<AudioPlayerRequest>,
    ) -> Result<Self, anyhow::Error> {
        println!("create audio player core");

        Ok(Self {
            device_context: AudioDeviceContext::new()?,
            streams: VecDeque::default(),
            status: 0,
            event_sender,
            request_sender,
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
            self.request_sender.clone(),
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

    pub fn next_stream(&mut self) -> Result<(), anyhow::Error> {
        self.streams.pop_front();

        if let Some(stream) = self.streams.get(0) {
            stream.play()?;
        } 

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