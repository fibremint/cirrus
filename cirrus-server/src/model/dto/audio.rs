use std::cmp::Eq;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use aiff::reader::AiffReader;
use chrono::{DateTime, Utc, TimeZone};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::util;

pub trait GetPathKey {
    fn get_mat_path_key() -> &'static str;
}

pub trait GetPathValue {
    fn get_mat_path_val(&self) -> &str;
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct AudioLibrary {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub materialized_path: String,
    pub os_path: String,
    pub modified_timestamp: i64,
}

impl AudioLibrary {
    pub fn new(path: &Path) -> Self {
        let os_path = util::path::replace_with_common_separator(path.to_str().unwrap());
        let materialized_path = util::path::path_to_materialized(&path);

        let modified_timestamp = util::path::get_timestamp(&path);
        
        Self {
            id: Some(mongodb::bson::oid::ObjectId::new()),
            materialized_path,
            os_path: os_path,
            modified_timestamp,
        }
    }

    pub fn check_modified(&self) -> bool {
        let local_timestamp = util::path::get_timestamp(Path::new(&self.os_path));

        local_timestamp != self.modified_timestamp
    }
}

impl GetPathKey for AudioLibrary {
    fn get_mat_path_key() -> &'static str {
        "materialized_path"
    }
}

impl GetPathValue for AudioLibrary {
    fn get_mat_path_val(&self) -> &str {
        &self.materialized_path
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioFile {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub modified_timestamp: i64,
    pub parent_path: String,
    pub filename: String,
    pub audio_tag_refer: Option<ObjectId>,
}

impl AudioFile {
    pub fn new(path: &Path) -> Self {
        let parent_path = path.parent().unwrap();
        let parent_path = util::path::path_to_materialized(&parent_path);

        let modified_timestamp = util::path::get_timestamp(path);

        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        Self {
            id: Some(mongodb::bson::oid::ObjectId::new()),
            modified_timestamp,
            parent_path,
            filename,
            audio_tag_refer: None,
        }
    }

    pub fn check_modified(&self) -> bool {
        let local_timestamp = util::path::get_timestamp(&self.get_os_path());

        local_timestamp != self.modified_timestamp
    }

    pub fn get_os_path(&self) -> PathBuf {
        let parent_path = util::path::materialized_to_path(&self.parent_path);
        let parent_path = Path::new(&parent_path);
        
        parent_path.join(&self.filename)
    }

    pub fn update_modified_timestamp(&mut self) {
        let path = self.get_os_path();
        
        self.modified_timestamp = util::path::get_timestamp(&path);
    }
}

impl GetPathKey for AudioFile {
    fn get_mat_path_key() -> &'static str {
        "parent_path"
    }
}

impl GetPathValue for AudioFile {
    fn get_mat_path_val(&self) -> &str {
        &self.parent_path
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AudioTag {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub property_hash: Option<i64>,

    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub date_recorded: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub date_released: Option<DateTime<Utc>>,
    pub disc: Option<u32>,
    pub duration: Option<u32>,
    pub genre: Option<String>,
    pub title: Option<String>,
    pub total_discs: Option<u32>,
    pub total_tracks: Option<u32>,
    pub track: Option<u32>,
    pub year: Option<i32>,
}

impl AudioTag {
    pub fn new(
        id: Option<ObjectId>,
        parent_path: &str,
        filename: &str,
    ) -> Result<Self, anyhow::Error> {
        let mut audio_file_path = Path::new(parent_path).to_path_buf();
        audio_file_path.push(filename);

        let audio_file = File::open(audio_file_path).unwrap();
        let mut aiff = AiffReader::new(audio_file);
        // aiff.read().unwrap();
        aiff.parse().unwrap();

        // let id3v2 = aiff.read_chunk::<aiff::chunks::ID3v2Chunk>(true, false, aiff::ids::AIFF).unwrap();

        // let id = Some(ObjectId::new());
        let id = match id {
            Some(id) => Some(id),
            None => Some(ObjectId::new())
        };

        let mut id_id3v2 = aiff::ids::ID3.to_vec();
        id_id3v2.push(0);

        let _audio_metadata = if let Some(id3v2) = aiff.read_chunk::<aiff::chunks::ID3v2Chunk>(true, false, &id_id3v2) {
            let date_recorded = match id3v2.tag.date_recorded() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            let date_released = match id3v2.tag.date_released() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            // let pictures: Vec<_> = id3v2.tag.pictures()
            //     .into_iter()
            //     .map(|item| document::audio::AudioFileMetadataPicture {
            //         description: item.description.clone(),
            //         mime_type: item.mime_type.clone(),
            //         picture_type: item.picture_type.to_string(),
            //         data: item.data.to_owned(),
            //     })
            //     .collect();

            let artist = match id3v2.tag.artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album = match id3v2.tag.album() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album_artist = match id3v2.tag.album_artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let genre = match id3v2.tag.genre() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let title = match id3v2.tag.title() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let mut audio_tag = Self {
                id,
                property_hash: None,
                artist: artist,
                album: album,
                album_artist: album_artist,
                date_recorded,
                date_released,
                disc: id3v2.tag.disc(),
                duration: id3v2.tag.duration(),
                genre: genre,
                // pictures: pictures,
                title: title,
                total_discs: id3v2.tag.total_discs(),
                total_tracks: id3v2.tag.total_tracks(),
                track: id3v2.tag.track(),
                year: id3v2.tag.year(),
            };

            audio_tag.property_hash = Some(util::hash::get_hashed_value(&audio_tag));

            return Ok(audio_tag)

        } else {
            return Ok(Self {
                id,
                property_hash: None,
                title: Some(filename.clone().to_owned()),
                ..Default::default()
            });
        };
    }
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
