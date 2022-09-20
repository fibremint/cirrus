use tauri::{State, Window, Runtime};

use cirrus_client_lib::{
    request,
    audio_player::state::PlaybackStatus
};
use cirrus_grpc::api::AudioTagRes;

use crate::state::AppState;


#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackPayload {
    status: PlaybackStatus,
    pos: f32,
    remain_buf: f32,
}

#[tauri::command]
pub fn set_playback_position(
    state: State<'_, AppState>,
    playback_pos: f32
) -> Result<(), &'static str> {

    println!("got set playback position command");

    match state.audio_player.set_playback_position(playback_pos) {
        Ok(content_length) => return Ok(content_length),
        Err(_) => return Err("tauri-plugin: failed to add audio"),
    }
}

#[tauri::command]
pub fn send_audio_player_status<R: Runtime>(
    window: Window<R>,
    state: State<'_, AppState>,
) {
    let audio_player = state.audio_player.clone();

    std::thread::spawn(move || loop {
        let playback_payload = PlaybackPayload {
            status: audio_player.get_status(),
            pos: audio_player.get_playback_position(),
            remain_buf: audio_player.get_remain_sample_buffer_sec(),
        };

        if let Err(e) = window.emit("update-playback-pos", playback_payload) {
            println!("{:?}", e);
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
    });
}

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AppState>,
    audio_tag_id: String
) -> Result<f32, &'static str> {

    println!("got load audio command");

    match state.audio_player.add_audio(&audio_tag_id).await {
        Ok(content_length) => return Ok(content_length),
        Err(_) => return Err("tauri-plugin: failed to add audio"),
    }
}

#[tauri::command]
pub fn start_audio(
    state: State<'_, AppState>
) -> Result<(), &'static str> {

    println!("got start audio command");

    match state.audio_player.play() {
        Ok(())=> return Ok(()),
        Err(_) => return Err("tauri-plugin: failed to play audio"), 
    }
}

#[tauri::command]
pub fn stop_audio(
    state: State<'_, AppState>
) -> Result<(), &'static str> {

    println!("got stop audio command");

    state.audio_player.stop();

    Ok(())
}

#[tauri::command]
pub fn pause_audio(
    state: State<'_, AppState>
) -> Result<(), &'static str> {
    println!("got pause audio command");


    match state.audio_player.pause() {
        Ok(_) => Ok(()),
        Err(_) => Err("failed to pause audio"),
    }
}

#[tauri::command]
pub async fn get_audio_tags(
    items_per_page: u64,
    page: u32,
) -> Result<Vec<AudioTagRes>, &'static str> {
    println!("got get-audio-tags command");

    match request::get_audio_tags(items_per_page, page as u64).await {
        Ok(audio_tags) => Ok(audio_tags),
        Err(_) => return Err("failed to get audio tags from server"),
    }
}