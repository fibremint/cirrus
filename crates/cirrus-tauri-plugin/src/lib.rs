use std::{path::PathBuf, sync::{mpsc, Mutex, Arc, Condvar}, thread, collections::HashMap};

use cirrus_client_core::{AudioPlayer, audio::{AudioPlayerMessage, AudioPlayerRequest, UpdatedStreamMessage, RequestType}};
use crossbeam_channel::{Receiver, Sender};
use enum_iterator::Sequence;
use state::AudioPlayerChannelState;
use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager, Window,
};
use dunce;

use crate::state::AudioEventChannelState;

pub mod state;
pub mod commands;
mod settings;

fn manage_player_event<R: Runtime>(window: &Window<R>) {
    // let id = window.listen(event, handler)
}

fn resolve_res_path<R: Runtime>(app: &AppHandle<R>, path: &str) -> PathBuf {
    let res_path = app.path_resolver()
        .resolve_resource(path)
        .expect("failed to resolve file path");

    let res_path = dunce::canonicalize(res_path).unwrap();

    res_path
}

// fn init_audio_player() {

// }


// pub fn manage_audio_player_events_sender<R: Runtime>(
//     is_listen: bool,
//     window: Window<R>,
//     // event_receiver: Receiver<UpdatedStreamMessage>,
// ) {
//     if is_listen {
//         std::thread::spawn(move || {
//             loop {
//                 window.emit(UPDATED_AUDIO_PLAYER_EVENT_NAME, payload)
//             }
//         });
//     } else {

//     }
// }

const RES_PATH_STR: &'static str = "resources";
const CONFIG_PATH_STR: &'static str = "configs/cirrus/client.toml";

fn start_audio_player_thread(
    update_event_sender: Option<Sender<UpdatedStreamMessage>>,
) -> AudioPlayerChannelState {
    // let audio_msg_receivers: HashMap<RequestType, Receiver<AudioPlayerMessage>> = HashMap::new();
    let audio_msg_receivers: Arc<Mutex<HashMap<RequestType, Receiver<AudioPlayerMessage>>>> = Arc::new(Mutex::new(HashMap::new()));

    let (audio_cmd_tx, audio_cmd_rx) = crossbeam_channel::unbounded::<AudioPlayerRequest>();
    
    let _audio_cmd_tx = audio_cmd_tx.clone();
    let rt_handle = tauri::async_runtime::TokioHandle::current();
    let mut _audio_msg_receivers = audio_msg_receivers.clone();

    thread::spawn(move || {
        let mut audio_player = AudioPlayer::new(
            "http://localhost:50000",
            update_event_sender,
        ).unwrap();

        {
            let mut audio_msg_receivers_guard = _audio_msg_receivers.lock().unwrap();

            for request_type in enum_iterator::all::<RequestType>() {
                let (message_sender, message_receiver) = crossbeam_channel::unbounded::<AudioPlayerMessage>();
                
                audio_player.enroll_mesage_sender(request_type.clone(), message_sender);
                audio_msg_receivers_guard.insert(request_type.clone(), message_receiver);
            }
        }

        // if let Some(event_sender) = update_event_sender {
        //     audio_player.enroll_event_sender(event_sender);
        // }

        loop {
            let message = audio_cmd_rx.recv().unwrap();

            audio_player.dispatch_message(
                rt_handle.clone(),
                message,
                _audio_cmd_tx.clone()
            );
        }
    });
    
    AudioPlayerChannelState {
        audio_cmd_sender: audio_cmd_tx,
        audio_msg_receivers,
    }
}

const UPDATED_AUDIO_PLAYER_EVENT_NAME: &'static str = "update-playback";

fn start_audio_event_send_thread<R: Runtime>() -> AudioEventChannelState<R> {
    let (audio_event_sender, audio_event_receiver) = crossbeam_channel::unbounded::<UpdatedStreamMessage>();

    let send_event_condvar = Arc::new((Mutex::new(false), Condvar::new()));

    let _send_event_condvar = send_event_condvar.clone();
    let _audio_event_receiver = audio_event_receiver.clone();

    let window: Arc<Mutex<Option<Window<R>>>> = Arc::new(Mutex::new(None));
    let _window = window.clone();

    std::thread::spawn(move || {
        loop {
            {
                let (send_event_mutex, send_event_cv) = &*_send_event_condvar;
                let mut send_event = send_event_mutex.lock().unwrap();

                while !*send_event {
                    send_event = send_event_cv.wait(send_event).unwrap();
                }
            }

            let message = _audio_event_receiver.recv().unwrap();

            let window_guard = _window.lock().unwrap();
            if let Some(w) = &*window_guard {
                if let Err(e) = w.emit(UPDATED_AUDIO_PLAYER_EVENT_NAME, message) {
                    println!("{:?}", e);
                }
            }

            // if window_guard.is_none() {
            //     println!("error: window is none");
            //     continue;
            // }

            // let w = window_guard.unwrap();
            // w.emit(UPDATED_AUDIO_PLAYER_EVENT_NAME, message).unwrap();

            // if let Some(w) = _window.clone() {
            //     w.emit(UPDATED_AUDIO_PLAYER_EVENT_NAME, message).unwrap();
            // }
        }
    });

    AudioEventChannelState {
        event_sender: audio_event_sender,
        event_receiver: audio_event_receiver,
        send_event_condvar,
        window,
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    // let tokio_handle = tokio::runtime::Handle::current();
    
    Builder::new("cirrus")
        .invoke_handler(tauri::generate_handler![
            commands::get_audio_tags,

            commands::load_audio,
            commands::pause_audio,
            commands::start_audio,
            commands::stop_audio,
            commands::set_listen_updated_events,
            commands::set_playback_position,
        ])
        .setup(|app| {
            let audio_event_channel_state = start_audio_event_send_thread::<R>();

            let _event_sender = audio_event_channel_state.event_sender.clone();
            app.manage(audio_event_channel_state);

            let audio_player_channel_state = start_audio_player_thread(
                Some(_event_sender),
            );
            
            app.manage(audio_player_channel_state);

            Ok(())
        })
        .build()
}