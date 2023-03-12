use std::sync::Arc;

use cirrus_client_core::audio::AudioPlayerMessage;
use cirrus_client_core::audio::AudioPlayerRequest;
use cirrus_client_core::audio::LoadAudioMessage;
use cirrus_client_core::audio::RequestType;
use cirrus_client_core::audio::SetPlaybackPosMessage;
use tauri::{State, Window, Runtime};

use cirrus_client_core::{
    request,
    audio_player::state::PlaybackStatus
};
use cirrus_protobuf::api::AudioTagRes;

use crate::state::AudioEventChannelState;
// use crate::state::AppState;
// use crate::state::PluginState;
// use crate::state::AudioPlayerState;
use crate::state::AudioPlayerChannelState;


#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackPayload {
    status: PlaybackStatus,
    pos: f64,
    remain_buf: f64,
}

#[tauri::command]
pub fn set_playback_position(
    // state: State<'_, AudioPlayerState>,
    playback_pos: f64,
    state: State<'_, AudioPlayerChannelState>
) -> Result<(), &'static str> {
    // let mut audio_guard = state.audio_player.lock()?;

    println!("got set playback position command");

    // match state.audio_player.set_playback_position(playback_pos) {
    //     Ok(content_length) => return Ok(content_length),
    //     Err(_) => return Err("tauri-plugin: failed to add audio"),
    // }
    // state.audio_cmd_sender.send("set_playback_pos".to_string()).unwrap();
    state.audio_cmd_sender.send(AudioPlayerRequest::SetPlaybackPos(
        SetPlaybackPosMessage {
            position_sec: playback_pos
        }
    )).unwrap();

    // let mut sel = crossbeam_channel::Select::new();
    // let oper = sel.recv(state.audio_msg_receivers.get("set_playback_pos").unwrap());
    // let res = 
    // todo!()

    // let mut sel = crossbeam_channel::Select::new();
    // state.audio_cmd_sender.lock().unwrap().send("t")
    let receiver = state.audio_msg_receivers.lock().unwrap().get(&RequestType::SetPlaybackPosition).unwrap();

    // let res = receiver.recv().unwrap();
    // if let AudioPlayerMessage::ResponsePlayerStatus(status) = res {
    //     println!("value: {:?}", status);
    // }
    // println!("set_playback_pos res: {:?}", );
    // let recv_oper = sel.recv(receiver);
    // let res = sel.s

    Ok(())
}

#[tauri::command] 
pub fn set_listen_updated_events<R: Runtime>(
    is_listen: bool,
    window: Window<R>,
    state: State<'_, AudioEventChannelState<R>>,
) {
    // let audio_player = state.audio_player.clone();
    println!("got set_listen_updated_events");

    let _send_event_condvar = state.send_event_condvar.clone();
    let (send_event_mutex, send_event_cv) = &*_send_event_condvar;
    let mut send_event_guard = send_event_mutex.lock().unwrap();

    *send_event_guard = is_listen;

    let mut window_guard = state.window.lock().unwrap();
    *window_guard = Some(window);
    
    send_event_cv.notify_one();

    // if is_listen {
    //     *send_event = true;

    // } else {
    //     *send_event = false;

    // }

    // manage_audio_player_events_sender(
    //     is_listen,
    //     window, 
    //     // state.event_receiver.clone()
    //     state,
    // );
    
    // state.audio_cmd_sender.send("get_player_status".to_string()).unwrap();
    // state.audio_cmd_sender.send(AudioPlayerRequest::GetPlayerStatus).unwrap();
    // state.audio_cmd_sender.send(AudioPlayerRequest::SetListenUpdatedEvents(is_listen)).unwrap();

    // let receiver = state.audio_msg_receivers.get("get_player_status").unwrap();

    // let res = receiver.recv().unwrap();
    // if let AudioPlayerMessage::ResponsePlayerStatus(status) = res {
    //     println!("value: {:?}", status);
    // }

    // while let Ok(value) = receiver.try_recv() {
    //     if let AudioPlayerMessage::ResponsePlayerStatus(status) = value {
    //         println!("value: {:?}", status);
    //     } else {
    //         println!("foo");
    //     }
    // }

    // std::thread::spawn(move || loop {
    //     let playback_payload = PlaybackPayload {
    //         status: audio_player.get_status(),
    //         pos: audio_player.get_playback_position(),
    //         remain_buf: audio_player.get_remain_sample_buffer_sec(),
    //     };

    //     if let Err(e) = window.emit("update-playback-pos", playback_payload) {
    //         println!("{:?}", e);
    //     }

    //     std::thread::sleep(std::time::Duration::from_millis(200));
    // });
}

#[tauri::command]
pub async fn load_audio(
    state: State<'_, AudioPlayerChannelState>,
    audio_tag_id: String
) -> Result<f64, &'static str> {

    println!("got load audio command");
    // state.audio_cmd_sender.send("load_audio".to_string()).unwrap();
    state.audio_cmd_sender.send(AudioPlayerRequest::LoadAudio(
        LoadAudioMessage {
            audio_tag_id,
        }
    )).unwrap();

    let receivers_guard = state.audio_msg_receivers.lock().unwrap();
    let receiver = receivers_guard.get(&RequestType::LoadAudio).unwrap();
    let res = receiver.recv().unwrap();

    // while let Ok(value) = receiver.try_recv() {
    //     if let AudioPlayerMessage::ResponseAudioMeta(status) = value {
    //         println!("value: {:?}", status);
    //     }
    // }

    // match state.audio_player.add_audio(&audio_tag_id).await {
    //     Ok(content_length) => return Ok(content_length),
    //     Err(_) => return Err("tauri-plugin: failed to add audio"),
    // }

    Ok(0.0)
}

#[tauri::command]
pub fn start_audio(
    state: State<'_, AudioPlayerChannelState>
) -> Result<(), &'static str> {

    println!("got start audio command");

    // state.audio_cmd_sender.send("start_audio".to_string()).unwrap();
    state.audio_cmd_sender.send(AudioPlayerRequest::StartAudio).unwrap();

    // match state.audio_player.play() {
    //     Ok(())=> return Ok(()),
    //     Err(_) => return Err("tauri-plugin: failed to play audio"), 
    // }

    Ok(())
}

#[tauri::command]
pub fn stop_audio(
    state: State<'_, AudioPlayerChannelState>
) -> Result<(), &'static str> {

    println!("got stop audio command");

    // state.audio_cmd_sender.send("stop_audio".to_string()).unwrap();
    state.audio_cmd_sender.send(AudioPlayerRequest::StopAudio).unwrap();

    // state.audio_player.stop();

    Ok(())
}

#[tauri::command]
pub fn pause_audio(
    state: State<'_, AudioPlayerChannelState>
) -> Result<(), &'static str> {
    println!("got pause audio command");
    // state.audio_cmd_sender.send("pause_audio".to_string()).unwrap();
    state.audio_cmd_sender.send(AudioPlayerRequest::PauseAudio).unwrap();

    // match state.audio_player.pause() {
    //     Ok(_) => Ok(()),
    //     Err(_) => Err("failed to pause audio"),
    // }

    Ok(())
}

#[tauri::command]
pub async fn get_audio_tags(
    state: State<'_, AudioPlayerChannelState>,
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
