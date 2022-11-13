use std::path::Path;

use async_trait::async_trait;
use bson::{oid::ObjectId};
use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc, results::{UpdateResult, DeleteResult, InsertManyResult, InsertOneResult}, error::Error, IndexModel, options::{IndexOptions, InsertManyOptions}};

use crate::{
    util, 
    model::{GetCollection, dto, document}
};

use super::{CrudMany, PathOperation, CrudSingle};

pub struct AudioFile {
    pub single: CrudSingle<dto::AudioFile>,
    pub many: CrudMany<dto::AudioFile>,
    pub path: PathOperation<dto::AudioFile>,
}

impl GetCollection<dto::AudioFile> for AudioFile {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioFile> {
        db.database("cirrus").collection::<dto::AudioFile>("audio")
    }
}

impl Default for AudioFile {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection)
        }
    }
}


// #[async_trait]
// impl CrudSingle for AudioFile {
//     type Document = dto::AudioFile;

//     async fn create(
//         db: mongodb::Client,
//         doc: Self::Document
//     ) -> Result<InsertOneResult, anyhow::Error> {
//         unimplemented!()
//     }

//     async fn update(
//         db: mongodb::Client,
//         id: &ObjectId,
//         doc: bson::Document,
//     ) -> Result<UpdateResult, anyhow::Error> {
//         unimplemented!()
//     }

//     async fn delete(
//         db: mongodb::Client,
//         id: ObjectId,
//     ) -> Result<DeleteResult, anyhow::Error> {
//         unimplemented!()
//     }
// }

// #[async_trait]
// impl CrudMany for AudioFile {
//     type Document = dto::AudioFile;

//     async fn create_many(
//         db: mongodb::Client, 
//         docs: Vec<Self::Document>
//     ) -> Result<InsertManyResult, anyhow::Error> {
//         let insert_res = Self::get_collection(db)
//             .insert_many(docs, None)
//             .await?;

//         Ok(insert_res)
//     }

//     async fn get_all(
//         db: mongodb::Client,
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let find_res = Self::get_collection(db)
//             .find(None, None)
//             .await?;

//         let found_docs = find_res.try_collect()
//             .await
//             .unwrap_or_else(|_| vec![]);

//         Ok(found_docs)
//     }

//     async fn get_many(
//         db: mongodb::Client,
//         ids: &Vec<ObjectId>,
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let query = document::query_many_id(ids);

//         let find_res = Self::get_collection(db)
//             .find(query, None)
//             .await?;

//         let found_docs = find_res.try_collect()
//             .await
//             .unwrap_or_else(|_| vec![]);

//         Ok(found_docs)
//     }

//     async fn update_many(
//         db: mongodb::Client, 
//         docs: &Vec<Self::Document>
//     ) -> Vec<Result<UpdateResult, anyhow::Error>> {
//         unimplemented!()
//     }

//     async fn delete_many(
//         db: mongodb::Client, 
//         ids: &Vec<ObjectId>,
//     ) -> Result<DeleteResult, anyhow::Error> {
//         let query = document::query_many_id(ids);

//         let delete_res = Self::get_collection(db)
//             .delete_many(query, None)
//             .await?;

//         // let collection = Self::get_collection(mongodb_client.clone());
        
//         // let delete_ids: Vec<_> = target.iter()
//         //     .map(|item| item.id.unwrap())
//         //     .collect();

//         // let query = doc! {
//         //     "_id": {
//         //         "$in": delete_ids,
//         //     }
//         // };

//         // let delete_res = collection.delete_many(query, None).await?;

//         Ok(delete_res)
//     }
// }

// #[async_trait]
// impl PathOperation for AudioFile {
//     type Document = dto::AudioFile;

//     async fn get_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let path = util::path::path_to_materialized(path);

//         let filter = document::path::query_path(&path);

//         let find_res = Self::get_collection(db)
//             .find(filter, None)
//             .await?;

//         let found_docs = find_res.try_collect()
//             .await
//             .unwrap_or_else(|_| vec![]);

//         Ok(found_docs)
//     }

//     async fn get_by_materialized_path(
//         db: mongodb::Client,
//         path: &str
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let filter = document::path::query_path(&path);

//         let find_res = Self::get_collection(db)
//             .find(filter, None)
//             .await?;

//         let found_docs = find_res.try_collect()
//             .await
//             .unwrap_or_else(|_| vec![]);

//         Ok(found_docs)
//     }

//     async fn delete_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<DeleteResult, anyhow::Error> {
//         let mat_path = util::path::path_to_materialized(path);
//         let filter = document::path::query_path(&mat_path);

//         let delete_res = Self::get_collection(db)
//             .delete_many(filter, None)
//             .await?;

//         Ok(delete_res)
//     }

//     async fn update_modified_timestamp(
//         db: mongodb::Client,
//         doc: &Self::Document,
//         timestamp: i64
//     ) -> Result<UpdateResult, anyhow::Error> {
//         unimplemented!()
//         // let query = document::path::create_find_by_path_filter(&doc.materialized_path);
//         // let update = document::time::create_update_timestamp(timestamp);

//         // let update_res = Self::get_collection(db)
//         //     .update_one(query, update, None)
//         //     .await?;

//         // Ok(update_res)
//     }

//     async fn check_exists_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<bool, anyhow::Error> {
//         let path = util::path::path_to_materialized(path);

//         let filter = document::path::query_path(&path);

//         let find_res = Self::get_collection(db)
//             .find_one(filter, None)
//             .await?;

//         Ok(find_res.is_some())
//     }
// }

// impl AudioFile {
//     pub async fn create(
//         mongodb_client: mongodb::Client,
//         doc: dto::audio::AudioFile
//     ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let insert_res = collection.insert_one(doc, None).await.unwrap();
        
//         Ok(insert_res)
//     }

//     pub async fn create_many(
//         mongodb_client: mongodb::Client,
//         doc: &Vec<dto::audio::AudioFile>,
//     ) -> Result<mongodb::results::InsertManyResult, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let insert_res = collection.insert_many(doc, None).await.unwrap();

//         Ok(insert_res)
//     }

//     pub async fn find_by_audio_tag_id(
//         mongodb_client: mongodb::Client,
//         audio_tag_id: ObjectId
//     ) -> Result<Option<dto::audio::AudioFile>, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "audio_tag_refer": audio_tag_id
//         };

//         let find_res = collection.find_one(filter, None).await?;

//         Ok(find_res)
//     }

//     // pub async fn get_self_by_library_path(
//     //     mongodb_client: mongodb::Client,
//     //     path: &Path,
//     //     filter_none_refer: bool,
//     // ) -> Result<Vec<dto::audio::AudioFile>, mongodb::error::Error> {
//     //     let collection = Self::get_collection(mongodb_client.clone());
//     //     let path = util::path::path_to_materialized(path);

//     //     let filter = {
//     //         if filter_none_refer {
//     //             doc! {
//     //                 "$and": [{
//     //                     "parent_path": { "$regex": format!("^{}", path) }
//     //                 }, {
//     //                     "audio_tag_refer": null,
//     //                 }]
//     //             }
//     //         } else {
//     //             doc! {
//     //                 "parent_path": { "$regex": format!("^{}", path) }
//     //             }
//     //         }
//     //     };

//     //     let find_res = collection.find(filter, None).await?;

//     //     Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
//     // }

//     pub async fn set_audio_tag_refer(
//         mongodb_client: mongodb::Client,
//         doc_id: &ObjectId,
//         tag_id: &ObjectId,        
//     ) -> Result<UpdateResult, Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let query = doc! {
//             "_id": doc_id
//         };

//         let update = doc! {
//             "$set": {"audio_tag_refer": tag_id}
//         };

//         collection.update_one(query, update, None).await

//     }

//     // pub async fn delete_by_selfs(
//     //     mongodb_client: mongodb::Client,
//     //     target: &Vec<dto::audio::AudioFile>,
//     // ) -> Result<DeleteResult, mongodb::error::Error> {
//     //     let collection = Self::get_collection(mongodb_client.clone());
        
//     //     let delete_ids: Vec<_> = target.iter()
//     //         .map(|item| item.id.unwrap())
//     //         .collect();

//     //     let query = doc! {
//     //         "_id": {
//     //             "$in": delete_ids,
//     //         }
//     //     };

//     //     let delete_res = collection.delete_many(query, None).await?;

//     //     Ok(delete_res)
//     // }

//     pub async fn update_self(
//         mongodb_client: mongodb::Client,
//         documents: &Vec<dto::audio::AudioFile>,
//     ) -> Vec<Result<UpdateResult, Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());
//         let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

//         for document in documents.iter() {
//             let query = doc! {
//                 "_id": document.id,
//             };

//             let serialized_document = mongodb::bson::to_document(&document).unwrap();
//             // document.
//             let update = doc! {
//                 "$set": serialized_document,
//             };

//             let update_res = collection.update_one(query, update, None).await;
//             update_results.push(update_res);
//         }

//         update_results
//     }
// }
