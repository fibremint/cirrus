mod logic;
mod model;
mod service;
mod util;
mod settings;

use std::env;
use std::{sync::Arc, path::Path};
use std::sync::mpsc::channel;
use std::time::Duration;

use futures::Future;
use notify::{Watcher, RecursiveMode, watcher};
use tokio::{sync::Mutex, task::JoinHandle};
use tonic::transport::Server as TonicServer;

use cirrus_protobuf::{
    audio_data_svc_server::AudioDataSvcServer,
    audio_library_svc_server::AudioLibrarySvcServer,
    audio_tag_svc_server::AudioTagSvcServer,
};
use settings::Settings;

use crate::model::get_mongodb_client;

async fn grpc_server_task(addr: &str, mongodb_client: mongodb::Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("run grpc server");
    let addr = addr.parse().unwrap();

    // let audio_data_svc = service::AudioDataSvcImpl::default();
    let audio_data_svc = service::AudioDataSvcImpl::new(mongodb_client.clone());
    let audio_library_svc = service::AudioLibrarySvcImpl::new(mongodb_client.clone());
    let audio_tag_svc = service::AudioTagSvcImpl::new(mongodb_client.clone());

    TonicServer::builder()
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

pub async fn run_server(addr: &str, mongodb_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // std::thread::spawn(|| {
    //     run_fs_notify()
    // });

    let mongodb_client = get_mongodb_client(mongodb_addr).await?;

    grpc_server_task(addr, mongodb_client.clone()).await?;

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

    run_server(&server_listen_address, &settings.mongodb.address).await?;

    Ok(())
}