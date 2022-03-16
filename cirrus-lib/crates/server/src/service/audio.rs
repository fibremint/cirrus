use std::{path::Path, rc::Rc, sync::Arc};

use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};

use cirrus_grpc::{
    api::{AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes, AudioLibraryReq},
    common::{RequestAction, Response as Res},

    audio_data_svc_server::AudioDataSvc,
    audio_library_svc_server::AudioLibrarySvc,
};

use crate::logic;

#[derive(Debug, Default)]
pub struct AudioDataSvcImpl {
    // db_handle: mongodb::Database,
}

#[tonic::async_trait]
impl AudioDataSvc for AudioDataSvcImpl {
    async fn get_meta(
        &self,
        request: Request<AudioMetaReq>
    ) -> Result<Response<AudioMetaRes>, Status> {
        let filepath = &request.get_ref().filename;

        let res = match logic::AudioFile::read_meta(filepath) {
            Ok(res) => Response::new(res),
            Err(err) => return Err(Status::new(Code::Internal, err)),
        };

        Ok(res)
    }


    async fn get_data(
        &self,
        request: Request<AudioDataReq>
    ) -> Result<Response<AudioDataRes>, Status> {
        let filepath = &request.get_ref().filename;
        let byte_start = request.get_ref().byte_start as usize;
        let byte_end = request.get_ref().byte_end as usize;

        let res = match logic::AudioFile::read_data(filepath, byte_start, byte_end) {
            Ok(res) => Response::new(res),
            Err(err) => return Err(Status::new(tonic::Code::Internal, err)),
        };

        Ok(res)
    }
}

// #[derive(Debug, Default)]
pub struct AudioLibrarySvcImpl {
    pub db_handle: Arc<Mutex<mongodb::Database>>,
}

impl AudioLibrarySvcImpl {
    pub fn new(db_handle: Arc<Mutex<mongodb::Database>>) -> Self {
        Self {
            db_handle
        }
    }
}

#[tonic::async_trait]
impl AudioLibrarySvc for AudioLibrarySvcImpl {
    async fn add_audio_library(
        &self,
        request: Request<AudioLibraryReq>
    ) -> Result<Response<Res>, Status> {
        let path = &request.get_ref().path;
        let path = Path::new(path);

        let db_handle = self.db_handle.lock().await;

        let res = match logic::AudioLibrary::add_audio_library(db_handle, path).await {
            Ok(_) => Response::new(Res {
                code: Code::Ok as u32,
                status: Option::None,
                // status: String::from("ok")
            }),
            Err(err) => return Err(Status::not_found(err)),
        };

        Ok(res)
    }

    async fn remove_audio_library(
        &self,
        request: Request<AudioLibraryReq>
    ) -> Result<Response<Res>, Status> {
        let path = request.get_ref().path.clone();
        let path = Path::new(path.as_str());

        let db_handle = self.db_handle.lock().await;

        let res = match logic::AudioLibrary::remove_audio_library(db_handle, path).await {
            Ok(res) => Response::new(Res {
                code: Code::Ok as u32,
                status: Some(format!("Removed item count: {}", res.deleted_count)),
            }),
            Err(err) => return Err(Status::not_found(err)),
        };

        Ok(res)
    }

    async fn refresh_audio_library(
        &self,
        request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {
        let db_handle = self.db_handle.lock().await;

        let res = match logic::AudioLibrary::refresh_audio_library(db_handle).await {
            Ok(_) => Response::new(Res {
                code: Code::Ok as u32,
                status: Some(format!("Refreshed audio library"))
            }),
            Err(err) => return Err(Status::internal(err)),
        };

        Ok(res)
    }
}
