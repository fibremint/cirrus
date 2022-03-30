use std::{sync::Arc, collections::HashMap, thread};

use tauri::async_runtime::{RwLock, Mutex};

use cirrus_client_lib::audio::AudioPlayer;
// use cirrus_client_lib::cirrus_grpc::api::AudioTagRes;

pub struct AppState {
    // audio_player: Arc<RwLock<AudioPlayer>>,
    pub audio_player: Arc<RwLock<AudioPlayer>>,
    // pub audio_tags: Mutex<HashMap<u32, Vec<AudioTag>>>,
    // pub audio_player: Arc<AudioPlayer>,
    // pub audio_player: Arc<Mutex<AudioPlayer>>,
    // pub audio_player: AudioPlayer,
}

impl AppState {
    pub async fn new() -> Self {
        // let mut audio_player = Arc::new(AudioPlayer::new());
        // let audio_player = Arc::new(Mutex::new(AudioPlayer::new()));
        // let audio_player = Arc::new(RwLock::new(AudioPlayer::new()));
    
        // let _audio_player = audio_player.clone();
        // let _rx = _audio_player.lock().await.rx.clone();

        // // tokio::spawn(async move {
        // //     while let Ok(data) = _rx.lock().unwrap().recv() {
        // //         println!("msg: {}", data);
        // //     }
        // // });

        // thread::spawn(move || {
        //     while let Ok(data) = _rx.lock().unwrap().recv() {
        //         println!("got audio player msg@appstate: {}", data);
        //     }
        //  });
      



        // thread::spawn(move || {      
        //     // while let Ok(data) = _audio_player.lock().await.recv() {
        //     // while let Ok(data) = _rx.lock().await.recv() {
        //     //     println!("received message: {:?}", data);
        //     //     match data {
        //     //         "stop" => _self.lock().unwrap().remove_audio(),
        //     //         _ => (),
        //     //     }
        //     // }
        // });
        // let mut _audio_player = audio_player.clone().lock().await;
        // let r = _audio_player.lock().await;
        // AudioPlayer::init(audio_player.lock().await);
        
        // audio_player.init();
        // audio_player.lock().await.
        // let audio_player = Arc::new(RwLock::new(AudioPlayer::new()));
        // let _audio_player = audio_player.read().await;
        // audio_player.
        // _audio_player
        Self {
            audio_player: Arc::new(RwLock::new(AudioPlayer::new())),
            // audio_tags: Mutex::new(HashMap::new()),
            // audio_player: AudioPlayer::new(),
        }
    }
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}