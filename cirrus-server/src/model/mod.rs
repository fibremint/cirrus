pub mod dto;
pub mod document;
pub mod crud;

use mongodb::{Client, options::ClientOptions};

use crate::settings::Settings;

pub async fn create_db_client() -> Result<mongodb::Client, anyhow::Error> {
    let settings = Settings::get()?;
    let client_options = ClientOptions::parse(settings.mongodb.address).await?;

    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub trait GetCollection<T> {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<T>;
}