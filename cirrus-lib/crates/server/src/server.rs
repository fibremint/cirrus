use tonic::{transport::Server, Request, Response, Status};

use cirrus_grpc::audio_proto::{
    AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes,
    audio_svc_server::{AudioSvc, AudioSvcServer}
};

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

        let filepath = &request.get_ref().filename;
        let audio_meta_res = audio::read_meta(filepath).unwrap();

        Ok(Response::new(audio_meta_res))
    }


    async fn get_data(
        &self,
        request: Request<AudioDataReq>
    ) -> Result<Response<AudioDataRes>, Status> {
        println!("Got a request: {:?}", request);

        let filepath = &request.get_ref().filename;
        let byte_start = request.get_ref().byte_start as usize;
        let byte_end = request.get_ref().byte_end as usize;
        let audio_data_res = audio::read_data(filepath, byte_start, byte_end).unwrap();

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