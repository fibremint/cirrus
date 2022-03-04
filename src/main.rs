// use std::{sync::Arc, net::SocketAddr, str::FromStr, error};
// // use std::fs::File;
// use bytes::{Bytes, BytesMut, Buf, BufMut};
// use futures::StreamExt;
// use h3::{quic::BidiStream, server::RequestStream};
// use tokio;
// use http::{Request, StatusCode};
// use aiff::reader::AiffReader;
// use tokio::fs::File;

// mod security;

// // static ALPN: &[u8] = b"h3";

// #[tokio::main]
// async fn run() {
//     let test_source = File::open("./test-source.aiff").await.unwrap();
//     let metadata = test_source.metadata().await.unwrap();
//     println!("metadata: {:?}", metadata);

//     // QUIC part
//     // let crypto = security::load_crypto();
//     // let server_config = h3_quinn::quinn::ServerConfig::with_crypto(Arc::new(crypto));
//     // let (endpoint, mut incoming) = h3_quinn::quinn::Endpoint::server(server_config, SocketAddr::from_str("[::1]:4433").unwrap()).unwrap();

//     // while let Some(new_conn) = incoming.next().await {
//     //     println!("new connection being attempted");

//     //     tokio::spawn(async move {
//     //         match new_conn.await {
//     //             Ok(conn) => {
//     //                 println!("new conn established");

//     //                 let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
//     //                     .await
//     //                     .unwrap();

//     //                 while let Some((req, stream)) = h3_conn.accept().await.unwrap() {
//     //                     println!("new request: {:#?}", req);

//     //                     tokio::spawn(async {
//     //                         handle_request(req, stream).await
//     //                     });
//     //                 }
//     //             }
//     //             Err(err) => {
//     //                 println!("accepting conn failed: {:?}", err);
//     //             }
//     //         }
//     //     });
//     // }

//     // endpoint.wait_idle().await;
// }

// async fn handle_request<T>(
//     req: Request<()>,
//     mut stream: RequestStream<T, Bytes>,
// ) where T: BidiStream<Bytes>
// {
//     // println!("uri: {:?}", req.uri().path_and_query().unwrap());
//     let req_path = req.uri().path_and_query().unwrap();
//     println!("path: {:?}", req_path.path());
//     // let test_audio_file = File::open("./test-source.aiff").unwrap();
//     // let mut test_audio_reader = AiffReader::new(test_audio_file);
//     // test_audio_reader.read().unwrap();

//     // let test_audio_comm = test_audio_reader.form().as_ref().unwrap().common().as_ref().unwrap();
//     // test_audio_comm

//     let resp = http::Response::builder()
//         .status(StatusCode::OK)
//         .body(())
//         .unwrap();

//     match stream.send_response(resp).await {
//         Ok(_) => println!("Successfully send a response"),
//         Err(err) => { 
//             println!("Failed to send response");
//             // error!("Failed to send response");
//         },
//     }

//     stream.finish().await;
// }

// fn main() {
//     run();
// }

pub mod audio;
pub mod server;

// pub mod audio_meta {
//     tonic::include_proto!("audio");
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50000".parse()?;
    let addr = "[::1]:50000";
    // let audio_meta_svc = AudioMetaSvc::default();

    // Server::builder()
    //     .add_service(AudioMetaServer::new(audio_meta_svc))
    //     .serve(addr)
    //     .await?;

    server::run_server(addr).await?;
    
    Ok(())
}