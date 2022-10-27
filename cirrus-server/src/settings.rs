use std::env;
use config::{Config, File, ConfigError};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Server {
    pub listen_address: String,
    pub listen_port: u32
}

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct MongoDB {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub server: Server,
    pub mongodb: MongoDB
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let server_config_path = current_dir.join("configs/server.toml");

        let s = Config::builder()
            .add_source(File::with_name(server_config_path.to_str().unwrap()))
            .build()?;

        s.try_deserialize()
    }
}