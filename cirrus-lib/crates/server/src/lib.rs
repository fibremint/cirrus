// pub mod audio;
// pub mod server;

mod logic;
mod model;
mod service;
mod util;

use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

use futures::Future;
use notify::{Watcher, RecursiveMode, watcher};
use tokio::{sync::Mutex, task::JoinHandle};
use tonic::transport::Server as TonicServer;

use cirrus_grpc::{
    audio_data_svc_server::AudioDataSvcServer,
    audio_library_svc_server::AudioLibrarySvcServer,
};

use crate::model::get_mongodb_client;

async fn grpc_server_task(addr: &str, mongodb_client: mongodb::Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("run grpc server");
    let addr = addr.parse().unwrap();

    let audio_data_svc = service::AudioDataSvcImpl::default();
    let audio_library_svc = service::AudioLibrarySvcImpl::new(mongodb_client.clone());

    TonicServer::builder()
        .add_service(AudioDataSvcServer::new(audio_data_svc))
        .add_service(AudioLibrarySvcServer::new(audio_library_svc))
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

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(|| {
        run_fs_notify()
    });

    let mongodb_client = get_mongodb_client().await?;

    grpc_server_task(addr, mongodb_client.clone()).await?;

    Ok(())
}