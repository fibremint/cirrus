use tonic::{transport::Server, Request, Response, Status};

use audio_meta::audio_meta_server::{AudioMeta, AudioMetaServer};
use audio_meta::{AudioMetaReq, AudioMetaRes};

pub mod audio_meta {
    tonic::include_proto!("audio");
}

use crate::audio;

#[derive(Debug, Default)]
pub struct AudioMetaSvc {}

#[tonic::async_trait]
impl AudioMeta for AudioMetaSvc {
    // fn get_meta< 'life0, 'async_trait>(& 'life0 self,request:tonic::Request<audio_meta::AudioMetaReq> ,) 
    // ->  core::pin::Pin<Box<dyn core::future::Future<Output = Result<tonic::Response<audio_meta::AudioMetaRes> ,tonic::Status> > + core::marker::Send+ 'async_trait> >where 'life0: 'async_trait,Self: 'async_trait {
    //     todo!()
    // }

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
}

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let audio_meta_svc = AudioMetaSvc::default();

    let server = Server::builder()
        .add_service(AudioMetaServer::new(audio_meta_svc))
        .serve(addr)
        .await?;

    Ok(())
}