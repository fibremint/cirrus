pub mod audio;
pub mod document;

use mongodb::{Client, options::ClientOptions};

use crate::settings::Settings;

pub async fn create_db_client() -> Result<mongodb::Client, anyhow::Error> {
    let settings = Settings::get()?;
    let client_options = ClientOptions::parse(settings.mongodb.address).await?;

    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub trait GetCollection<T> {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<T>;
}