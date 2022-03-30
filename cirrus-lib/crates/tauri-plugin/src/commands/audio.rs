use std::borrow::Borrow;

use http::Response;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

use cirrus_client_lib::request;
use cirrus_grpc::api::AudioTagRes;

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

#[tauri::command]
pub async fn get_audio_tags(
    // state: State<'_, AppState>,
    items_per_page: u64,
    page: u32,
) -> Result<Vec<AudioTagRes>, String> {
    println!("got get-audio-tags commnad");

    let audio_tags = request::get_audio_tags(items_per_page, page as u64).await.unwrap();

    Ok(audio_tags)

    // let mut audio_tags = state.audio_tags.lock().await;
    // // let res: Vec<AudioTagRes> = Vec::new();

    // match audio_tags.get(&page) {
    //     Some(_) => (),
    //     None => {
    //         let response = request::get_audio_tags(items_per_page, page as u64).await.unwrap();
    //         let response: Vec<_> = response
    //             .into_iter()
    //             // .filter_map(|item| item.ok())
    //             .collect();

    //         audio_tags.insert(page, response);
    //     },
    // }

    // let res = audio_tags.get(&page).unwrap().clone();
    // // let mut res: Vec<TagResponse> = Vec::new();
    // // res.push(TagResponse {
    // //     artist: String::from("foo"),
    // //     genre: String::from("bar"),
    // //     title: String::from("baz"),
    // // });

    // Ok(res)
    // // Ok(audio_tags.get(&page).clone())

    // // audio_tags.i
}