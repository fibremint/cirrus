use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::ops::{DerefMut, Deref};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct AudioLibrary {
    // #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    // pub id: Option<ObjectId>,
    // #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")]
    pub id: String,
    pub path: Option<String>,
    pub modified_timestamp: i64,
    // pub contents: Option<Vec<FileMetadata>>,
}

impl AudioLibrary {
    pub fn create_from_path(path: &Path) -> Self {
        let id = util::path::replace_with_common_separator(path.to_str().unwrap());
        let modified_timestamp = util::path::get_timestamp(&path);
        let path = util::path::path_to_materialized(&path);
        
        Self {
            id,
            path: Some(path),
            modified_timestamp,
        }
    }
    // pub fn check_modified(&self, local_path: &Path) -> bool {
    pub fn check_modified(&self) -> bool {
        // let local_path = util::path::replace_with_common_separator(local_path.to_str().unwrap());
        // assert!(local_path == self.id);
        // if local_path != self.id {
        //     return Err()
        // }

        let local_timestamp = util::path::get_timestamp(Path::new(&self.id));

        local_timestamp != self.modified_timestamp
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFile {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    // pub id: Option<i64>,
    pub modified_timestamp: i64,
    pub parent_path: String,
    pub filename: String,
    pub audio_tag_refer: Option<ObjectId>,
}

// impl Deref for AudioFile {
//     type Target;

//     fn deref(&self) -> &Self::Target {
//         todo!()
//     }
// }

// impl DerefMut for AudioFile {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         todo!()
//     }
// }

impl AudioFile {
    pub fn create_from_path(path: &Path) -> Self {
        let parent_path = path.parent().unwrap();
        let parent_path_materialized = util::path::path_to_materialized(&parent_path);
        let modified_timestamp = util::path::get_timestamp(path);
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        Self {
            id: Some(mongodb::bson::oid::ObjectId::new()),
            modified_timestamp,
            parent_path: parent_path_materialized,
            filename,
            audio_tag_refer: None,
        }
    }

    pub fn check_modified(&self) -> bool {
        let local_timestamp = util::path::get_timestamp(&self.get_path());

        local_timestamp != self.modified_timestamp
    }

    pub fn get_path(&self) -> PathBuf {
        let parent_path = util::path::materialized_to_path(&self.parent_path);
        let mut path = Path::new(&parent_path).to_path_buf();
        path.push(&self.filename);

        path
    }

    pub fn update_modified_timestamp(&mut self) {
        let path = self.get_path();
        
        self.modified_timestamp = util::path::get_timestamp(&path);
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AudioTag {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    // #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    // pub id: Option<i64>,
    // pub audio_file_refer: Option<i64>,
    pub property_hash: Option<i64>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none", with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    // #[serde_as(as = "Option<DurationSeconds<i64>>")]
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub date_recorded: Option<DateTime<Utc>>,
    // #[serde(skip_serializing_if = "Option::is_none", with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    // #[serde_as(as = "Option<DurationSeconds<i64>>")]
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub date_released: Option<DateTime<Utc>>,
    pub disc: Option<u32>,
    pub duration: Option<u32>,
    pub genre: Option<String>,
    // picture: Option<Vec<u8>>,
    pub pictures: Vec<AudioFileMetadataPicture>,
    pub title: Option<String>,
    pub total_discs: Option<u32>,
    pub total_tracks: Option<u32>,
    pub track: Option<u32>,
    pub year: Option<i32>,
}

impl Hash for AudioTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.artist.hash(state);
        self.album.hash(state);
        self.date_recorded.hash(state);
        self.date_released.hash(state);
        self.title.hash(state);
        self.year.hash(state);
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFileMetadataPicture {
    pub description: String,
    pub mime_type: String,
    pub picture_type: String,
    pub data: Vec<u8>,
}