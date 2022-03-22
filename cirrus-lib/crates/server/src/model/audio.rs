use std::path::Path;

use bson::oid::ObjectId;
use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc, results::UpdateResult, error::Error, IndexModel, options::IndexOptions};

use crate::{
    util, 
    model::{GetCollection, document}
};

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
        mongodb_client: mongodb::Client,
        doc: document::AudioLibrary
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn create_many(
        mongodb_client: mongodb::Client,
        doc: Vec<document::AudioLibrary>,
    ) -> Result<mongodb::results::InsertManyResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_many(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn get_by_path(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<Vec<document::AudioLibrary>, Box<dyn std::error::Error>> {
        // let path = util::path::path_to_materialized(path.to_str().unwrap());
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
    ) -> Result<Vec<document::AudioLibrary>, Box<dyn std::error::Error>> {
        // let path = util::path::path_to_materialized(path.to_str().unwrap());
        // let path = util::path::path_to_materialized(path);
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
    ) -> mongodb::results::DeleteResult {
        let path = util::path::path_to_materialized(path);

        let collection = Self::get_collection(mongodb_client.clone());

        // let filter = doc! {
        //     "path": path
        // };

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };

        let delete_res = collection.delete_many(filter, None).await.unwrap();

        delete_res
    }

    // pub async fn get_none_refered_files(
    //     mongodb_client: mongodb::Client,
    // ) {
    //     let collection = Self::get_collection(mongodb_client.clone());

    //     collection.
    // }
}

pub struct AudioLibraryRoot {}

impl GetCollection<document::AudioLibrary> for AudioLibraryRoot {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<document::AudioLibrary> {
        mongodb_client
            .database("cirrus")
            .collection::<document::AudioLibrary>("library_roots")
    }
}

impl AudioLibraryRoot {
    pub async fn create(
        mongodb_client: mongodb::Client,
        doc: document::AudioLibrary
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    // pub async fn create_many(
    //     mongodb_client: mongodb::Client,
    //     doc: Vec<document::AudioLibrary>,
    // ) -> Result<mongodb::results::InsertManyResult, Box<dyn std::error::Error>> {
    //     let collection = Self::get_collection(mongodb_client.clone());

    //     let insert_res = collection.insert_many(doc, None).await.unwrap();
        
    //     Ok(insert_res)
    // }

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
    ) -> Result<Option<document::AudioLibrary>, Box<dyn std::error::Error>> {
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

        // let filter = doc! {
        //     "path": path
        // };

        let filter = doc! {
            "path": { "$regex": format!("^{}", path) }
        };

        let delete_res = collection.delete_one(filter, None).await.unwrap();

        delete_res
    }
}

pub struct AudioFile {}

impl GetCollection<document::AudioFile> for AudioFile {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<document::AudioFile> {
        mongodb_client
            .database("cirrus")
            .collection::<document::AudioFile>("audio")
    }
}

impl AudioFile {
    pub async fn create(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        mongodb_client: mongodb::Client,
        doc: document::AudioFile
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn create_many(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        mongodb_client: mongodb::Client,
        doc: Vec<document::AudioFile>,
    ) -> Result<mongodb::results::InsertManyResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        // let doc_keys: Vec<_> = doc.iter()
        //     .map(|item| doc! { "_id": item.id.unwrap() } )
        //     .collect();

        // let options = IndexOptions::builder().unique(true).build();
        // let models = IndexModel::builder()
        //     .keys(doc_keys)
        //     .options(options)
        //     .build();

        // collection.index

        let insert_res = collection.insert_many(doc, None).await.unwrap();
        
        Ok(insert_res)
    }

    pub async fn get_by_library_path(
        mongodb_client: mongodb::Client,
        path: &Path,
        filter_none_refer: bool,
    ) -> Vec<document::AudioFile> {
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

        // let filter = doc! {
        //     "parent_path": { "$regex": format!("^{}", path) }
        // };

            //     // let filter = doc! {
    //     //     "$and": [{
    //     //         "parent_path": { "$regex": format!("^{}", path) }
    //     //     }, {
    //     //         "audio_tag_refer": null,
    //     //     }]
    //     // };

        let find_res = collection.find(filter, None).await.unwrap();

        find_res.try_collect().await.unwrap_or_else(|_| vec![])
    }

    pub async fn set_audio_tag_refer(
        mongodb_client: mongodb::Client,
        doc_id: &i64,
        tag_id: &ObjectId,
    ) -> Result<UpdateResult, Error> {
        let collection = Self::get_collection(mongodb_client.clone());

        // let doc_id = doc_id.to_string();
        let query = doc! {
            "_id": doc_id
        };

        let update = doc! {
            "$set": {"audio_tag_refer": tag_id}
        };

        collection.update_one(query, update, None).await
    }

    // pub async fn get_by_library_path_and_none_referer(
    //     mongodb_client: mongodb::Client,
    //     path: &Path,
    // ) -> Vec<document::AudioFile> {
    //     let collection = Self::get_collection(mongodb_client.clone());
    //     let path = util::path::path_to_materialized(path);

    //     // let filter = doc! {
    //     //     "parent_path": { "$regex": format!("^{}", path) }
    //     // };

    //     // let filter = doc! {
    //     //     "$and": [{
    //     //         "parent_path": { "$regex": format!("^{}", path) }
    //     //     }, {
    //     //         "audio_tag_refer": null,
    //     //     }]
    //     // };

    //     let find_res = collection.find(filter, None).await.unwrap();

    //     find_res.try_collect().await.unwrap_or_else(|_| vec![])
    // }

    // pub async fn get_by_materialized_library_path(
    //     mongodb_client: mongodb::Client,
    //     path: &Path,
    // ) -> Vec<document::AudioFile> {
    //     let collection = Self::get_collection(mongodb_client.clone());
    //     // let path = util::path::path_to_materialized(path);

    //     let filter = doc! {
    //         "parent_path": { "$regex": format!("^{}", path) }
    //     };

    //     let find_res = collection.find(filter, None).await.unwrap();

    //     find_res.try_collect().await.unwrap_or_else(|_| vec![])
    // }
}

pub struct AudioTag {}

impl GetCollection<document::AudioTag> for AudioTag {
    fn get_collection(mongodb_client: mongodb::Client) -> mongodb::Collection<document::AudioTag> {
        mongodb_client
            .database("cirrus")
            .collection::<document::AudioTag>("audio_tag")
    }
}

impl AudioTag {
    pub async fn create(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        mongodb_client: mongodb::Client,
        doc: document::AudioTag
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
        let collection = Self::get_collection(mongodb_client.clone());

        let insert_res = collection.insert_one(doc, None).await.unwrap();
        
        Ok(insert_res)
    }
}