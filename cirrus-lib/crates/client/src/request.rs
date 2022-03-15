use tonic::{Request, Response};

use cirrus_grpc::{
    api::{AudioDataReq, AudioDataRes, AudioMetaReq, AudioMetaRes},
    audio_data_svc_client::AudioDataSvcClient,
};

pub async fn get_audio_meta(filepath: &str) -> Result<Response<AudioMetaRes>, Box<dyn std::error::Error>> {
    let mut client = AudioDataSvcClient::connect("http://[::1]:50000").await?;

    let request = Request::new({
        AudioMetaReq {
            filename: filepath.to_string()
        }
    });

    let response = client.get_meta(request).await?;

    Ok(response)
}

pub async fn get_audio_data(filepath: &str, sample_pos_start: u32, sample_pos_end: u32) -> Result<Response<AudioDataRes>, Box<dyn std::error::Error>> {
    let mut client = AudioDataSvcClient::connect("http://[::1]:50000").await?;

    let request = Request::new({
        AudioDataReq {
            filename: filepath.to_string(),
            byte_start: sample_pos_start,
            byte_end: sample_pos_end,
        }
    });

    let response = client.get_data(request).await?;

    Ok(response)
}

