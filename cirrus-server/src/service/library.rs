use std::path::Path;

use async_trait::async_trait;
use cirrus_protobuf::{
    api::AudioLibraryReq,

    common::{Response as CirrusResponse, RequestAction},

    audio_library_svc_server::AudioLibrarySvc
};
use mongodb::Client;
use tonic::{Status, Response, Code, Request};

use crate::{logic, model};

use super::GetMongoClient;

pub struct AudioLibrarySvcImpl {
    logic: logic::AudioLibrary,
}

impl Default for AudioLibrarySvcImpl {
    fn default() -> Self {
        Self { 
            logic: Default::default()
        }
    }
}

#[async_trait]
impl GetMongoClient for AudioLibrarySvcImpl {
    async fn create_db_client(&self) -> Result<Client, Status> {
        let db = match model::create_db_client().await {
            Ok(db) => db,
            Err(err) => {
                return Err(Status::new(Code::Internal, err.to_string()))
            },
        };

        Ok(db)
    }
}

#[tonic::async_trait]
impl AudioLibrarySvc for AudioLibrarySvcImpl {
    async fn add_audio_library(
        &self,
        request: Request<AudioLibraryReq>
    ) -> Result<Response<CirrusResponse>, Status> {
        let path = &request.get_ref().path;
        let path = Path::new(path);
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'add audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match self.logic.add_audio_library(self.create_db_client().await?, path).await {
            Ok(_) => Response::new(CirrusResponse {
                code: Code::Ok as u32,
                status: Option::None,
            }),
            Err(err) => return Err(Status::not_found(err.to_string())),
        };

        Ok(res)
    }

    async fn remove_audio_library(
        &self,
        request: Request<AudioLibraryReq>
    ) -> Result<Response<CirrusResponse>, Status> {
        let path = request.get_ref().path.clone();
        let path = Path::new(path.as_str());

        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'remove audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match self.logic.remove_audio_library(self.create_db_client().await?, path).await {
            Ok(res) => Response::new(CirrusResponse {
                code: Code::Ok as u32,
                status: Some(res),
            }),
            Err(err) => return Err(Status::not_found(err.to_string())),
        };

        Ok(res)
    }

    async fn analyze_audio_library(
        &self,
        request: Request<RequestAction>
    ) -> Result<Response<CirrusResponse>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'analyze audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match self.logic.analyze_audio_library(self.create_db_client().await?).await {
            Ok(_) => Response::new(CirrusResponse {
                code: Code::Ok as u32,
                status: Some(format!("Refreshed audio library"))
            }),
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        Ok(res)
    }

    async fn refresh_audio_library(
        &self,
        request: Request<RequestAction>
    ) -> Result<Response<CirrusResponse>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'refresh audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match self.logic.refresh_audio_library(self.create_db_client().await?).await {
            Ok(_) => Response::new(CirrusResponse {
                code: Code::Ok as u32,
                status: Some(format!("Refreshed audio library"))
            }),
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        Ok(res)
    }
}