use crate::request;

pub struct AudioSourceMetadata {
    pub bit_rate: u32,
    pub sample_rate: u32,
    pub channels: usize,
    pub sample_frames: usize,
}

pub struct AudioSource {
    pub server_address: String,
    pub id: String,
    pub metadata: AudioSourceMetadata,
}

impl AudioSource {
    pub async fn new(server_address: &str, audio_tag_id: &str) -> Result<Self, anyhow::Error> {
        let server_address = server_address.to_string();

        let metadata_res = request::get_audio_meta(
            server_address.clone(), 
            audio_tag_id
        ).await.unwrap().into_inner();

        let metadata = AudioSourceMetadata {
            bit_rate: metadata_res.bit_rate,
            sample_rate: metadata_res.sample_rate,
            channels: metadata_res.channels as usize,
            sample_frames: metadata_res.sample_frames as usize,
        };

        Ok(Self {
            server_address,
            id: audio_tag_id.to_string(),
            metadata,
        })
    }
}
