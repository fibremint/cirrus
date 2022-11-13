use std::path::Path;

use async_trait::async_trait;
use bson::{oid::ObjectId, Document};
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::results::{InsertOneResult, InsertManyResult, DeleteResult, UpdateResult};

mod file;
mod library;
mod tag;

pub use library::{AudioLibraryRoot, AudioLibrary};
pub use file::AudioFile;
use serde::{Serialize, de::DeserializeOwned};
pub use tag::AudioTag;

use crate::util;

use super::{GetCollection, document, dto::{self, GetPathKey, GetPathValue}};


pub struct Pagination<T> {
    object: Option<T>,
    col_fn: fn(mongodb::Client) -> mongodb::Collection<T>,
}

impl<T> Pagination<T> 
// where
//     T: Serialize,
{
    fn new(col_fn: fn(mongodb::Client) -> mongodb::Collection<T>) -> Self {
        Self {
            object: None,
            col_fn,
        }
    }
}

impl<T> Pagination<T> 
where
    T: Serialize + DeserializeOwned + Sync + Send + Unpin
{
    pub async fn get_paginated(
        &self,
        db: mongodb::Client,
        limit: i64,
        page: u64
    ) -> Result<Vec<T>, anyhow::Error> {
        let options = mongodb::options::FindOptions::builder()
            .limit(limit)
            .skip(limit as u64 * (page-1))
            .build();

        let find_res = (self.col_fn)(db)
            .find(None, options)
            .await?;

        let found_docs = find_res
            .try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(found_docs)
    }
}


// #[async_trait]
// pub trait Pagination {
//     async fn get_paginated<T>(
//         db: mongodb::Client,
//         limit: i64,
//         page: u64
//     ) -> Vec<T>;
// }

pub struct CrudMany<T> 
// where
//     T: Serialize,
{
    object: Option<T>,
    col_fn: fn(mongodb::Client) -> mongodb::Collection<T>,
}

impl<T> CrudMany<T> 
// where
//     T: Serialize,
{
    fn new(col_fn: fn(mongodb::Client) -> mongodb::Collection<T>) -> Self {
        Self {
            object: None,
            col_fn,
        }
    }
}

// impl<T: Serialize> Default for CrudMany<T> {
//     fn default() -> Self {
//         Self { object: None }
//     }
// }

// impl<T> CrudMany<T> {
//     pub fn new() -> Self {
//         Self { object: T }
//     }
// }

// impl<T: Serialize> Default for CrudMany<T> {
//     fn default() -> Self {
//         Self { object: T }
//     }
// }

impl<T> CrudMany<T> 
where
    T: Serialize + DeserializeOwned + Sync + Send + Unpin
{
    pub async fn create_many(
        &self,
        db: mongodb::Client,
        docs: Vec<T>
    ) -> Result<InsertManyResult, anyhow::Error> {
        let insert_res = (self.col_fn)(db)
            .insert_many(docs, None)
            .await?;

        Ok(insert_res)
    }

    pub async fn get_all(
        &self,
        db: mongodb::Client,
    ) -> Result<Vec<T>, anyhow::Error> {
        // let col = db.database("cirrus").collection::<dto::AudioLibrary>("library_roots");

        let find_res = (self.col_fn)(db)
            .find(None, None)
            .await?;

        let found_docs = find_res.try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(found_docs)
    }

    pub async fn get_many(
        &self,
        db: mongodb::Client,
        ids: Option<&Vec<ObjectId>>,
        query: Option<Document>,
    ) -> Result<Vec<T>, anyhow::Error> {
        let query = match query {
            Some(query) => query,
            None => {
                if ids.is_none() {
                    return Err(anyhow::anyhow!("object id is not set"));
                }

                document::query_many_id(ids.unwrap())
            },
        };

        // let query = document::query_many_id(ids);

        let find_res = (self.col_fn)(db)
            .find(query, None)
            .await?;

        let found_docs = find_res.try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(found_docs)
    }
    
    pub async fn update_many(
        &self,
        db: mongodb::Client,
        docs: &Vec<T>
    ) -> Vec<Result<UpdateResult, anyhow::Error>> {
        todo!()
    }

    // async fn delete_many(
    //     db: mongodb::Client,
    //     docs: &Vec<Self::Document>
    // ) -> Result<DeleteResult, anyhow::Error>;

    pub async fn delete_many(
        &self,
        db: mongodb::Client,
        ids: &Vec<ObjectId>
    ) -> Result<DeleteResult, anyhow::Error> {
        let query = document::query_many_id(ids);

        let delete_res = (self.col_fn)(db)
            .delete_many(
                query, 
                None
            ).await?;

        Ok(delete_res)
    }
}

// #[async_trait]

pub struct CrudSingle<T> {
    object: Option<T>,
    col_fn: fn(mongodb::Client) -> mongodb::Collection<T>
}

impl<T> CrudSingle<T> {
    fn new(col_fn: fn(mongodb::Client) -> mongodb::Collection<T>) -> Self {
        Self {
            object: None,
            col_fn,
        }
    }
}

impl<T> CrudSingle<T> 
where
    T: Serialize + DeserializeOwned + Sync + Send + Unpin 
{
    pub async fn create(
        &self,
        db: mongodb::Client,
        doc: &T
    ) -> Result<InsertOneResult, anyhow::Error> {
        let insert_res = (self.col_fn)(db)
            .insert_one(
                doc, 
                None
            ).await?;

        Ok(insert_res)
    }

    pub async fn get(
        &self,
        db: mongodb::Client,
        id: Option<&ObjectId>,
        query: Option<Document>,
    ) -> Result<Option<T>, anyhow::Error> {
        let query = match query {
                Some(query) => query,
                None => {
                    if id.is_none() {
                        return Err(anyhow::anyhow!("object id is not set"));
                    }

                    document::query_single_id(id.unwrap())
                },
            };

        // let query = document::query_single_id(id);

        let found_doc = (self.col_fn)(db)
            .find_one(query, None)
            .await?;

        Ok(found_doc)
    }

    pub async fn update(
        &self,
        db: mongodb::Client,
        id: &ObjectId,
        doc: &T,
    ) -> Result<UpdateResult, anyhow::Error> {
        let query = document::query_single_id(id);
        let update = document::update_doc(doc)?;

        let update_res = (self.col_fn)(db)
            .update_one(query, update, None)
            .await?;

        Ok(update_res)
    }

    pub async fn delete(
        &self,
        db: mongodb::Client,
        id: &ObjectId,
    ) -> Result<DeleteResult, anyhow::Error> {
        let query = document::query_single_id(id);

        let delete_res = (self.col_fn)(db)
            .delete_one(
                query,
                None
            ).await?;

        Ok(delete_res)
    }
}

pub struct PathOperation<T> {
    object: Option<T>,
    col_fn: fn(mongodb::Client) -> mongodb::Collection<T>,
}

impl<T> PathOperation<T> {
    fn new(col_fn: fn(mongodb::Client) -> mongodb::Collection<T>) -> Self {
        Self {
            object: None,
            col_fn,
        }
    }
}

impl<T> PathOperation<T> 
where
    T: Serialize + DeserializeOwned + Sync + Send + Unpin + GetPathKey + GetPathValue,
{
    pub async fn get_by_path(
        &self,
        db: mongodb::Client,
        path: &Path
    ) -> Result<Vec<T>, anyhow::Error> {
        let path = util::path::path_to_materialized(path);
        let filter = document::path::query_path(T::get_mat_path_key(), &path);

        let find_res = (self.col_fn)(db)
            .find(filter, None)
            .await?;

        let found_docs = find_res.try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(found_docs)
    }

    pub async fn get_by_materialized_path(
        &self,
        db: mongodb::Client,
        path: &str
    ) -> Result<Vec<T>, anyhow::Error> {
        // let (path_key, path_val) = doc.get_mat_path();
        let query = document::path::query_path(T::get_mat_path_key(), path);

        let find_res = (self.col_fn)(db)
            .find(query, None)
            .await?;

        let found_docs = find_res.try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(found_docs)
    }

    pub async fn delete_by_path(
        &self,
        db: mongodb::Client,
        path: &Path
    ) -> Result<DeleteResult, anyhow::Error> {
        let mat_path = util::path::path_to_materialized(path);
        let filter = document::path::query_path(T::get_mat_path_key(), &mat_path);

        let delete_res = (self.col_fn)(db)
            .delete_many(filter, None)
            .await?;

        Ok(delete_res)
    }

    pub async fn update_modified_timestamp(
        &self,
        db: mongodb::Client,
        doc: &T,
        timestamp: i64
    ) -> Result<UpdateResult, anyhow::Error> {
        // let (path_key, path_val) = doc.get_mat_path();
        let query = document::path::query_path(T::get_mat_path_key(), doc.get_mat_path_val());

        let update = document::time::create_update_timestamp(timestamp);

        let update_res = (self.col_fn)(db)
            .update_one(query, update, None)
            .await?;

        Ok(update_res)
    }

    pub async fn check_exists_by_path(
        &self,
        db: mongodb::Client,
        path: &Path
    ) -> Result<bool, anyhow::Error> {
        let path = util::path::path_to_materialized(path);
        let filter = document::path::query_path(T::get_mat_path_key(), &path);

        let find_res = (self.col_fn)(db)
            .find_one(filter, None)
            .await?;

        Ok(find_res.is_some())
    }
}

// #[async_trait]
// pub trait PathOperation {
//     type Document;

//     async fn get_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<Vec<Self::Document>, anyhow::Error>;

//     async fn get_by_materialized_path(
//         db: mongodb::Client,
//         path: &str
//     ) -> Result<Vec<Self::Document>, anyhow::Error>;

//     async fn delete_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<DeleteResult, anyhow::Error>;

//     async fn update_modified_timestamp(
//         db: mongodb::Client,
//         doc: &Self::Document,
//         timestamp: i64
//     ) -> Result<UpdateResult, anyhow::Error>;

//     async fn check_exists_by_path(
//         db: mongodb::Client,
//         path: &Path
//     ) -> Result<bool, anyhow::Error>;
// }
