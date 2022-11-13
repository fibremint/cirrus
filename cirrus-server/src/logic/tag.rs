
use std::{path::Path, fs::File};

use aiff::reader::AiffReader;
use bson::oid::ObjectId;
use chrono::{Utc, TimeZone};
use cirrus_protobuf::api::AudioTagRes;

use crate::{model::{self, dto, crud}, util};

pub struct AudioTag {
    crud_audio_tag: crud::AudioTag,
}

impl Default for AudioTag {
    fn default() -> Self {
        Self { 
            crud_audio_tag: Default::default(),
        }
    }
}

impl AudioTag {
    pub async fn list_audio_tags(
        &self,
        db: mongodb::Client,
        max_item_num: u64,
        page: u64,
    ) -> Result<Vec<AudioTagRes>, anyhow::Error> {
        let get_all_res = self.crud_audio_tag
            .page
            .get_paginated(
                db.clone(), 
                max_item_num as i64, 
                page
            ).await?;
        // let get_all_res = model::audio::AudioTag::get_all(mongodb_client.clone(), max_item_num as i64, page).await;

        let res = get_all_res
            .iter()
            .map(|item| AudioTagRes {
                id: item.id.as_ref().unwrap().to_string(),
                artist: item.artist.as_ref().unwrap().to_string(),
                genre: item.genre.as_ref().unwrap().to_string(),
                title: item.title.as_ref().unwrap().to_string(),
            })
            .collect::<Vec<_>>();

        Ok(res)
    }
}