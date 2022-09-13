use std::sync::Arc;

use cirrus_client_lib::audio::AudioPlayer;

pub struct AppState {
    // pub audio_player: AudioPlayer,
    pub audio_player: Arc<AudioPlayer>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            // audio_player: AudioPlayer::new(),
            audio_player: Arc::new(AudioPlayer::new())
        }
    }
}
