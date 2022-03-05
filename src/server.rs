use tonic::{transport::Server, Request, Response, Status};

pub mod audio_proto {
    tonic::include_proto!("audio");
}

use audio_proto::audio_svc_server::{AudioSvc, AudioSvcServer};
use audio_proto::{AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes};

use crate::audio;

#[derive(Debug, Default)]
pub struct AudioSvcImpl {}

#[tonic::async_trait]
impl AudioSvc for AudioSvcImpl {
    async fn get_meta(
        &self,
        request: Request<AudioMetaReq>
    ) -> Result<Response<AudioMetaRes>, Status> {
        println!("Got a request: {:?}", request);

        // println!("resp: {:?}", request.get_ref().filename);
        // audio::check_file_exists(&request.get_ref().filename);

        // audio::read_meta(&request.get_ref().filename);

        let filepath = &request.get_ref().filename;

        let audio_meta_res = audio::read_meta(filepath).unwrap();

        // if let audio_meta_res = Some(audio::read_meta(filepath)) {
        //     Ok(resp) => Response::new(resp),
        //     Err(err) => Response,
        // }

        // let resp = AudioMetaRes {
        //     content: format!("msg 1"),
        // };

        Ok(Response::new(audio_meta_res))
    }

    // fn get_data< 'life0, 'async_trait>(& 'life0 self,request:tonic::Request<audio_proto::AudioDataReq> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = Result<tonic::Response<audio_proto::AudioDataRes> ,tonic::Status> > + core::marker::Send+ 'async_trait> >where 'life0: 'async_trait,Self: 'async_trait {
    //     todo!()
    // }

    async fn get_data(
        &self,
        request: Request<AudioDataReq>
    ) -> Result<Response<AudioDataRes>, Status> {
        println!("Got a request: {:?}", request);

        let filepath = &request.get_ref().filename;
        // let filepath = 
        let byte_start = request.get_ref().byte_start as usize;
        let byte_end = request.get_ref().byte_end as usize;
        let audio_data_res = audio::read_data(filepath, byte_start, byte_end).unwrap();

        // let audio_data_res = AudioDataRes {
        //     content: Vec::<u8>::new()
        // };

        Ok(Response::new(audio_data_res))

    }
}

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let audio_svc = AudioSvcImpl::default();

    let server = Server::builder()
        .add_service(AudioSvcServer::new(audio_svc))
        .serve(addr)
        .await?;

    Ok(())
}