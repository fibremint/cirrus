use std::{collections::VecDeque, sync::{mpsc, Arc}, mem::MaybeUninit};
use cirrus_protobuf::api::AudioDataRes;
use ringbuf::{SharedRb, Producer};
use rubato::Resampler;

use crate::{dto::AudioSource, request};

use super::{packet::EncodedBuffer, context::HostStreamConfig};

pub struct AudioSample {
    pub source: AudioSource,

    sample_frame_buf: Vec<VecDeque<f32>>,
    packet_buf: EncodedBuffer,
    packet_decoder: opus::Decoder,
    resampler: AudioResampler,
    context: AudioSampleContext,

    ringbuf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,

    // pub tx: mpsc::Sender<&'static str>,
    // rx: mpsc::Receiver<&'static str>,
    // audio_data_res_tx: mpsc::Sender<AudioDataRes>,
}

impl AudioSample {
    pub fn new(
        source: AudioSource,
        host_stream_config: HostStreamConfig,
        ringbuf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
        // audio_data_res_tx: mpsc::Sender<AudioDataRes>,
    ) -> Result<Self, anyhow::Error> {
        let mut sample_frame_buf: Vec<VecDeque<f32>> = Vec::with_capacity(2);

        for _ in 0..host_stream_config.output_channels {
            sample_frame_buf.push(VecDeque::new());
        }

        let packet_buf = EncodedBuffer::new(source.content_packets);
        let packet_decoder = opus::Decoder::new(48_000, opus::Channels::Stereo).unwrap();
        
        // let (tx, rx) = mpsc::channel(64);
        // let (tx, rx) = mpsc::channel();

        Ok(Self {
            source,
            sample_frame_buf,
            packet_buf,
            packet_decoder,
            resampler: AudioResampler::new(host_stream_config.sample_rate.try_into()?, 960)?,
            context: AudioSampleContext::default(),
            ringbuf_producer,
            // tx,
            // rx,
            // audio_data_res_tx,
        })
    }
}

struct AudioSampleContext {
    pub playback_sample_frame_pos: usize,
    pub buffer_status: usize,
    // pub host_sample_rate: u32,
    // pub host_output_channels: usize,

    pub packet_playback_idx: usize,
}

impl Default for AudioSampleContext {
    fn default() -> Self {
        Self { 
            playback_sample_frame_pos: Default::default(), 
            buffer_status: Default::default(), 
            // host_sample_rate: Default::default(), 
            // host_output_channels: Default::default(), 
            packet_playback_idx: Default::default() 
        }
    }
}

struct AudioResampler {
    resampler: rubato::FftFixedIn<f32>,
    input_buf: Vec<Vec<f32>>,
    output_buf: Vec<Vec<f32>>,
}

impl AudioResampler {
    fn new(
        output_sample_rate: usize,
        chunk_size_in: usize,
    ) -> Result<Self, anyhow::Error> {
        let resampler = rubato::FftFixedIn::<f32>::new(
            48_000,
            output_sample_rate,
            chunk_size_in,
            2,
            2
        )?;

        let mut input_buf = resampler.input_buffer_allocate();
        for input_buf_ch in input_buf.iter_mut() {
            input_buf_ch.extend(vec![0.; chunk_size_in]);
        }

        let output_buf = resampler.output_buffer_allocate();

        Ok(Self {
            resampler,
            input_buf,
            output_buf,
        })
    }
}