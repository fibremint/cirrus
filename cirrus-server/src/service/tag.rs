use async_trait::async_trait;
use cirrus_protobuf::{audio_tag_svc_server::AudioTagSvc, api::AudioTagRes, common::ListRequest};
use mongodb::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Status, Response, Code};

use crate::{logic, model};

use super::GetMongoClient;

pub struct AudioTagSvcImpl {
    logic: logic::AudioTag,
}

impl Default for AudioTagSvcImpl {
    fn default() -> Self {
        Self { 
            logic: logic::AudioTag::default(),
        }
    }
}

#[async_trait]
impl GetMongoClient for AudioTagSvcImpl {
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
impl AudioTagSvc for AudioTagSvcImpl {
    type ListAudioTagsStream = ReceiverStream<Result<AudioTagRes, Status>>;

    async fn list_audio_tags(
        &self,
        request: tonic::Request<ListRequest>
    ) -> Result<Response<Self::ListAudioTagsStream>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            println!("info: {} requests 'list audio tags'", remote_addr);
        } else {
            println!("warn: unknown remote address tries to request");
        }

        let req_page = request.get_ref().page;
        let req_items_per_page = request.get_ref().items_per_page;

        let (tx, rx) = mpsc::channel(req_items_per_page as usize);
        // let res = logic::AudioTag::list_audio_tags(self.create_db_client().await?, req_items_per_page, req_page).await.unwrap();

        let res = self.logic.list_audio_tags(
            self.create_db_client().await?, 
            req_items_per_page, 
            req_page
        ).await.unwrap();

        tokio::spawn(async move {
            for r in res.into_iter() {
                tx.send(Ok(r)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}