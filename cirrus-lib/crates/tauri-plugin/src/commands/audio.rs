use tauri::State;

use cirrus_client_lib::request;
use cirrus_grpc::api::AudioTagRes;

use crate::state::AppState;

#[tauri::command]
pub async fn init_audio_player(
    state: State<'_, AppState>,
) -> Result<String, String> {
    println!("run init audio player command");

    state.audio_player.init().await;

    Ok(String::from("ok"))
}

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AppState>,
    audio_tag_id: String
) -> Result<String, String> {
    println!("load audio fn, got: {}", audio_tag_id);

    state.audio_player.add_audio(audio_tag_id.as_str()).await.unwrap();
    state.audio_player.play().await;

    Ok(String::from("ok"))
}

#[tauri::command]
pub async fn stop_audio(
    state: State<'_, AppState>
) -> Result<String, String> {
    println!("got request stop_audio");

    state.audio_player.stop().await;

    Ok(String::from("ok"))
}

#[tauri::command]
pub async fn get_audio_tags(
    items_per_page: u64,
    page: u32,
) -> Result<Vec<AudioTagRes>, String> {
    println!("got get-audio-tags commnad");

    let audio_tags = request::get_audio_tags(items_per_page, page as u64).await.unwrap();

    Ok(audio_tags)
}