use crate::request;

pub struct AudioSource {
    pub id: String,
    pub metadata: AudioSourceMetadata,
}

impl AudioSource {
    pub async fn new(audio_tag_id: &str) -> Result<Self, anyhow::Error> {
        let metadata_res = request::get_audio_meta(audio_tag_id).await.unwrap().into_inner();

        let metadata = AudioSourceMetadata {
            bit_rate: metadata_res.bit_rate,
            sample_rate: metadata_res.sample_rate,
            channels: metadata_res.channels as usize,
            content_bytes: metadata_res.size,
            sample_frames: metadata_res.sample_frames as usize,
        };

        Ok(Self {
            id: audio_tag_id.to_string(),
            metadata,
        })
    }
}

pub struct AudioSourceMetadata {
    pub bit_rate: u32,
    pub sample_rate: u32,
    pub channels: usize,
    pub content_bytes: u32,
    pub sample_frames: usize,
}

// impl AudioSourceMetadata {
//     pub fn new(bit_rate: u32, sample_rate: u32, channels: usize, content_bytes: u32, sample_frames: usize) -> Self {
//         Self {
//             bit_rate,
//             sample_rate,
//             channels,
//             content_bytes,
//             sample_frames,
//         }
//     }
// }