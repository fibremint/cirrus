use tauri::{
    Runtime,
    plugin::{TauriPlugin, Builder}, 
    Invoke, 
    AppHandle, Manager
};

pub mod state;
pub mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("cirrus")
        .invoke_handler(tauri::generate_handler![
            commands::load_audio,
            commands::get_audio_tags,
            commands::stop_audio,
        ])
        .setup(|app_handle| {
            // ref: https://rfdonnelly.github.io/post/tauri-async-rust-process/
            let app_handle = app_handle.app_handle();
            let state = tauri::async_runtime::block_on(async move {
                state::AppState::new().await
            });
            app_handle.manage(state);
            Ok(())
        })
        .build()
}