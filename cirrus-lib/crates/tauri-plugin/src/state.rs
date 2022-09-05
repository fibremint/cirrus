// use cirrus_client_lib::audio::{AudioPlayer, AudioPlayerWrapper};
use cirrus_client_lib::audio::AudioPlayer;

pub struct AppState {
    pub audio_player: AudioPlayer,
}

impl AppState {
    pub async fn new() -> Self {       
        Self {
            audio_player: AudioPlayer::new().await,
        }
    }
}
