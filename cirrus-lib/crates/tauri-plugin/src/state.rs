use std::sync::Arc;

use cirrus_client_lib::AudioPlayer;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            audio_player: Arc::new(AudioPlayer::new())
        }
    }
}
