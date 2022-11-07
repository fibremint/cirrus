use std::{sync::Arc, path::PathBuf};

use cirrus_client_core::AudioPlayer;

use super::settings::Settings;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    pub settings: Settings
}

impl AppState {
    pub fn new(config_path: &PathBuf) -> Self {
        Self {
            audio_player: Arc::new(AudioPlayer::new()),
            settings: Settings::new(config_path).unwrap()
        }
    }
}
