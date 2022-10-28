use std::{env, path::PathBuf};
use config::{Config, File, ConfigError};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Server {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub server: Server,
}

impl Settings {
    pub fn new(config_path: &PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(config_path.to_str().unwrap()))
            .build()?;

        s.try_deserialize()
    }
}