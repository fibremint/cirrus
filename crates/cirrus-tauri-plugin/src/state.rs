use std::{sync::Arc, path::PathBuf};
use cirrus_client_core::AudioPlayer;

use super::settings::Settings;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    //pub settings: Settings,
}

impl AppState {
    pub fn new() -> Result<Self, anyhow::Error> {
        //let config_path = res_root_path.join(config_path_str);
        //let settings = Settings::new(&config_path).unwrap();

        let mut audio_player = AudioPlayer::new("grpcs://127.0.0.1:50000");

        //if settings.tls.use_tls {
        //    let cert_path = res_root_path.join(&settings.tls.cert_path);
//
//            audio_player.load_cert(
//                &cert_path,
//                &settings.tls.domain_name,
//            )?;
//        }

        Ok(Self {
            audio_player: Arc::new(audio_player),
            //settings
        })
    }
}
