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

pub fn init<R: Runtime>() -> TauriPlugin<R> {
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
            let config_path = app.path_resolver()
                .resolve_resource("resources/configs/cirrus/client.toml")
                .expect("failed to resolve config file");

            let config_path = dunce::canonicalize(config_path).unwrap();

            let state = state::AppState::new(&config_path);
            app.manage(state);

            Ok(())
        })
        .build()
}