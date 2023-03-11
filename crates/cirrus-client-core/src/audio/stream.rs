use std::{sync::{Arc, mpsc::{SyncSender, Sender, self}, Mutex}, mem::MaybeUninit};
use ringbuf::{HeapRb, SharedRb, Consumer};

use cpal::traits::{DeviceTrait, StreamTrait};
use cirrus_protobuf::api::AudioDataRes;
use tokio::runtime::Handle;
// use tokio::sync::mpsc;

use super::{sample::AudioSample, context::AudioContext};
use crate::dto::AudioSource;

pub struct AudioStream {
    stream: cpal::Stream,
    audio_sample: AudioSample,
    status: usize,
}

impl AudioStream {
    pub fn new(
        rt_handle: &Arc<Handle>,
        audio_ctx: &AudioContext,
        source: AudioSource,
    ) -> Result<Self, anyhow::Error> {
        // let (audio_data_tx, audio_data_rx) = mpsc::channel::<AudioDataRes>();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        // let _audio_sample_tx = Arc::new(audio_sample.tx.clone());
        // let (tx, rx) = mpsc::channel(64);
        // let (tx, rx) = std::sync::mpsc::sync_channel(1);
        // let (tx, rx) = std::sync::mpsc::channel::<&'static str>();

        // let _tx = Arc::new(Mutex::new(tx));
        // let _tx2 = _tx.clone();
        
        let hsc = audio_ctx.host_stream_config();
        // ringbuf len: (ms)
        let ring_buf_len = 150.;
        let latency_frames = (ring_buf_len / 1_000.0) * hsc.sample_rate as f32;
        let latency_samples = latency_frames as usize * hsc.output_channels as usize;

        let ringbuf = HeapRb::<f32>::new(latency_samples * 2);
        let (rb_prod, mut rb_con) = ringbuf.split();
        
        let audio_sample = AudioSample::new(
            source,
            audio_ctx.host_stream_config(),
            // audio_data_tx,
            rb_prod
        )?;

        audio_sample.fetch_buffer(rt_handle).unwrap();
        audio_sample.start_process_buf();

        // let audio_sample_tx = audio_sample.tx.clone();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut input_buf_fell_behind = false;
            for sample in data {
                *sample = match rb_con.pop() {
                    Some(s) => s,
                    None => {
                        input_buf_fell_behind = true;
                        0.0
                    }
                };
            }

            if input_buf_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");
            }
        };

        let stream = audio_ctx.device.build_output_stream(
            &audio_ctx.stream_config, 
            output_data_fn, 
            err_fn
        )?;

        Ok(Self {
            stream,
            audio_sample,
            status: 0,
        })

    }

    pub fn play(&mut self) -> Result<(), anyhow::Error> {
        self.stream.play()?;

        Ok(())
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        self.stream.pause()?;

        Ok(())
    }
}

// fn output_data_fn_test(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    
// }

// fn audio_stream_pipeline(
//     output: &mut [f32],
//     // audio_sample_tx: mpsc::Sender<&'static str>,
//     // audio_sample_tx: SyncSender<&'static str>,
//     // audio_sample_tx: Arc<Mutex<Sender<&'static str>>>,
//     consumer: Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
// ) {

// }

// fn get_packet(audio_sample_tx: mpsc::Sender<&'static str>) -> AudioDataRes {
//     todo!()
// }