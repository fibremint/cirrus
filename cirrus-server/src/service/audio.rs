use std::path::Path;

use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use cirrus_protobuf::{
    api::{AudioMetaReq, AudioMetaRes, AudioDataReq, AudioDataRes, AudioLibraryReq, AudioTagRes},
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
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'get audio metadata'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match logic::audio::AudioFile::read_meta(self.mongodb_client.clone(), audio_tag_id).await {
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
        let req = request.get_ref();
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'get audio data'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let mut audio_sample_iter = match logic::audio::AudioFile::get_audio_sample_iterator(
            self.mongodb_client.clone(), 
            &req.audio_tag_id, 
            req.packet_start_idx, 
            req.packet_num,
            req.channels,
        ).await {
            Ok(iter) => iter,
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        };

        tokio::spawn(async move {
            while let Some(sample_frame_packet) = audio_sample_iter.next() {
                // let ch_sample_frames = sample_data.encoded_data.iter().enumerate()
                //     .map(|(ch_idx, item)| AudioChannelSampleFrames {
                //         ch_idx: ch_idx.try_into().unwrap(),
                //         encoded_samples: item.to_owned()
                //     })
                //     .collect::<Vec<_>>();

                if let Err(_err) = tx.send(Ok(AudioDataRes {
                    // num_frames: ch_sample_frames[0].encoded_samples.len().try_into().unwrap(),
                    // ch_sample_frames
                    packet_idx: sample_frame_packet.packet_idx,
                    sp_frame_duration: sample_frame_packet.sample_frame_duration,
                    sp_frame_num: sample_frame_packet.sample_frame_num,
                    encoded_samples: sample_frame_packet.encoded_data.to_owned(),
                    // ch_sample_frames,
                })).await {
                    break;
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
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'add audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match logic::audio::AudioLibrary::add_audio_library(self.mongodb_client.clone(), path).await {
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

        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'remove audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match logic::audio::AudioLibrary::remove_audio_library(self.mongodb_client.clone(), path).await {
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
        request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'analyze audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match logic::audio::AudioLibrary::analyze_audio_library(self.mongodb_client.clone()).await {
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
        request: Request<RequestAction>
    ) -> Result<Response<Res>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'refresh audio library'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let res = match logic::audio::AudioLibrary::refresh_audio_library(self.mongodb_client.clone()).await {
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
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'list audio tags'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let req_page = request.get_ref().page;
        let req_items_per_page = request.get_ref().items_per_page;

        let (tx, rx) = mpsc::channel(req_items_per_page as usize);
        let res = logic::audio::AudioTag::list_audio_tags(self.mongodb_client.clone(), req_items_per_page, req_page).await.unwrap();

        tokio::spawn(async move {
            for r in res.into_iter() {
                tx.send(Ok(r)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}