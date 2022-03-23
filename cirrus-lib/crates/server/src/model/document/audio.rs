use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::path::Path;

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Deserialize, Serialize, Debug)]
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
    pub fn check_modified(&self, local_path: &Path) -> bool {
        let local_path = util::path::replace_with_common_separator(local_path.to_str().unwrap());
        // assert!(local_path == self.id);
        // if local_path != self.id {
        //     return Err()
        // }

        let local_timestamp = util::path::get_timestamp(Path::new(&local_path));

        local_timestamp != self.modified_timestamp
    }
}
// #[derive(Deserialize, Serialize, Debug)]
// pub struct AudioLibrary {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub path: String,
//     // #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
//     // pub modified: DateTime<Utc>,
//     pub modified_timestamp: i64
// }

// pub struct AudioLibrary {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub path: String,
//     // #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
//     // pub modified: DateTime<Utc>,
//     pub modified_timestamp: i64
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AudioLibrary {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub path: String,
//     pub modified_timestamp: i64,
//     pub audio_files: Option<Vec<AudioFileSimple>>,
//     pub parent: Option<Rc<AudioLibrary>>,
// }

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

// impl Hash for AudioFile {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.modified_timestamp.hash(state);
//         // self.parent_path.hash(state);
//         self.filename.hash(state);
//     }
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AudioFileSimple {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub modified_timestamp: i64,
//     pub audio_filename: String,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AudioLibraryDetail {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     id: Option<ObjectId>,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AudioFile {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub metadata: Option<AudioFileMetadata>,
//     pub modified_timestamp: i64,
//     pub path: String,
// }

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