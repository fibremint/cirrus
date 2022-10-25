use std::path::Path;

use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use cirrus_protobuf::{
    api::{AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes, AudioLibraryReq, AudioTagRes, AudioChannelData},
    common::{RequestAction, Response as Res, ListRequest},

    audio_data_svc_server::AudioDataSvc,
    audio_library_svc_server::AudioLibrarySvc,
    audio_tag_svc_server::AudioTagSvc,
};

use crate::logic;

#[derive(Debug)]
pub struct AudioDataSvcImpl {
    mongodb_client: mongodb::Client,
}

impl AudioDataSvcImpl {
    pub fn new(mongodb_client: mongodb::Client) -> Self {
        Self {
            mongodb_client
        }
    }
}

#[tonic::async_trait]
impl AudioDataSvc for AudioDataSvcImpl {
    type GetDataStream = ReceiverStream<Result<AudioDataRes, Status>>;

    async fn get_meta(
        &self,
        request: Request<AudioMetaReq>
    ) -> Result<Response<AudioMetaRes>, Status> {
        let audio_tag_id = &request.get_ref().audio_tag_id;

        let res = match logic::AudioFile::read_meta(self.mongodb_client.clone(), audio_tag_id).await {
            Ok(res) => Response::new(res),
            Err(err) => return Err(Status::new(Code::Internal, err)),
        };

        Ok(res)
    }

    async fn get_data(
        &self,
        request: Request<AudioDataReq>
    ) -> Result<Response<Self::GetDataStream>, Status> {
        let (tx, rx) = mpsc::channel(16);
        
        let audio_tag_id = &request.get_ref().audio_tag_id;
        let samples_size = request.get_ref().samples_size as usize;
        let samples_start_idx = request.get_ref().samples_start_idx as usize;
        let samples_end_idx = request.get_ref().samples_end_idx as usize;
        let channel_size = 2;

        let mut audio_sample_iter = match logic::AudioFile::read_data(
                self.mongodb_client.clone(), 
                audio_tag_id,
                samples_size,
                samples_start_idx, 
                samples_end_idx
            ).await {
                Ok(audio_raw_data) => logic::AudioSampleIterator::new(
                    samples_size, 
                    channel_size, 
                    audio_raw_data
                ),
                Err(err) => return Err(Status::new(tonic::Code::Internal, err)),
            };

        tokio::spawn(async move {
            while let Some(sample_data) = audio_sample_iter.next() {
                let audio_channel_data = sample_data.iter()
                    .map(|item| AudioChannelData {
                        content: item.to_owned()
                    })
                    .collect::<Vec<_>>();

                if let Err(_err) = tx.send(Ok(AudioDataRes {
                    audio_channel_data
                })).await {
                    break;
                    // return Err(Status::new(tonic::Code::Aborted, err))
                    // println!("WARN: closed the stream of send audio data");
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

// #[derive(Debug, Default)]
pub struct AudioLibrarySvcImpl {
    mongodb_client: mongodb::Client,
}

impl AudioLibrarySvcImpl {
    pub fn new(mongodb_client: mongodb::Client) -> Self {
        Self {
            mongodb_client
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

        let res = match logic::AudioLibrary::add_audio_library(self.mongodb_client.clone(), path).await {
            Ok(_) => Response::new(Res {
                code: Code::Ok as u32,
                status: Option::None,
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

        let res = match logic::AudioLibrary::remove_audio_library(self.mongodb_client.clone(), path).await {
            Ok(res) => Response::new(Res {
                code: Code::Ok as u32,
                status: Some(res),
            }),
            Err(err) => return Err(Status::not_found(err)),
        };

        Ok(res)
    }

    async fn analyze_audio_library(
        &self,
        _request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {

        let res = match logic::AudioLibrary::analyze_audio_library(self.mongodb_client.clone()).await {
            Ok(_) => Response::new(Res {
                code: Code::Ok as u32,
                status: Some(format!("Refreshed audio library"))
            }),
            Err(err) => return Err(Status::internal(err)),
        };

        Ok(res)
    }

    async fn refresh_audio_library(
        &self,
        _request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {

        let res = match logic::AudioLibrary::refresh_audio_library(self.mongodb_client.clone()).await {
            Ok(_) => Response::new(Res {
                code: Code::Ok as u32,
                status: Some(format!("Refreshed audio library"))
            }),
            Err(err) => return Err(Status::internal(err)),
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub struct AudioTagSvcImpl {
    mongodb_client: mongodb::Client,
}

impl AudioTagSvcImpl {
    pub fn new(mongodb_client: mongodb::Client) -> Self {
        Self {
            mongodb_client,
        }
    }
}

#[tonic::async_trait]
impl AudioTagSvc for AudioTagSvcImpl {
    type ListAudioTagsStream = ReceiverStream<Result<AudioTagRes, Status>>;

    async fn list_audio_tags(
        &self,
        request: tonic::Request<ListRequest>
    ) -> Result<Response<Self::ListAudioTagsStream>, Status> {
        // let tag_num = 20;
        let req_page = request.get_ref().page;
        let req_items_per_page = request.get_ref().items_per_page;

        let (tx, rx) = mpsc::channel(req_items_per_page as usize);
        let res = logic::AudioTag::list_audio_tags(self.mongodb_client.clone(), req_items_per_page, req_page).await.unwrap();

        tokio::spawn(async move {
            for r in res.into_iter() {
                tx.send(Ok(r)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}