use crate::request;

pub struct AudioSource {
    pub server_address: String,
    pub id: String,
    pub length: f32,
    pub channels: usize,
    pub bit_rate: u32,
    pub sample_rate: usize,
}

impl AudioSource {
    pub async fn new(server_address: &str, audio_tag_id: &str) -> Result<Self, anyhow::Error> {
        let server_address = server_address.to_string();

        let metadata_res = request::get_audio_meta(
            server_address.clone(),
            audio_tag_id
        ).await.unwrap().into_inner();

        Ok(Self {
            server_address: server_address.to_string(),
            id: audio_tag_id.to_string(),
            length: metadata_res.length,
            channels: metadata_res.channels.try_into().unwrap(),
            bit_rate: metadata_res.bit_rate,
            sample_rate: metadata_res.sample_rate as usize,
        })

    }
}
