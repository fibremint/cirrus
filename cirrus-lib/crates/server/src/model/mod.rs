use mongodb::{Client, options::ClientOptions};

pub async fn get_mongodb_handle() -> Result<mongodb::Database, Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    let client = Client::with_options(client_options)?;
    let db = client.database("cirrus");

    Ok(db)
}