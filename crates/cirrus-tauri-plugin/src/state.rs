use std::{sync::Arc, path::PathBuf};
use cirrus_client_core::AudioPlayer;

use super::settings::Settings;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    pub settings: Settings,
}

impl AppState {
    pub fn new(config_path: &PathBuf, cert_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let settings = Settings::new(config_path).unwrap();

        let mut audio_player = AudioPlayer::new(settings.server.grpc_endpoint);

        if settings.tls.tls {
            audio_player.load_cert(
                cert_path,
                &settings.tls.domain_name,
            )?;
        }

        Ok(Self {
            audio_player: Arc::new(audio_player),
            settings: Settings::new(config_path).unwrap()
        })
    }
}
