use futures::StreamExt;
use tonic::{Request, Response, Status};

use cirrus_grpc::{
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

pub async fn get_audio_data(audio_tag_id: &str, sample_pos_start: u32, sample_pos_end: u32) -> Result<Response<AudioDataRes>, anyhow::Error> {
    let mut client = AudioDataSvcClient::connect("http://127.0.0.1:50000").await?;

    let request = Request::new({
        AudioDataReq {
            // filename: filepath.to_string(),
            audio_tag_id: audio_tag_id.to_string(),
            byte_start: sample_pos_start,
            byte_end: sample_pos_end,
        }
    });

    let response = client.get_data(request).await?;

    Ok(response)
}

// pub async fn get_audio_tags(items_per_page: u64, page: u64) -> Result<Vec<Result<AudioTagRes, Status>>, Box<dyn std::error::Error>> {
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
            // res.push(data::AudioTag {
            //     id: i.id.clone().to_owned(),
            //     artist: i.artist.clone().to_owned(),
            //     genre: i.genre.clone().to_owned(),
            //     title: i.title.clone().to_owned(),
            // })
            res.push(i);
        }
        // res.push(item);
    }

    Ok(res)
}