
use cirrus_protobuf::api::AudioTagRes;

use crate::model;

pub struct AudioTag {}

impl AudioTag {
    pub async fn list_audio_tags(
        mongodb_client: mongodb::Client,
        max_item_num: u64,
        page: u64,
    ) -> Result<Vec<AudioTagRes>, String> {
        let get_all_res = model::audio::AudioTag::get_all(mongodb_client.clone(), max_item_num as i64, page).await;

        let res: Vec<_> = get_all_res
            .iter()
            .map(|item| AudioTagRes {
                id: item.id.as_ref().unwrap().to_string(),
                artist: item.artist.as_ref().unwrap().to_string(),
                genre: item.genre.as_ref().unwrap().to_string(),
                title: item.title.as_ref().unwrap().to_string(),
            })
            .collect();

        Ok(res)
    }
}