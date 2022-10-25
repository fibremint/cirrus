use std::path::Path;

use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use ndarray::prelude::*;

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
        // let filepath = &request.get_ref().filename;
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

        let audio_data = match logic::AudioFile::read_data(
            self.mongodb_client.clone(), 
            audio_tag_id,
            samples_size,
            samples_start_idx, 
            samples_end_idx
        ).await {
            Ok(res) => res,
            Err(err) => return Err(Status::new(tonic::Code::Internal, err)),
        };

        // let t = ArrayView3::from_shape(
        //     (samples_end_idx-samples_start_idx, 2, samples_size).strides((samples_size*2, 1, 2)), &audio_data).unwrap();

        // let l = t.len_of(Axis(0));
        // for idx in 0..l {
        //     let r = t.slice(s![idx, .., ..]);
        //     // let r = t.slice(s![idx, .., ..]).into_shape((2, samples_size)).unwrap();
        //     println!("{:?}", r);
        //     let r2 = AudioChannelData {
        //         content: Vec::from(r)
        //         // content: Vec::from(r.to_slice().unwrap())
        //     };
        //     println!("{:?}", r2);
        // }

        let channel_size = 2;

        // let mut data_buf = Vec::with_capacity(channel_size);
        // for _ in 0..channel_size {
        //     data_buf.push(vec![0.; 0]);
        // }

        // let mut data_chunk_iter = audio_data.chunks(samples_size*channel_size);
        // while let Some(sample_data) = data_chunk_iter.next() {
        //     for ch_buf in data_buf.iter_mut() {
        //         *ch_buf = vec![0.; sample_data.len()/channel_size];
        //     }

        //     for (sample_idx, sample_chunk) in sample_data.chunks(channel_size).enumerate() {
        //         for ch_idx in 0..channel_size {
        //             data_buf[ch_idx][sample_idx] = sample_chunk[ch_idx];
        //         }
        //     }

        //     let t2 = data_buf.iter()
        //         .map(|item| AudioChannelData {
        //             content: item.to_owned()
        //         })
        //         .collect::<Vec<_>>();
        // }

        tokio::spawn(async move {
            let mut data_buf = Vec::with_capacity(channel_size);
            for _ in 0..channel_size {
                data_buf.push(vec![0.; 0]);
            }
    
            let mut data_chunk_iter = audio_data.chunks(samples_size*channel_size);
            while let Some(sample_data) = data_chunk_iter.next() {
                for ch_buf in data_buf.iter_mut() {
                    *ch_buf = vec![0.; sample_data.len()/channel_size];
                }
    
                for (sample_idx, sample_chunk) in sample_data.chunks(channel_size).enumerate() {
                    for ch_idx in 0..channel_size {
                        data_buf[ch_idx][sample_idx] = sample_chunk[ch_idx];
                    }
                }
    
                let audio_channel_data = data_buf.iter()
                    .map(|item| AudioChannelData {
                        content: item.to_owned()
                    })
                    .collect::<Vec<_>>();

                if let Err(_) = tx.send(Ok(AudioDataRes {
                    audio_channel_data
                })).await {
                    // println!("WARN: closed the stream of send audio data");
                }
            }

            // while let Some(chunk_item) = data_chunk_iter.next() {
            //     if let Err(_) = tx.send(Ok(AudioDataRes {
            //         content: chunk_item.to_vec()
            //     })).await {
            //         // println!("WARN: closed the stream of send audio data");
            //     }
            // }

            // let a = AudioDataRes {
            //     audio_channel_data: Vec::new()
            // };
            // let audio_channel_data = res.into_inner().audio_channel_data;
            // let mut data_chunk_iter = content.chunks(1024);
            // let mut audio_data_ch_chunks = audio_data.iter().map(|item| item.chunks(samples_size)).collect::<Vec<_>>();

            // loop {
            //     // let mut audio_channel_data: Vec<Vec<f32>> = Vec::with_capacity(2);
            //     let mut audio_channel_data: Vec<AudioChannelData> = Vec::with_capacity(2);
            //     // for _ in 0..2 {
            //     //     // audio_channel_data.push(Vec::with_capacity(1024));
            //     //     audio_channel_data.push(AudioChannelData {
            //     //         content: None
            //     //     });
            //     // }

            //     for ch_chunk in audio_data_ch_chunks.iter_mut() {
            //         match ch_chunk.next() {
            //             Some(item) => audio_channel_data.push(AudioChannelData {
            //                 content: item.to_vec()
            //             }),
            //             None => return
            //         }
            //     }

            //     if let Err(_) = tx.send(Ok(AudioDataRes {
            //         audio_channel_data: audio_channel_data.to_vec()
            //     })).await {
            //         // println!("WARN: closed the stream of send audio data");
            //     }
            // }

            // while let Some(chunk_item) = data_chunk_iter.next() {
            //     if let Err(_) = tx.send(Ok(AudioDataRes {
            //         content: chunk_item.to_vec()
            //     })).await {
            //         // println!("WARN: closed the stream of send audio data");
            //     }
            // }
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
        request: Request<RequestAction>
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
        request: Request<RequestAction>
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