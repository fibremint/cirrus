use std::path::Path;

use bson::{oid::ObjectId};
use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc, results::{UpdateResult, DeleteResult}, error::Error, IndexModel, options::{IndexOptions, InsertManyOptions}};

use crate::{
    util, 
    model::{GetCollection, dto}
};

pub struct AudioLibrary {}

impl GetCollection<dto::AudioLibrary> for AudioLibrary {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        mongodb_client
            .database("cirrus")
            .collection::<dto::AudioLibrary>("libraries")
    }
}

impl AudioLibrary {
    pub async fn create(
        mongodb_client: mongodb::Client,
        doc: dto::AudioLibrary
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn create_many(
        mongodb_client: mongodb::Client,
        doc: Vec<dto::AudioLibrary>,
    ) -> Result<mongodb::results::InsertManyResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_many(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn get_by_path(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<Vec<dto::AudioLibrary>, Box<dyn std::error::Error>> {
        let path = util::path::path_to_materialized(path);
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };
        
        let find_res = collection.find(filter, None).await.unwrap();

        Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))

    }

    pub async fn get_by_materialized_path(
        mongodb_client: mongodb::Client,
        path: &str
    ) -> Result<Vec<dto::AudioLibrary>, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };
        
        let find_res = collection.find(filter, None).await.unwrap();

        Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
    }

    pub async fn delete_by_path(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());
       
        let path = util::path::path_to_materialized(path);
        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };

        let delete_res = collection.delete_many(filter, None).await?;

        Ok(delete_res)
    }

    pub async fn update_modified_timestamp(
        mongodb_client: mongodb::Client,
        doc_id: &str,
        modified_timestamp: i64,        
    ) -> Result<UpdateResult, Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let query = doc! {
            "_id": doc_id
        };

        let update = doc! {
            "$set": {"modified_timestamp": modified_timestamp}
        };

        collection.update_one(query, update, None).await
    }

    pub async fn update_self(
        mongodb_client: mongodb::Client,
        documents: &Vec<dto::AudioLibrary>,
    ) -> Vec<Result<UpdateResult, Error>> {
        let collection = Self::get_collection(mongodb_client.clone());
        let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

        for document in documents.iter() {
            let query = doc! {
                "_id": document.id.clone(),
            };

            let serialized_document = mongodb::bson::to_document(&document).unwrap();
            // document.
            let update = doc! {
                "$set": serialized_document,
            };

            let update_res = collection.update_one(query, update, None).await;
            update_results.push(update_res);
        }

        update_results
    }
}

pub struct AudioLibraryRoot {}

impl GetCollection<dto::AudioLibrary> for AudioLibraryRoot {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        mongodb_client
            .database("cirrus")
            .collection::<dto::AudioLibrary>("library_roots")
    }
}

impl AudioLibraryRoot {
    pub async fn create(
        mongodb_client: mongodb::Client,
        doc: dto::AudioLibrary
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        collection.insert_one(doc, None).await        
    }

    pub async fn get_all(
        mongodb_client: mongodb::Client,
    ) -> Vec<dto::AudioLibrary> {
        let collection = Self::get_collection(mongodb_client.clone());

        let find_res = collection.find(None, None).await.unwrap();

        find_res.try_collect().await.unwrap_or_else(|_| vec![])
    }

    pub async fn check_exists_by_path(
        mongodb_client: mongodb::Client,
        path: &Path,
    ) -> bool {
        let collection = Self::get_collection(mongodb_client.clone());
        let path = path.to_str().unwrap();

        let filter = doc! {
            "_id": path
        };

        let find_res = collection.find_one(filter, None).await.unwrap();
        println!("clre findres: {:?}", find_res);

        find_res.is_some()
    }

    pub async fn get_by_path(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<Option<dto::AudioLibrary>, Box<dyn std::error::Error>> {
        let path = util::path::path_to_materialized(path);
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };
        
        let find_res = collection.find_one(filter, None).await.unwrap();

        Ok(find_res)
    }

    pub async fn delete_by_path(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> mongodb::results::DeleteResult {
        let path = util::path::path_to_materialized(path);

        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };

        let delete_res = collection.delete_one(filter, None).await.unwrap();

        delete_res
    }
}

pub struct AudioFile {}

impl GetCollection<dto::AudioFile> for AudioFile {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<dto::AudioFile> {
        mongodb_client
            .database("cirrus")
            .collection::<dto::AudioFile>("audio")
    }
}

impl AudioFile {
    pub async fn create(
        mongodb_client: mongodb::Client,
        doc: dto::AudioFile
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn create_many(
        mongodb_client: mongodb::Client,
        doc: &Vec<dto::AudioFile>,
    ) -> Result<mongodb::results::InsertManyResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_many(doc, None).await.unwrap();

        Ok(insert_res)
    }

    pub async fn find_by_audio_tag_id(
        mongodb_client: mongodb::Client,
        audio_tag_id: ObjectId
    ) -> Result<Option<dto::AudioFile>, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "audio_tag_refer": audio_tag_id
        };

        let find_res = collection.find_one(filter, None).await?;

        Ok(find_res)
    }

    pub async fn get_self_by_library_path(
        mongodb_client: mongodb::Client,
        path: &Path,
        filter_none_refer: bool,
    ) -> Result<Vec<dto::AudioFile>, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());
        let path = util::path::path_to_materialized(path);

        let filter = {
            if filter_none_refer {
                doc! {
                    "$and": [{
                        "parent_path": { "$regex": format!("^{}", path) }
                    }, {
                        "audio_tag_refer": null,
                    }]
                }
            } else {
                doc! {
                    "parent_path": { "$regex": format!("^{}", path) }
                }
            }
        };

        let find_res = collection.find(filter, None).await?;

        Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
    }

    pub async fn set_audio_tag_refer(
        mongodb_client: mongodb::Client,
        doc_id: &ObjectId,
        tag_id: &ObjectId,        
    ) -> Result<UpdateResult, Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let query = doc! {
            "_id": doc_id
        };

        let update = doc! {
            "$set": {"audio_tag_refer": tag_id}
        };

        collection.update_one(query, update, None).await

    }

    pub async fn delete_by_selfs(
        mongodb_client: mongodb::Client,
        target: &Vec<dto::AudioFile>,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());
        
        let delete_ids: Vec<_> = target.iter()
            .map(|item| item.id.unwrap())
            .collect();

        let query = doc! {
            "_id": {
                "$in": delete_ids,
            }
        };

        let delete_res = collection.delete_many(query, None).await?;

        Ok(delete_res)
    }

    pub async fn update_self(
        mongodb_client: mongodb::Client,
        documents: &Vec<dto::AudioFile>,
    ) -> Vec<Result<UpdateResult, Error>> {
        let collection = Self::get_collection(mongodb_client.clone());
        let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

        for document in documents.iter() {
            let query = doc! {
                "_id": document.id,
            };

            let serialized_document = mongodb::bson::to_document(&document).unwrap();
            // document.
            let update = doc! {
                "$set": serialized_document,
            };

            let update_res = collection.update_one(query, update, None).await;
            update_results.push(update_res);
        }

        update_results
    }
}

pub struct AudioTag {}

impl GetCollection<dto::AudioTag> for AudioTag {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<dto::AudioTag> {
        mongodb_client
            .database("cirrus")
            .collection::<dto::AudioTag>("audio_tag")
    }
}

impl AudioTag {
    pub async fn create(
        mongodb_client: mongodb::Client,
        doc: dto::AudioTag
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());
        collection.insert_one(doc, None).await
    }

    pub async fn get_all(
        mongodb_client: mongodb::Client,
        limit: i64,
        page: u64,
    ) -> Vec<dto::AudioTag> {
        let collection = Self::get_collection(mongodb_client.clone());

        let options = mongodb::options::FindOptions::builder()
            .limit(limit)
            .skip(limit as u64 * (page-1))
            .build();

        let find_res = collection.find(None, options).await.unwrap();

        find_res.try_collect().await.unwrap_or_else(|_| vec![])
    }

    pub async fn get_by_ids(
        mongodb_client: mongodb::Client,
        ids: Vec<ObjectId>,
    ) -> Result<Vec<dto::AudioTag>, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let filter = doc! {
            "_id": {
                "$in": ids,
            }
        };

        let find_res = collection.find(filter, None).await?;
        
        Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
    }

    pub async fn delete_by_ids(
        mongodb_client: mongodb::Client,
        ids: &Vec<ObjectId>,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        let query = doc! {
            "_id": {
                "$in": ids,
            }
        };

        let delete_res = collection.delete_many(query, None).await?;

        Ok(delete_res)
    }

    pub async fn update_self(
        mongodb_client: mongodb::Client,
        documents: &Vec<dto::AudioTag>,
    ) -> Vec<Result<UpdateResult, Error>> {
        let collection = Self::get_collection(mongodb_client.clone());
        let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

        for document in documents.iter() {
            let query = doc! {
                "_id": document.id,
            };

            let serialized_document = mongodb::bson::to_document(&document).unwrap();

            let update = doc! {
                "$set": serialized_document,
            };

            let update_res = collection.update_one(query, update, None).await;
            update_results.push(update_res);
        }

        update_results
    }
}
