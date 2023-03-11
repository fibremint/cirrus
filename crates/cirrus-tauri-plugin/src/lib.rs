use std::{path::PathBuf, sync::{mpsc, Mutex, Arc}, thread, collections::HashMap};

use cirrus_client_core::{AudioPlayer, audio::{AudioPlayerMessage, AudioPlayerRequest}};
use crossbeam_channel::Receiver;
use state::AudioPlayerChannelState;
use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager, Window,
};
use dunce;

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

const RES_PATH_STR: &'static str = "resources";
const CONFIG_PATH_STR: &'static str = "configs/cirrus/client.toml";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    // let tokio_handle = tokio::runtime::Handle::current();
    
    Builder::new("cirrus")
        .invoke_handler(tauri::generate_handler![
            commands::load_audio,
            commands::get_audio_tags,
            commands::start_audio,
            commands::stop_audio,
            commands::pause_audio,
            commands::send_audio_player_status,
            commands::set_playback_position,
        ])
        .setup(|app| {
            // let res_root_path = resolve_res_path(app, &RES_PATH_STR);

            let mut audio_msg_receivers: HashMap<String, Receiver<AudioPlayerMessage>> = HashMap::new();

             let (audio_cmd_tx, audio_cmd_rx) = crossbeam_channel::unbounded::<AudioPlayerRequest>();
           
            let (set_playback_pos_sender, set_playback_pos_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("set_playback_pos".to_string(), set_playback_pos_receiver);

            let (get_player_status_sender, get_player_status_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("get_player_status".to_string(), get_player_status_receiver);

            let (load_audio_sender, load_audio_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("load_audio".to_string(), load_audio_receiver);

            let (start_audio_sender, start_audio_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("start_audio".to_string(), start_audio_receiver);
            
            let (stop_audio_sender, stop_audio_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("stop_audio".to_string(), stop_audio_receiver);

            let (pause_audio_sender, pause_audio_receiver) = crossbeam_channel::unbounded();
            audio_msg_receivers.insert("pause_audio".to_string(), pause_audio_receiver);

            // let mut audio_cmd_tx: Option<mpsc::Sender<String>>;
            let _audio_cmd_tx = audio_cmd_tx.clone();

            // let tokio_handle = tokio::runtime::Handle::current();
            let rt_handle = tauri::async_runtime::TokioHandle::current();

            let handle = thread::spawn(move || {
                let mut audio_player = AudioPlayer::new(
                    "http://localhost:50000",
                    // Some(audio_msg_tx.clone())
                ).unwrap();

                println!("enroll message senders");

                audio_player.enroll_mesage_sender("set_playback_pos".to_string(), set_playback_pos_sender);
                audio_player.enroll_mesage_sender("get_player_status".to_string(), get_player_status_sender);
                audio_player.enroll_mesage_sender("load_audio".to_string(), load_audio_sender);
                audio_player.enroll_mesage_sender("start_audio".to_string(), start_audio_sender);
                audio_player.enroll_mesage_sender("stop_audio".to_string(), stop_audio_sender);
                audio_player.enroll_mesage_sender("pause_audio".to_string(), pause_audio_sender);

                audio_player.start_command_handler(
                    Arc::new(rt_handle),
                    _audio_cmd_tx,
                    audio_cmd_rx
                );
                println!("started audio command handler (inner)");
                // audio_player.command_tx
            });

            println!("started audio command handler");

            let channel_state = AudioPlayerChannelState {
                // audio_cmd_sender: Mutex::new(audio_cmd_tx),
                audio_cmd_sender: audio_cmd_tx,
                // msg_rx: Mutex::new(audio_msg_rx),
                audio_msg_receivers,
            };

            app.manage(channel_state);

            Ok(())
        })
        .build()
}