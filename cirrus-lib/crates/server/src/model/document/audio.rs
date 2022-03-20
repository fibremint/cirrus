use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioLibrary {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub path: String,
    pub modified_timestamp: i64,
    pub contents: Option<Vec<FileMetadata>>,
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
pub struct FileMetadata {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub modified_timestamp: i64,
    pub filename: String,
}

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

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFile {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub metadata: Option<AudioFileMetadata>,
    pub modified_timestamp: i64,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFileMetadata {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFileMetadataPicture {
    pub description: String,
    pub mime_type: String,
    pub picture_type: String,
    pub data: Vec<u8>,
}