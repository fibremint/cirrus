use std::path::Path;

use tonic::{Code, Request, Response, Status};

use cirrus_grpc::{
    api::{AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes, AudioLibraryReq},
    common::{RequestAction, Response as Res},

    audio_data_svc_server::AudioDataSvc,
    audio_library_svc_server::AudioLibrarySvc,
};

use crate::logic;

#[derive(Debug, Default)]
pub struct AudioDataSvcImpl {}

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

#[derive(Debug, Default)]
pub struct AudioLibrarySvcImpl {}

#[tonic::async_trait]
impl AudioLibrarySvc for AudioLibrarySvcImpl {
    async fn add_audio_library(
        &self,
        request: Request<AudioLibraryReq>
    ) -> Result<Response<Res>, Status> {
        let path = &request.get_ref().path;
        let path = Path::new(path);

        let res = match logic::AudioLibrary::add_audio_library(path) {
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

        logic::AudioLibrary::remove_audio_library(path);

        // let is_path_exists = match path.
        let res = Res {
            code: Code::Ok as u32,
            status: Option::None,
            // status: String::from("ok")
        };

        Ok(Response::new(res))

    }

    async fn refresh_audio_library(
        &self,
        request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {
        logic::AudioLibrary::refresh_audio_library();

        let res = Res {
            code: Code::Ok as u32,
            status: Option::None,
            // status: String::from("ok")
        };

        Ok(Response::new(res))
    }
}
