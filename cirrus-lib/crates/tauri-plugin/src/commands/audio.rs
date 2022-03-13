use std::borrow::Borrow;

use http::Response;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

use crate::state::AppState;

// #[tauri::command]
// pub fn baz(app_state: State<'_, AppState>, target: String) {
//     println!("load audio fn ");
// }

// #[tauri::command]
// pub fn load_audio(
//     app_state: State<'_, AppState>,
//     target: String
// ) {
//     println!("load audio fn");
// }

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AppState>,
    request: String
) -> Result<String, String> {
    println!("load audio fn, got: {}", request);

    // let mut audio_player = state.audio_player.write();
    // audio_player.await;
    let mut audio_player = state.audio_player.write().await;
    audio_player.add_audio(request.as_str()).await.unwrap();
    audio_player.play();

    // state.audio_player.add_audio(request.as_str());

    // let audio_player = state.audio_player.clone().write().await;

    // let audio_player = state.audio_player.clone();
    // audio_player.write().await.add_audio(request.as_str());

    // let mut audio_player = state.audio_player.clone().write().await;
    // audio_player.add_audio(request.as_str());

    // let mut audio_player = state.audio_player.write().await;
    // let audio_player = state.audio_player.clone();
    // let audio_player = audio_player.write().await;

    Ok(String::from("ok"))

    // audio_player.await.write().add_audio(request.as_str());

    // state.audio_player.add_audio(request.as_str());
}