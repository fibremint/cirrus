use futures::StreamExt;
use tonic::{Request, Response, Streaming, codegen::StdError};

use cirrus_protobuf::{
    api::{AudioDataReq, AudioDataRes, AudioMetaReq, AudioMetaRes, AudioTagRes},
    common::ListRequest,
    audio_data_svc_client::AudioDataSvcClient,
    audio_tag_svc_client::AudioTagSvcClient,
};

pub async fn get_audio_meta(server_address: String, audio_tag_id: &str) -> Result<Response<AudioMetaRes>, anyhow::Error> 
{
    let mut client = AudioDataSvcClient::connect(server_address).await?;

    let request = Request::new({
        AudioMetaReq {
            audio_tag_id: audio_tag_id.to_string()
        }
    });

    let response = client.get_meta(request).await?;

    Ok(response)
}

pub async fn get_audio_data_stream(
    server_address: String,
    audio_tag_id: &str,
    packet_start_idx: u32,
    packet_num: u32,
    channels: u32,
) -> Result<Streaming<AudioDataRes>, anyhow::Error> {
    let mut client = AudioDataSvcClient::connect(server_address).await?;

    let request = Request::new({
        AudioDataReq {
            audio_tag_id: audio_tag_id.to_string(),
            packet_start_idx,
            packet_num,
            channels,
        }
    });

    let response = client.get_data(request).await?;

    let stream = response.into_inner();
    
    Ok(stream)
}

pub async fn get_audio_tags(server_address: String, items_per_page: u64, page: u64) -> Result<Vec<AudioTagRes>, Box<dyn std::error::Error>> {
    let mut client = AudioTagSvcClient::connect(server_address).await?;

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