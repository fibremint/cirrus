use std::{sync::{Arc, Mutex, Condvar}};
use cirrus_client_core::{AudioPlayer, audio::UpdatedStreamMessage};
use crossbeam_channel::{Receiver, Sender};
use tauri::{Runtime, Window};

pub struct AudioEventChannelState<R: Runtime> {
    pub event_sender: Sender<UpdatedStreamMessage>,
    pub event_receiver: Receiver<UpdatedStreamMessage>,

    pub send_event_condvar: Arc<(Mutex<bool>, Condvar)>,
    pub window: Arc<Mutex<Option<Window<R>>>>,
}

impl<R> AudioEventChannelState<R> 
where
    R: Runtime
{
    pub fn new(
        event_sender: Sender<UpdatedStreamMessage>,
        event_receiver: Receiver<UpdatedStreamMessage>,
    ) -> Self {

        Self {
            event_sender,
            event_receiver,

            send_event_condvar: Arc::new((Mutex::new(false), Condvar::new())),
            window: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct AudioPlayerState(pub AudioPlayer);

impl AudioPlayerState {
    pub fn new(
        event_sender: Option<Sender<UpdatedStreamMessage>>,
        grpc_endpoint: &str
    ) -> Result<Self, anyhow::Error> {

        Ok(Self {
            0: AudioPlayer::new(event_sender, grpc_endpoint)?
        })   
    }
}
