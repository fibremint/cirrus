use std::sync::Arc;

use tauri::async_runtime::{RwLock, Mutex};

use cirrus_client_lib::audio::AudioPlayer;

pub struct AppState {
    // audio_player: Arc<RwLock<AudioPlayer>>,
    pub audio_player: Arc<RwLock<AudioPlayer>>,
    // pub audio_player: Arc<Mutex<AudioPlayer>>,
    // pub audio_player: AudioPlayer,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            audio_player: Arc::new(RwLock::new(AudioPlayer::new())),
            // audio_player: AudioPlayer::new(),
        }
    }
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}