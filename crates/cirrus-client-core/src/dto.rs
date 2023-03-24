use tonic::transport::ClientTlsConfig;

use crate::request;

#[derive(Debug)]
pub struct Server {
    pub grpc_endpoint: String,
    pub tls_config: Option<ClientTlsConfig>,
}

#[derive(Debug)]
pub struct AudioSource {
    pub server: Server,
    pub id: String,
    pub length: f64,
    pub channels: usize,
    pub bit_rate: u32,
    pub sample_rate: usize,
    pub packet_dur: f64,
    pub content_packets: u32,
}

impl AudioSource {
    pub async fn new(
        grpc_endpoint: &str,
        tls_config: &Option<ClientTlsConfig>,
        audio_tag_id: &str
    ) -> Result<Self, anyhow::Error> {
        let metadata_res = request::get_audio_meta(
            grpc_endpoint,
            tls_config,
            audio_tag_id
        ).await.unwrap().into_inner();

        let server = Server {
            grpc_endpoint: grpc_endpoint.to_string(),
            tls_config: tls_config.clone(),
        };

        Ok(Self {
            server,
            id: audio_tag_id.to_string(),
            length: metadata_res.content_length,
            channels: metadata_res.channels.try_into().unwrap(),
            bit_rate: metadata_res.orig_bit_rate,
            sample_rate: metadata_res.orig_sample_rate as usize,
            packet_dur: metadata_res.packet_dur,
            content_packets: metadata_res.sp_packets,
        })

    }
}
