use async_trait::async_trait;
use cirrus_protobuf::{api::{AudioMetaReq, AudioDataRes, AudioDataReq, AudioMetaRes}, audio_data_svc_server::AudioDataSvc};
use mongodb::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Status, Response, Code, Request};

use crate::{logic, model};

use super::GetMongoClient;

pub struct AudioDataSvcImpl {}

impl Default for AudioDataSvcImpl {
    fn default() -> Self {
        Self {  }
    }
}

#[async_trait]
impl GetMongoClient for AudioDataSvcImpl {
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
impl AudioDataSvc for AudioDataSvcImpl {
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

        let res = match logic::AudioFile::read_meta(self.create_db_client().await?, audio_tag_id).await {
            Ok(res) => Response::new(res),
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        };

        Ok(res)
    }

    type GetDataStream = ReceiverStream<Result<AudioDataRes, Status>>;

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

        let mut packets = match logic::AudioFile::get_audio_sample_iterator(
            self.create_db_client().await?, 
            &req.audio_tag_id, 
            req.packet_start_idx.try_into().unwrap(), 
            req.packet_num.try_into().unwrap(),
            req.channels,
        ).await {
            Ok(iter) => iter,
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        };

        tokio::spawn(async move {
            while let Some(packet) = packets.next() {
                let packet_res = AudioDataRes {
                    packet_idx: packet.idx.try_into().unwrap(),

                    frame_ts: packet.frame_ts.try_into().unwrap(),
                    sp_frame_duration: packet.frame_dur.try_into().unwrap(),
                    sp_frame_num: packet.frame_len.try_into().unwrap(),
                    packet_start_ts: packet.next_pkt_seek_ts,

                    encoded_samples: packet.frame.to_owned()
                };

                if let Err(_err) = tx.send(Ok(packet_res)).await {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}