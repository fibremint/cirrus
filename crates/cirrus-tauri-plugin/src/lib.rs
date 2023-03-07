use std::path::PathBuf;

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

const RES_PATH_STR: &'static str = "resources";
const CONFIG_PATH_STR: &'static str = "configs/cirrus/client.toml";

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
        .setup(|app, _api| {
            //let res_root_path = resolve_res_path(app, &RES_PATH_STR);
            let state = state::AppState::new().unwrap();

            app.manage(state);

            Ok(())
        })
        .build()
}
