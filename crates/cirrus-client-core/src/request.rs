use futures::StreamExt;
use tonic::{Request, Response, Streaming};

use cirrus_protobuf::{
    api::{AudioDataReq, AudioDataRes, AudioMetaReq, AudioMetaRes, AudioTagRes},
    common::ListRequest,
    audio_data_svc_client::AudioDataSvcClient,
    audio_tag_svc_client::AudioTagSvcClient,
};

pub async fn get_audio_meta(audio_tag_id: &str) -> Result<Response<AudioMetaRes>, anyhow::Error> {
    let mut client = AudioDataSvcClient::connect("http://127.0.0.1:50000").await?;

    let request = Request::new({
        AudioMetaReq {
            audio_tag_id: audio_tag_id.to_string()
        }
    });

    let response = client.get_meta(request).await?;

    Ok(response)
}

pub async fn get_audio_data_stream(
    audio_tag_id: &str,
    samples_size: u32,
    samples_start_idx: u32, 
    samples_end_idx: u32
) -> Result<Streaming<AudioDataRes>, anyhow::Error> {
    let mut client = AudioDataSvcClient::connect("http://127.0.0.1:50000").await?;

    let request = Request::new({
        AudioDataReq {
            audio_tag_id: audio_tag_id.to_string(),
            samples_size,
            samples_start_idx,
            samples_end_idx,
        }
    });

    let response = client.get_data(request).await?;

    let stream = response.into_inner();
    
    Ok(stream)
}

pub async fn get_audio_tags(items_per_page: u64, page: u64) -> Result<Vec<AudioTagRes>, Box<dyn std::error::Error>> {
    let mut client = AudioTagSvcClient::connect("http://127.0.0.1:50000").await?;

    let request = Request::new( {
        ListRequest {
            items_per_page,
            page,
        }
    });

    let response = client.list_audio_tags(request).await.unwrap();
    let mut res: Vec<_> = Vec::new();

    let stream = response.into_inner();
    let mut stream = stream.take(items_per_page as usize);

    while let Some(item) = stream.next().await {
        if let Ok(i) = item {
            res.push(i);
        }
    }

    Ok(res)
}