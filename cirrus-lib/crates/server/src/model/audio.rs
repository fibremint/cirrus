use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc};

use crate::model::{GetCollection, document};

pub struct AudioLibrary {}

impl GetCollection<document::AudioLibrary> for AudioLibrary {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<document::AudioLibrary> {
        mongodb_client
            .database("cirrus")
            .collection::<document::AudioLibrary>("libraries")
    }
}

impl AudioLibrary {
    pub async fn create(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        mongodb_client: mongodb::Client,
        doc: document::AudioLibrary
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    // pub async fn get_count(
    //     mongodb_client: mongodb::Client
    // ) -> usize {
    //     let collection = Self::get_collection(mongodb_client.clone());

    //     let document_count = collection.cou
    // }

    pub async fn get_all(
        mongodb_client: mongodb::Client,
    ) -> Vec<document::AudioLibrary> {
        let collection = Self::get_collection(mongodb_client.clone());

        let find_res = collection.find(None, None).await.unwrap();
        // let docs: Vec<_> = find_res.map(|item| item.unwrap()).collect();
        // let docs: Vec<Result<document::AudioLibrary, mongodb::error::Error>> = find_res.collect().await;
        // docs

        find_res.try_collect().await.unwrap_or_else(|_| vec![])

        // let doc_collect = find_res.collect::<document::AudioLibrary>();
        // let doc_collect = find_res.collect::<Vec<Result<document::AudioLibrary, mongodb::error::Error>>>();
    }

    pub async fn get_by_path(
        mongodb_client: mongodb::Client,
        path: &str
    ) -> Result<Option<document::AudioLibrary>, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": path
        };
        let find_res = collection.find_one(filter, None).await.unwrap();
        
        Ok(find_res)
    }

    pub async fn delete_by_path(
        mongodb_client: mongodb::Client,
        path: &str
    ) -> mongodb::results::DeleteResult {
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": path
        };

        let delete_res = collection.delete_one(filter, None).await.unwrap();

        delete_res
    }
}