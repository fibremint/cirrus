use std::path::Path;

use async_trait::async_trait;
use bson::{oid::ObjectId};
use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc, results::{UpdateResult, DeleteResult, InsertManyResult}, error::Error, IndexModel, options::{IndexOptions, InsertManyOptions}};

use crate::{
    util, 
    model::{GetCollection, dto, document}
};

use super::{CrudMany, CrudSingle, PathOperation, Pagination};

pub struct AudioTag {
    pub single: CrudSingle<dto::AudioTag>,
    pub many: CrudMany<dto::AudioTag>,
    pub path: PathOperation<dto::AudioTag>,
    pub page: Pagination<dto::AudioTag>,
}

impl GetCollection<dto::AudioTag> for AudioTag {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioTag> {
        db.database("cirrus")
            .collection::<dto::AudioTag>("audio_tag")
    }
}

impl Default for AudioTag {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection),
            page: Pagination::new(Self::get_collection),
        }
    }
}

// #[async_trait]
// impl CrudMany for AudioTag {
//     type Document = dto::AudioTag;

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


// impl AudioTag {
//     pub async fn create(
//         mongodb_client: mongodb::Client,
//         doc: dto::audio::AudioTag
//     ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());
//         collection.insert_one(doc, None).await
//     }

//     pub async fn get_all(
//         mongodb_client: mongodb::Client,
//         limit: i64,
//         page: u64,
//     ) -> Vec<dto::audio::AudioTag> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let options = mongodb::options::FindOptions::builder()
//             .limit(limit)
//             .skip(limit as u64 * (page-1))
//             .build();

//         let find_res = collection.find(None, options).await.unwrap();

//         find_res.try_collect().await.unwrap_or_else(|_| vec![])
//     }

//     pub async fn get_by_ids(
//         mongodb_client: mongodb::Client,
//         ids: Vec<ObjectId>,
//     ) -> Result<Vec<dto::audio::AudioTag>, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "_id": {
//                 "$in": ids,
//             }
//         };

//         let find_res = collection.find(filter, None).await?;
        
//         Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
//     }

//     pub async fn delete_by_ids(
//         mongodb_client: mongodb::Client,
//         ids: &Vec<ObjectId>,
//     ) -> Result<DeleteResult, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let query = doc! {
//             "_id": {
//                 "$in": ids,
//             }
//         };

//         let delete_res = collection.delete_many(query, None).await?;

//         Ok(delete_res)
//     }

//     pub async fn update_self(
//         mongodb_client: mongodb::Client,
//         documents: &Vec<dto::audio::AudioTag>,
//     ) -> Vec<Result<UpdateResult, Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());
//         let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

//         for document in documents.iter() {
//             let query = doc! {
//                 "_id": document.id,
//             };

//             let serialized_document = mongodb::bson::to_document(&document).unwrap();

//             let update = doc! {
//                 "$set": serialized_document,
//             };

//             let update_res = collection.update_one(query, update, None).await;
//             update_results.push(update_res);
//         }

//         update_results
//     }
// }
