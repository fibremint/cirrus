use std::{
    fs::File,
    path::Path,
};

use aiff::reader::AiffReader;
use chrono::DateTime;
use cirrus_grpc::api::{
    AudioDataRes, AudioMetaRes
};
// use futures::{TryStreamExt};
use mongodb::{bson::{Document, doc}, options::FindOptions};
use tokio::sync::{Mutex, MutexGuard};

use crate::model::{self, document};

pub struct AudioFile {}

impl AudioFile {
    pub fn read_meta(
        filepath: &str
    ) -> Result<AudioMetaRes, String> {
        // let file = File::open(filepath)?;
        let file = match File::open(filepath) {
            Ok(file) => file,
            Err(err) => return Err(String::from("failed to load file")),
        };

        let mut reader = AiffReader::new(file);
        // reader.read().unwrap();
        match reader.read() {
            Ok(_) => (),
            Err(err) => match err {
                aiff::chunks::ChunkError::InvalidID(id) => return Err(String::from("invalid id")),
                aiff::chunks::ChunkError::InvalidFormType(id) => return Err(String::from("invalid form type")),
                aiff::chunks::ChunkError::InvalidID3Version(ver) => return Err(String::from("invalid id3 version")),
                aiff::chunks::ChunkError::InvalidSize(exp, actual) => return Err(format!("invalid size, expected: {}, actual: {}", exp, actual)),
                aiff::chunks::ChunkError::InvalidData(msg) => return Err(msg.to_string()),
            },
        }

        let common = reader.form().as_ref().unwrap().common().as_ref().unwrap();
        let sound = reader.form().as_ref().unwrap().sound().as_ref().unwrap();

        Ok(AudioMetaRes {
            bit_rate: common.bit_rate as u32,
            block_size: sound.block_size,
            channels: sound.block_size,
            offset: sound.offset,
            sample_frames: common.num_sample_frames,
            sample_rate: common.sample_rate as u32,
            size: sound.size as u32,
        })
    }

    pub fn read_data(
        filepath: &str, 
        byte_start: usize, 
        byte_end: usize
    ) -> Result<AudioDataRes, String> {
        // let file = File::open(filepath)?;
        let file = match File::open(filepath) {
            Ok(file) => file,
            Err(err) => return Err(String::from("failed to load file")),
        };
    
        let mut reader = AiffReader::new(file);
        // reader.read().unwrap();
        match reader.read() {
            Ok(_) => (),
            Err(err) => match err {
                aiff::chunks::ChunkError::InvalidID(id) => return Err(String::from("invalid id")),
                aiff::chunks::ChunkError::InvalidFormType(id) => return Err(String::from("invalid form type")),
                aiff::chunks::ChunkError::InvalidID3Version(ver) => return Err(String::from("invalid id3 version")),
                aiff::chunks::ChunkError::InvalidSize(exp, actual) => return Err(format!("invalid size, expected: {}, actual: {}", exp, actual)),
                aiff::chunks::ChunkError::InvalidData(msg) => return Err(msg.to_string()),
            },
        }
    
        let reader_form_ref = reader.form().as_ref().unwrap();
        let data = reader_form_ref.sound().as_ref().unwrap();
        let mut audio_data_part = Vec::<u8>::new();
        audio_data_part.extend_from_slice(&data.sound_data[4*byte_start..4*byte_end]);
    
        Ok(AudioDataRes {
            content: audio_data_part
        })
    }
}

pub struct AudioLibrary {}

impl AudioLibrary {
    // * path not exist -> return not found
    // * path is added already -> return added already 
    pub async fn add_audio_library(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        // db_handle: mongodb::Database,
        // db_handle: &mongodb::Database,
        mongodb_client: mongodb::Client,

        // db: Mutex<mongodb::Database>,
        
        path: &Path
    ) -> Result<(), String> {
        // use futures::StreamExt;

        if !path.exists() {
            return Err(String::from("not exists"))
        }

        let path_str = path.to_str().unwrap();

        if let Some(res) = model::AudioLibrary::get_by_path(mongodb_client.clone(), path_str).await.unwrap() {
            return Err(format!("path '{}' already exists", path_str))
        }

        let path_modified_time = path.metadata().unwrap().modified().unwrap();
        let path_modified_time = DateTime::<chrono::Utc>::from(path_modified_time);

        let path_doc = document::AudioLibrary {
            id: None,
            path: String::from(path_str),
            modified: path_modified_time,
        };

        let create_res = model::AudioLibrary::create(mongodb_client.clone(), path_doc).await;
        // let collection = db_handle.collection::<Document>("libraries");
        // let filter = doc! {
        //     "path": path.to_str(),
        // };
        // let mut cursor = collection.find(filter, None).await.unwrap();

        // let count = cursor.count().await;

        // if count > 0 {
        //     return Err(String::from("path is already added"))
        // }

        // let doc = doc! {
        //     "path": path.to_str(),
        //     "modified": path_modified_time.to_string(),
        // };
        // collection.insert_one(doc, None).await.unwrap();

        Ok(())
    }

    pub async fn remove_audio_library(
        // db_handle: MutexGuard<'_, mongodb::Database>,
        // db_handle: mongodb::Database,
        mongodb_client: mongodb::Client,

        // db: Mutex<mongodb::Database>,
        path: &Path
    ) -> Result<mongodb::results::DeleteResult, String> {
        use futures::StreamExt;

        let path_str = path.to_str().unwrap();

        if let None = model::AudioLibrary::get_by_path(mongodb_client.clone(), path_str).await.unwrap() {
            return Err(format!("path '{}' is not registered", path_str))
        }

        let delete_res = model::AudioLibrary::delete_by_path(mongodb_client.clone(), path_str).await;

        // let collection = db_handle.collection::<Document>("libraries");
        // let filter = doc! {
        //     "path": path.to_str(),
        // };
        // let cursor = collection.find(filter, None).await.unwrap();

        // let count = cursor.count().await;
        // if count == 0 {
        //     return Err(String::from("path is not registered"))
        // }

        // let query = doc! {
        //     "path": path.to_str(),
        // };

        // let delete_res = match collection.delete_one(query, None).await {
        //     Ok(res) => res,
        //     Err(err) => return Err(err.to_string()),
        // };

        Ok(delete_res)
    }

    pub async fn refresh_audio_library(
        // db_handle: MutexGuard<'_, mongodb::Database>
        // db_handle: mongodb::Database,
        mongodb_client: mongodb::Client,

        // db: Mutex<mongodb::Database>,
    ) -> Result<(), String> {
        let paths = model::AudioLibrary::get_all(mongodb_client.clone()).await;

        println!("paths: {:?}", paths);
        // collection; libraries - audio library root
        //             libraries-detail - actual contents (sub_dirs, audio_files)

        // filter updated path (by paths' modified datetime)
        // use futures::TryStreamExt;

        // use futures::StreamExt;

        // let collection = db_handle.collection::<Document>("libraries");
        // let cursor = collection.find(None, None).await.unwrap();

        // // let paths: Vec<Document> = cursor.try_collect().await;
        // let paths = cursor.collect::<Vec<Result<Document, mongodb::error::Error>>>().await;

        // println!("paths: {:?}", paths);

        Ok(())
    }

    async fn create_audio_library(
        db_handle: MutexGuard<'_, mongodb::Database>
    ) -> Result<(), String> {
        todo!()
    }
}