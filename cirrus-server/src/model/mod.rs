pub mod audio;
pub mod document;

pub use audio::*;

use mongodb::{Client, options::ClientOptions};
use super::settings::Settings;

pub async fn get_mongodb_client() -> Result<mongodb::Client, Box<dyn std::error::Error>> {
    let settings = Settings::new().unwrap();

    let client_options = ClientOptions::parse(settings.mongodb.address).await?;

    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub trait GetCollection<T> {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<T>;
}