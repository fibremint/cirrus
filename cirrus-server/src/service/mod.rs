mod tag;
mod data;
mod library;

use async_trait::async_trait;
use mongodb::Client;
use tonic::Status;

pub use data::AudioDataSvcImpl;
pub use tag::AudioTagSvcImpl;
pub use library::AudioLibrarySvcImpl;

#[async_trait]
trait GetMongoClient {
    async fn create_db_client(&self) -> Result<Client, Status>;
}
