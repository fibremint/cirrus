use std::path::Path;

use async_trait::async_trait;
use bson::{oid::ObjectId, Document};
use futures::{lock::MutexGuard, TryStreamExt};
use mongodb::{self, bson::doc, results::{UpdateResult, DeleteResult, InsertManyResult, InsertOneResult}, error::Error, IndexModel, options::{IndexOptions, InsertManyOptions}};

use crate::{
    util, 
    model::{GetCollection, dto, document}
};

use super::{CrudMany, PathOperation, CrudSingle};

pub struct AudioLibraryRoot {
    pub single: CrudSingle<dto::AudioLibrary>,
    pub many: CrudMany<dto::AudioLibrary>,
    pub path: PathOperation<dto::AudioLibrary>
}

impl GetCollection<dto::AudioLibrary> for AudioLibraryRoot {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        db.database("cirrus").collection::<dto::AudioLibrary>("library_roots")
    }
}

impl Default for AudioLibraryRoot {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection),
        }
    }
}

// #[async_trait]
// impl CrudSingle for AudioLibraryRoot {
//     type Document = dto::AudioLibrary;

//     async fn create(
//         db: mongodb::Client,
//         doc: Self::Document
//     ) -> Result<InsertOneResult, anyhow::Error> {
//         let create_res = Self::get_collection(db)
//             .insert_one(doc, None)
//             .await?;

//         Ok(create_res)
//     }

//     async fn update(
//         db: mongodb::Client,
//         id: &ObjectId,
//         doc: Self::Document
//     ) -> Result<UpdateResult, anyhow::Error> {
//         let query = document::query_single_id(id);
//         let update = document::update_doc(doc)
//     }
// }

// #[async_trait]
// impl CrudMany for AudioLibraryRoot {
//     type Document = dto::AudioLibrary;

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

//     async fn update_many(
//         db: mongodb::Client, 
//         docs: &Vec<Self::Document>
//     ) -> Vec<Result<UpdateResult, anyhow::Error>> {
//         unimplemented!()
//     }

//     async fn delete_many(
//         db: mongodb::Client, 
//         docs: &Vec<Self::Document>
//     ) -> Result<DeleteResult, anyhow::Error> {
//         unimplemented!()
//     }
// }

// #[async_trait]
// impl PathOperation for AudioLibraryRoot {
//     type Document = dto::AudioLibrary;

//     async fn get_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let path = util::path::path_to_materialized(path);

//         let filter = document::path::create_find_by_path_filter(&path);

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
//         unimplemented!()
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
//         let query = document::path::create_find_by_path_filter(&doc.materialized_path);
//         let update = document::time::create_update_timestamp(timestamp);

//         let update_res = Self::get_collection(db)
//             .update_one(query, update, None)
//             .await?;

//         Ok(update_res)
//     }

//     async fn check_exists_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<bool, anyhow::Error> {
//         let path = util::path::path_to_materialized(path);

//         let filter = document::path::create_find_by_path_filter(&path);

//         let find_res = Self::get_collection(db)
//             .find_one(filter, None)
//             .await?;

//         Ok(find_res.is_some())
//     }
// }


pub struct AudioLibrary {
    pub single: CrudSingle<dto::AudioLibrary>,
    pub many: CrudMany<dto::AudioLibrary>,
    pub path: PathOperation<dto::AudioLibrary>
}

// pub enum AudioLibrary {
//     Single(CrudSingle<dto::AudioLibrary>),
//     Many(CrudMany<dto::AudioLibrary>),
// }

impl GetCollection<dto::AudioLibrary> for AudioLibrary {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        db.database("cirrus").collection::<dto::AudioLibrary>("libraries")
    }
}

impl Default for AudioLibrary {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection),
            many: CrudMany::new(Self::get_collection),
            path: PathOperation::new(Self::get_collection),
        }
    }
}

// #[async_trait]
// impl CrudSingle for AudioLibrary {
//     type Document = dto::AudioLibrary;

//     async fn create(
//         db: mongodb::Client,
//         doc: Self::Document
//     ) -> Result<InsertOneResult, anyhow::Error> {
//         let create_res = Self::get_collection(db)
//             .insert_one(doc, None)
//             .await?;

//         Ok(create_res)
//     }

//     async fn update(
//         db: mongodb::Client,
//         doc: Self::Document
//     ) -> Result<UpdateResult, anyhow::Error> {
//         todo!()
//     }

//     async fn delete(
//         db: mongodb::Client,
//         id: ObjectId,
//     ) -> Result<DeleteResult, anyhow::Error> {
//         let query = document::query_single_id(&id);

//         let delete_res = Self::get_collection(db)
//             .delete_one(query, None)
//             .await?;

//         Ok(delete_res)
//     }
// }

// #[async_trait]
// impl CrudMany for AudioLibrary {
//     type Document = dto::AudioLibrary;

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
//         unimplemented!()
//     }

//     async fn update_many(
//         db: mongodb::Client, 
//         docs: &Vec<Self::Document>
//     ) -> Vec<Result<UpdateResult, anyhow::Error>> {
//         todo!()
//         // let mut update_results = vec![];

//         // for doc in docs {
//         //     let query = document::create_find_id(&doc.id);

//         //     let serialized_doc = mongodb::bson::to_document(&doc).unwrap();

//         //     let update = doc! {
//         //         "$set": serialized_doc,
//         //     };

//         //     let update_res = Self::get_collection(db)
//         //         .update_one(query, update, None)
//         //         .await;

//         //     update_results.push(update_res);
//         // }

//         // update_results
//     }

//     async fn delete_many(
//         db: mongodb::Client, 
//         docs: &Vec<Self::Document>
//     ) -> Result<DeleteResult, anyhow::Error> {
//         todo!()
//     }
// }

// #[async_trait]
// impl PathOperation for AudioLibrary {
//     type Document = dto::AudioLibrary;

//     async fn get_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<Vec<Self::Document>, anyhow::Error> {
//         let path = util::path::path_to_materialized(path);

//         let filter = document::path::create_find_by_path_filter(&path);

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
//         let filter = document::path::create_find_by_path_filter(path);

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
//         let filter = document::path::create_find_by_path_filter(&mat_path);

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
//         let query = document::path::create_find_by_path_filter(&doc.materialized_path);
//         let update = document::time::create_update_timestamp(timestamp);

//         let update_res = Self::get_collection(db)
//             .update_one(query, update, None)
//             .await?;

//         Ok(update_res)
//     }

//     async fn check_exists_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<bool, anyhow::Error> {
//         todo!()
//     }
// }

impl AudioLibrary {
//     pub async fn create(
//         mongodb_client: mongodb::Client,
//         doc: dto::audio::AudioLibrary
//     ) -> Result<mongodb::results::InsertOneResult, Box<dyn std::error::Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let insert_res = collection.insert_one(doc, None).await.unwrap();
        
//         Ok(insert_res)
//     }

//     pub async fn create_many(
//         mongodb_client: mongodb::Client,
//         doc: Vec<dto::audio::AudioLibrary>,
//     ) -> Result<mongodb::results::InsertManyResult, Box<dyn std::error::Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let insert_res = collection.insert_many(doc, None).await.unwrap();
        
//         Ok(insert_res)
//     }

//     pub async fn get_by_path(
//         mongodb_client: mongodb::Client,
//         path: &Path
//     ) -> Result<Vec<dto::audio::AudioLibrary>, Box<dyn std::error::Error>> {
//         let path = util::path::path_to_materialized(path);
//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "path": { "$regex": format!("^{}", path) }
//         };
        
//         let find_res = collection.find(filter, None).await.unwrap();

//         Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))

//     }

//     pub async fn get_by_materialized_path(
//         mongodb_client: mongodb::Client,
//         path: &str
//     ) -> Result<Vec<dto::audio::AudioLibrary>, Box<dyn std::error::Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "path": { "$regex": format!("^{}", path) }
//         };
        
//         let find_res = collection.find(filter, None).await.unwrap();

//         Ok(find_res.try_collect().await.unwrap_or_else(|_| vec![]))
//     }

//     pub async fn delete_by_path(
//         mongodb_client: mongodb::Client,
//         path: &Path
//     ) -> Result<DeleteResult, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());
       
//         let path = util::path::path_to_materialized(path);
//         let filter = doc! {
//             "path": { "$regex": format!("^{}", path) }
//         };

//         let delete_res = collection.delete_many(filter, None).await?;

//         Ok(delete_res)
//     }

//     pub async fn update_modified_timestamp(
//         mongodb_client: mongodb::Client,
//         doc_id: &str,
//         modified_timestamp: i64,        
//     ) -> Result<UpdateResult, Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let query = doc! {
//             "_id": doc_id
//         };

//         let update = doc! {
//             "$set": {"modified_timestamp": modified_timestamp}
//         };

//         collection.update_one(query, update, None).await
//     }

//     pub async fn update_self(
//         mongodb_client: mongodb::Client,
//         documents: &Vec<dto::audio::AudioLibrary>,
//     ) -> Vec<Result<UpdateResult, Error>> {
//         let collection = Self::get_collection(mongodb_client.clone());
//         let mut update_results: Vec<Result<UpdateResult, Error>> = vec![];

//         for document in documents.iter() {
//             let query = doc! {
//                 "_id": document.id.clone(),
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
}


// impl AudioLibraryRoot {
//     pub async fn create(
//         mongodb_client: mongodb::Client,
//         doc: dto::audio::AudioLibrary
//     ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         collection.insert_one(doc, None).await        
//     }

//     pub async fn get_all(
//         mongodb_client: mongodb::Client,
//     ) -> Vec<dto::audio::AudioLibrary> {
//         let collection = Self::get_collection(mongodb_client.clone());

//         let find_res = collection.find(None, None).await.unwrap();

//         find_res.try_collect().await.unwrap_or_else(|_| vec![])
//     }

//     pub async fn check_exists_by_path(
//         mongodb_client: mongodb::Client,
//         path: &Path,
//     ) -> bool {
//         let collection = Self::get_collection(mongodb_client.clone());
//         let path = path.to_str().unwrap();

//         let filter = doc! {
//             "_id": path
//         };

//         let find_res = collection.find_one(filter, None).await.unwrap();
//         println!("clre findres: {:?}", find_res);

//         find_res.is_some()
//     }

//     pub async fn get_by_path(
//         mongodb_client: mongodb::Client,
//         path: &Path
//     ) -> Result<Option<dto::audio::AudioLibrary>, Box<dyn std::error::Error>> {
//         let path = util::path::path_to_materialized(path);
//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "path": { "$regex": format!("^{}", path) }
//         };
        
//         let find_res = collection.find_one(filter, None).await.unwrap();

//         Ok(find_res)
//     }

//     pub async fn delete_by_path(
//         mongodb_client: mongodb::Client,
//         path: &Path
//     ) -> mongodb::results::DeleteResult {
//         let path = util::path::path_to_materialized(path);

//         let collection = Self::get_collection(mongodb_client.clone());

//         let filter = doc! {
//             "path": { "$regex": format!("^{}", path) }
//         };

//         let delete_res = collection.delete_one(filter, None).await.unwrap();

//         delete_res
//     }
// }
