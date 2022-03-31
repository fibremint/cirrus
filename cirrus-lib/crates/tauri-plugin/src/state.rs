use cirrus_client_lib::audio::{AudioPlayer, AudioPlayerWrapper};

pub struct AppState {
    pub audio_player: AudioPlayerWrapper,
}

impl AppState {
    pub fn new() -> Self {       
        Self {
            audio_player: AudioPlayerWrapper::new(),
        }
    }
}
