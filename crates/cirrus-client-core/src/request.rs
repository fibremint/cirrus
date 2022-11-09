use futures::StreamExt;
use tonic::{Request, Response, Streaming, transport::{ClientTlsConfig, Channel, Endpoint}};

use cirrus_protobuf::{
    api::{AudioDataReq, AudioDataRes, AudioMetaReq, AudioMetaRes, AudioTagRes},
    common::ListRequest,
    audio_data_svc_client::AudioDataSvcClient,
    audio_tag_svc_client::AudioTagSvcClient,
};

pub async fn get_audio_meta(
    grpc_endpoint: &str,
    tls_config: &Option<ClientTlsConfig>,
    audio_tag_id: &str
) -> Result<Response<AudioMetaRes>, anyhow::Error> {
    let endpoint = create_endpoint(grpc_endpoint.to_string(), tls_config)?;
    let tonic_channels = endpoint.connect().await?;

    let mut client = AudioDataSvcClient::new(tonic_channels);

    let request = Request::new({
        AudioMetaReq {
            audio_tag_id: audio_tag_id.to_string()
        }
    });

    let response = client.get_meta(request).await?;

    Ok(response)
}

pub async fn get_audio_data_stream(
    grpc_endpoint: &str,
    tls_config: &Option<ClientTlsConfig>,
    audio_tag_id: &str,
    packet_start_idx: u32,
    packet_num: u32,
    channels: u32,
    // packet_start_ts: u64,
    seek_start_pkt_idx: u64,
    seek_start_pkt_ts: u64,
    // pkt_read_start_offset: u32,
) -> Result<Streaming<AudioDataRes>, anyhow::Error> {
    let endpoint = create_endpoint(grpc_endpoint.to_string(), tls_config)?;
    let tonic_channels = endpoint.connect().await?;

    let mut client = AudioDataSvcClient::new(tonic_channels);

    let request = Request::new({
        AudioDataReq {
            audio_tag_id: audio_tag_id.to_string(),
            packet_start_idx,
            packet_num,
            channels,
            seek_start_pkt_idx,
            seek_start_pkt_ts,
            // pkt_read_start_offset,
        }
    });

    let response = client.get_data(request).await?;

    let stream = response.into_inner();
    
    Ok(stream)
}

pub async fn get_audio_tags(
    grpc_endpoint: &str,
    tls_config: &Option<ClientTlsConfig>,
    items_per_page: u64,
    page: u64
) -> Result<Vec<AudioTagRes>, Box<dyn std::error::Error>> {
    let endpoint = create_endpoint(grpc_endpoint.to_string(), tls_config)?;
    let tonic_channels = endpoint.connect().await?;

    let mut client = AudioTagSvcClient::new(tonic_channels);

    let request = Request::new( 
        ListRequest {
            items_per_page,
            page,
        }
    );

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

fn create_endpoint(
    grpc_endpoint: String, 
    tls_config: &Option<ClientTlsConfig>
) -> Result<Endpoint, anyhow::Error> {
    let mut endpoint = Channel::from_shared(grpc_endpoint)?;
    if let Some(tc) = tls_config {
        endpoint = endpoint.tls_config(tc.clone()).unwrap();
    }

    Ok(endpoint)
}