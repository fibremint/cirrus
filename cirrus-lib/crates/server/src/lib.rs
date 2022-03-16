// pub mod audio;
// pub mod server;

mod logic;
mod model;
mod service;

use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::Server;

use cirrus_grpc::{
    audio_data_svc_server::AudioDataSvcServer,
    audio_library_svc_server::AudioLibrarySvcServer,
};

use crate::model::get_mongodb_handle;

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;

    let db_handle = get_mongodb_handle().await?;
    let audio_data_svc = service::AudioDataSvcImpl::default();
    // let audio_library_svc = service::AudioLibrarySvcImpl::default();
    let audio_library_svc = service::AudioLibrarySvcImpl::new(Arc::new(Mutex::new(db_handle)));

    Server::builder()
        .add_service(AudioDataSvcServer::new(audio_data_svc))
        .add_service(AudioLibrarySvcServer::new(audio_library_svc))
        .serve(addr)
        .await?;

    Ok(())
}