use tonic::{Request, Response};

use audio_proto::audio_svc_client::AudioSvcClient;
use audio_proto::{AudioDataReq, AudioDataRes, AudioMetaReq, AudioMetaRes};

pub mod audio_proto {
    tonic::include_proto!("audio");
}

pub async fn get_audio_meta(filepath: &str) -> Result<Response<AudioMetaRes>, Box<dyn std::error::Error>> {
    let mut client = AudioSvcClient::connect("http://[::1]:50000").await?;

    let request = Request::new({
        AudioMetaReq {
            filename: filepath.to_string()
        }
    });

    let response = client.get_meta(request).await?;

    // println!("response: {:?}", response);

    Ok(response)
}

pub async fn get_audio_data(filepath: &str, sample_pos_start: u32, sample_pos_end: u32) -> Result<Response<AudioDataRes>, Box<dyn std::error::Error>> {
    let mut client = AudioSvcClient::connect("http://[::1]:50000").await?;

    let request = Request::new({
        AudioDataReq {
            filename: filepath.to_string(),
            byte_start: sample_pos_start * 4,
            byte_end: sample_pos_end * 4,
        }
    });

    let response = client.get_data(request).await?;

    // println!("response: {:?}", response);

    Ok(response)
}

