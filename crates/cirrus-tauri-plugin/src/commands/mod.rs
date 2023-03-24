use tauri::{State, Window, Runtime};

use cirrus_client_core::request;
use cirrus_protobuf::api::AudioTagRes;

use crate::state::AudioEventChannelState;
use crate::state::AudioPlayerState;

#[tauri::command]
pub fn set_playback_position(
    playback_pos: f64,
    state: State<'_, AudioPlayerState>,
) -> Result<(), &'static str> {
    state.0.set_playback_position(playback_pos).unwrap();
   
    Ok(())
}

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AudioPlayerState>,
    audio_tag_id: String
) -> Result<f64, &'static str> {

    println!("got load audio command");

    let res = state.0.add_audio(&audio_tag_id).unwrap();

    Ok(res.content_length)
}

#[tauri::command]
pub fn start_audio(
    state: State<'_, AudioPlayerState>,
) -> Result<(), &'static str> {

    println!("got start audio command");

    state.0.play().unwrap();
   
    Ok(())
}

#[tauri::command]
pub fn stop_audio(
    state: State<'_, AudioPlayerState>,
) -> Result<(), &'static str> {

    println!("got stop audio command");
    state.0.stop().unwrap();

    Ok(())
}

#[tauri::command]
pub fn pause_audio(
    state: State<'_, AudioPlayerState>,
) -> Result<(), &'static str> {
    println!("got pause audio command");

    state.0.pause().unwrap();

    Ok(())
}

#[tauri::command] 
pub fn set_listen_updated_events<R: Runtime>(
    is_listen: bool,
    window: Window<R>,
    state: State<'_, AudioEventChannelState<R>>,
) {
    println!("got set_listen_updated_events");

    let _send_event_condvar = state.send_event_condvar.clone();
    let (send_event_mutex, send_event_cv) = &*_send_event_condvar;
    let mut send_event_guard = send_event_mutex.lock().unwrap();

    *send_event_guard = is_listen;

    let mut window_guard = state.window.lock().unwrap();
    *window_guard = Some(window);
    
    send_event_cv.notify_one();
}

#[tauri::command]
pub async fn get_audio_tags(
    items_per_page: u64,
    page: u32,
) -> Result<Vec<AudioTagRes>, &'static str> {
    println!("got get-audio-tags command");

    // match request::get_audio_tags(
    //     &state.audio_player.server_state.grpc_endpoint,
    //     &state.audio_player.server_state.tls_config,
    //     items_per_page, 
    //     page as u64
    // ).await {
    //     Ok(audio_tags) => Ok(audio_tags),
    //     Err(_) => return Err("failed to get audio tags from server"),
    // }
    match request::get_audio_tags(
        "http://localhost:50000",
        &None,
        items_per_page, 
        page as u64
    ).await {
        Ok(audio_tags) => Ok(audio_tags),
        Err(_) => return Err("failed to get audio tags from server"),
    }
}
