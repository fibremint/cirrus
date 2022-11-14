mod logic;
mod model;
mod service;
mod util;
mod settings;

// use notify::{Watcher, RecursiveMode, watcher};
use tonic::transport::{Server as TonicServer, Identity, ServerTlsConfig};

use cirrus_protobuf::{
    audio_data_svc_server::AudioDataSvcServer,
    audio_library_svc_server::AudioLibrarySvcServer,
    audio_tag_svc_server::AudioTagSvcServer,
};
use settings::Settings;

async fn serve_grpc_service() -> Result<(), anyhow::Error> {
    let settings = Settings::get()?;

    let server_listen_address = format!(
        "{}:{}", 
        settings.server.listen_address, 
        settings.server.listen_port
    );

    println!("info: listen address: {}", server_listen_address);
    println!("info: use tls: {}", settings.server.tls);

    let addr = server_listen_address.parse().unwrap();
    let mut tonic_server = TonicServer::builder();

    if settings.server.tls {
        let cert = tokio::fs::read(&settings.server.cert_path).await?;
        let key = tokio::fs::read(&settings.server.key_path).await?;

        let identity = Identity::from_pem(cert, key);
        tonic_server = tonic_server.tls_config(ServerTlsConfig::new().identity(identity))?;

        println!("info: loaded TLS identity successfully");
    }

    println!("info: start grpc service");

    tonic_server
        .add_service(AudioDataSvcServer::new(service::AudioDataSvcImpl::default()))
        .add_service(AudioLibrarySvcServer::new(service::AudioLibrarySvcImpl::default()))
        .add_service(AudioTagSvcServer::new(service::AudioTagSvcImpl::default()))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("Cirrus v0.2.0");

    serve_grpc_service().await?;

    Ok(())
}