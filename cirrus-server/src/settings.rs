use std::env;
use config::{Config, File, ConfigError};
use serde_derive::{Serialize, Deserialize};

const CONFIG_PATH: &'static str = "configs/cirrus/server.toml";

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Server {
    pub listen_address: String,
    pub listen_port: u32,
    pub tls: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct MongoDB {
    pub address: String,
}


#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct AudioSamleFramePacket {
    pub sample_rate: u32,
    pub len: u32,
}

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub server: Server,
    pub mongodb: MongoDB,
    pub audio_sample_frame_packet: AudioSamleFramePacket,
}

impl Settings {
    pub fn get() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let server_config_path = current_dir.join(CONFIG_PATH);
        
        let s = Config::builder()
            .add_source(File::from(server_config_path))
            .build()?;

        s.try_deserialize()
    }
}