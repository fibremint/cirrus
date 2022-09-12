use cirrus_client_lib::audio::AudioPlayer;

pub struct AppState {
    pub audio_player: AudioPlayer,
}

impl AppState {
    pub fn new() -> Self {  
        Self {
            audio_player: AudioPlayer::new(),
        }
    }
}
