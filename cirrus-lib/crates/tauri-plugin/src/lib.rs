use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager, Window,
};

pub mod state;
pub mod commands;

fn manage_player_event<R: Runtime>(window: &Window<R>) {
    // let id = window.listen(event, handler)
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("cirrus")
        .invoke_handler(tauri::generate_handler![
            commands::load_audio,
            commands::get_audio_tags,
            commands::start_audio,
            commands::stop_audio,
            commands::pause_audio,
            commands::send_playback_position,
        ])
        .setup(|app_handle| {
            let state = state::AppState::new();
            app_handle.manage(state);

            Ok(())
        })
        .build()
}