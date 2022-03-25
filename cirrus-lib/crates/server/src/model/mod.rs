pub mod audio;
pub mod document;

pub use audio::*;

use mongodb::{Client, options::ClientOptions};

pub async fn get_mongodb_client() -> Result<mongodb::Client, Box<dyn std::error::Error>> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub trait GetCollection<T> {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<T>;
}