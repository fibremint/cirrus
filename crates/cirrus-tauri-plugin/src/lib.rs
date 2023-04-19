use std::sync::{Mutex, Arc, Condvar};

use cirrus_client_core::audio::UpdatedStreamMessage;
use state::AudioPlayerState;
use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder},
    Manager, Window,
};
// use dunce;

use crate::state::AudioEventChannelState;

pub mod state;
pub mod commands;
mod settings;

// fn manage_player_event<R: Runtime>(window: &Window<R>) {
//     // let id = window.listen(event, handler)
// }

// fn resolve_res_path<R: Runtime>(app: &AppHandle<R>, path: &str) -> PathBuf {
//     let res_path = app.path_resolver()
//         .resolve_resource(path)
//         .expect("failed to resolve file path");

//     let res_path = dunce::canonicalize(res_path).unwrap();

//     res_path
// }

// const RES_PATH_STR: &'static str = "resources";
// const CONFIG_PATH_STR: &'static str = "configs/cirrus/client.toml";


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
    .setup(|app, _api| {
      let audio_event_channel_state = start_audio_event_send_thread::<R>();

      let _event_sender = audio_event_channel_state.event_sender.clone();
      app.manage(audio_event_channel_state);

      app.manage(AudioPlayerState::new(
          Some(_event_sender),
          "http://localhost:50000")?
      );

      Ok(())
      })
    .build()
}