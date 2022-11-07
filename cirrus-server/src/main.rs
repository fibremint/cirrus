mod logic;
mod model;
mod service;
mod util;
mod settings;

use std::env;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Watcher, RecursiveMode, watcher};
use tonic::transport::{Server as TonicServer, Identity, ServerTlsConfig};

use cirrus_protobuf::{
    audio_data_svc_server::AudioDataSvcServer,
    audio_library_svc_server::AudioLibrarySvcServer,
    audio_tag_svc_server::AudioTagSvcServer,
};
use settings::Settings;

use model::get_mongodb_client;

async fn grpc_server_task(addr: &str, settings: &Settings, mongodb_client: mongodb::Client) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();

    let audio_data_svc = service::audio::AudioDataSvcImpl::new(mongodb_client.clone());
    let audio_library_svc = service::audio::AudioLibrarySvcImpl::new(mongodb_client.clone());
    let audio_tag_svc = service::audio::AudioTagSvcImpl::new(mongodb_client.clone());

    let mut tonic_server = TonicServer::builder();

    if settings.server.tls {
        let cert = tokio::fs::read(&settings.server.cert_path).await?;
        let key = tokio::fs::read(&settings.server.key_path).await?;

        let identity = Identity::from_pem(cert, key);
        tonic_server = tonic_server.tls_config(ServerTlsConfig::new().identity(identity))?;

        println!("info: created TLS identity successfully");
    }

    println!("info: start grpc service");

    tonic_server
        .add_service(AudioDataSvcServer::new(audio_data_svc))
        .add_service(AudioLibrarySvcServer::new(audio_library_svc))
        .add_service(AudioTagSvcServer::new(audio_tag_svc))
        .serve(addr)
        .await?;

    Ok(())
}

fn run_fs_notify() -> Result<(), ()> {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(5)).unwrap();
    watcher.watch("d:\\tmp", RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

// fn process_chaged_audio_files_at_start() {
//     let audio_types = ["aiff"];

//     // let mut libraries: HashMap<PathBuf, Vec<document::AudioFile>> = HashMap::new();
//     // let mut libraries = HashSet::new();
//     // let mut audio_file_docs: Vec<document::AudioFile> = Vec::new();

//     let audio_file_entry_iter = WalkDir::new(library_root).into_iter()
//         .filter_map(|item| item.ok())
//         // .map(|item| item.unwrap())
//         .filter(|item| 
//             item.metadata().unwrap().is_file() && 
//             item.path().extension() != None)
//         .filter(|item| {
//             let file_extension = item.path().extension().unwrap();
//             audio_types.contains(&file_extension.to_str().unwrap())
//         });
// }

pub async fn run_server(addr: &str, settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    // std::thread::spawn(|| {
    //     run_fs_notify()
    // });

    println!("Cirrus v0.2.0");
    println!("info: listen address: {}", addr);
    println!("info: use tls: {}", settings.server.tls);

    let mongodb_client = get_mongodb_client(&settings.mongodb.address).await?;

    grpc_server_task(addr, &settings, mongodb_client.clone()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir().unwrap();
    let server_config_path = current_dir.join("configs/cirrus/server.toml");
    
    let settings = Settings::new(&server_config_path).unwrap();

    let server_listen_address = format!(
        "{}:{}", 
        settings.server.listen_address, 
        settings.server.listen_port
    );

    run_server(&server_listen_address, settings).await?;

    Ok(())
}