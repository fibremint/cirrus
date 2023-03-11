use std::{path::PathBuf, sync::{Arc, mpsc, Mutex}, collections::HashMap};
use cirrus_client_core::{AudioPlayer, audio::{AudioPlayerMessage, AudioPlayerRequest}};
use crossbeam_channel::{Receiver, Sender};
// use cirrus_client_core::AudioPlayer2;

use super::settings::Settings;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    // pub audio_player: AudioPlayer,
    // pub audio_player2: AudioPlayer2,
    pub settings: Settings,
}

impl AppState {
    pub fn new(res_root_path: &PathBuf, config_path_str: &str) -> Result<Self, anyhow::Error> {
        let config_path = res_root_path.join(config_path_str);
        let settings = Settings::new(&config_path).unwrap();

        let mut audio_player = AudioPlayer::new(&settings.server.grpc_endpoint)?;

        if settings.tls.use_tls {
            let cert_path = res_root_path.join(&settings.tls.cert_path);

            // audio_player.load_cert(
            //     &cert_path,
            //     &settings.tls.domain_name,
            // )?;
        }

        // let audio_player2 = AudioPlayer2 {};

        Ok(Self {
            audio_player: Arc::new(audio_player),
            // audio_player: audio_player,
            // audio_player2,
            settings
        })
    }
}

pub struct PluginState {
    pub audio_player: Arc<Mutex<AudioPlayer>>,
    pub settings: Settings,
}

impl PluginState {
    pub fn new(
        res_root_path: &PathBuf, 
        config_path_str: &str
    ) -> Result<Self, anyhow::Error> {
        let config_path = res_root_path.join(config_path_str);
        let settings = Settings::new(&config_path)?;

        let mut audio_player = AudioPlayer::new(&settings.server.grpc_endpoint)?;

        Ok(Self {
            audio_player: Arc::new(Mutex::new(audio_player)),
            settings,
        })
    }
}

pub struct AudioPlayerState {
    inner: Arc<Mutex<AudioPlayer>>,
}

impl AudioPlayerState {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: Arc::new(
                Mutex::new(
                    AudioPlayer::new("http://localhost:50000")?
                )
            ),
        })
    }
}

pub struct AudioPlayerChannelState {
    // pub audio_cmd_sender: Mutex<mpsc::Sender<String>>,
    pub audio_cmd_sender: Sender<AudioPlayerRequest>,

    // pub msg_rx: Mutex<mpsc::Receiver<AudioPlayerMessage>>,

    // pub receivers: Vec<Receiver<AudioPlayerMessage>>,
    pub audio_msg_receivers: HashMap<String, Receiver<AudioPlayerMessage>>,
}