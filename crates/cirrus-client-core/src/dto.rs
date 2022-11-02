use crate::request;

pub struct AudioSource {
    pub server_address: String,
    pub id: String,
    pub length: f64,
    pub channels: usize,
    pub bit_rate: u32,
    pub sample_rate: usize,
    pub packet_dur: f64,
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
            length: metadata_res.content_length,
            channels: metadata_res.channels.try_into().unwrap(),
            bit_rate: metadata_res.orig_bit_rate,
            sample_rate: metadata_res.orig_sample_rate as usize,
            packet_dur: metadata_res.packet_dur,
        })

    }
}
