use std::{collections::VecDeque, sync::{mpsc, Arc, Mutex}, mem::MaybeUninit, time::Duration};
use audio::{InterleavedBufMut, wrap::Interleaved, InterleavedBuf, Buf};
use cirrus_protobuf::api::AudioDataRes;
use ringbuf::{SharedRb, Producer};
use rubato::Resampler;
use tokio::{runtime::Handle, sync::RwLock};
use tokio_stream::StreamExt;

use crate::{dto::AudioSource, request};

use super::{packet::EncodedBuffer, stream::StreamPlaybackContext};

pub struct AudioSample {
    inner: Arc<Mutex<AudioSampleInner>>,
}

impl AudioSample {
    pub fn new(
        source: AudioSource,
        // host_stream_config: HostStreamConfig,
        output_stream_config: &cpal::StreamConfig,
        ringbuf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
        stream_playback_context: &Arc<RwLock<StreamPlaybackContext>>,
    ) -> Result<Self, anyhow::Error> {

        Ok(Self {
            inner: Arc::new(
                Mutex::new(
                    AudioSampleInner::new(
                        source,
                        output_stream_config,
                        ringbuf_producer,
                        stream_playback_context,
                    )?
                )
            )
        })
    }

    pub fn fetch_buffer(
        &self,
        rt_handle: &Handle,
    ) -> Result<(), anyhow::Error> {
        self.inner.lock().unwrap().fetch_buffer(rt_handle)
    }

    pub fn start_process_buf(
        &self,
    ) {
        let _inner = self.inner.clone();
        std::thread::spawn(move || {
            _inner.lock().unwrap().process_buf()
        });
    }

    pub fn update_from_output_stream_config(&self) {
        todo!()
    }
}

pub struct AudioSampleInner {
    pub source: AudioSource,
    context: AudioSampleContext,

    sample_frame_buf: Vec<VecDeque<f32>>,
    packet_buf: Arc<RwLock<EncodedBuffer>>,

    packet_decoder: opus::Decoder,
    resampler: AudioResampler,

    audio_stream_buf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    // pub tx: mpsc::Sender<&'static str>,
    // rx: mpsc::Receiver<&'static str>,
    // audio_data_res_tx: mpsc::Sender<AudioDataRes>,
    // resampler: AudioResampler,
}

impl AudioSampleInner {
    pub fn new(
        source: AudioSource,
        output_stream_config: &cpal::StreamConfig,
        audio_stream_buf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
        stream_playback_context: &Arc<RwLock<StreamPlaybackContext>>
        // audio_data_res_tx: mpsc::Sender<AudioDataRes>,
    ) -> Result<Self, anyhow::Error> {
        let mut sample_frame_buf: Vec<VecDeque<f32>> = Vec::with_capacity(2);

        for _ in 0..output_stream_config.channels {
            sample_frame_buf.push(VecDeque::new());
        }

        let packet_buf = EncodedBuffer::new(source.content_packets);
        let packet_decoder = opus::Decoder::new(48_000, opus::Channels::Stereo).unwrap();
        
        // let (tx, rx) = mpsc::channel(64);
        // let (tx, rx) = mpsc::channel();

        Ok(Self {
            source,
            sample_frame_buf,
            packet_buf: Arc::new(RwLock::new(packet_buf)),
            packet_decoder,
            resampler: AudioResampler::new(
                output_stream_config.sample_rate.0.try_into()?,
                960
            )?,
            context: AudioSampleContext::default(),
            audio_stream_buf_producer,
            // tx,
            // rx,
            // audio_data_res_tx,
        })
    }

    pub fn fetch_buffer(
        &mut self,
        rt_handle: &Handle,
    ) -> Result<(), anyhow::Error> {
        let packet_buf = self.packet_buf.blocking_read();

        let last_buf_chunk = packet_buf.buf_chunk_info.get(&packet_buf.last_node_id).unwrap();

        let content_packets = packet_buf.content_packets;
        let last_chunk_end_idx = last_buf_chunk.lock().unwrap().end_idx;
        let chunks_num = packet_buf.get_chunks_num_from_current();

        let audio_tag_id = self.source.id.clone();
        let mut last_idx = 0;

        drop(packet_buf);

        let _packet_buf = self.packet_buf.clone();

        rt_handle.spawn(async move {
            let mut audio_data_stream = request::get_audio_data_stream(
                "http://localhost:50000", 
                &None, 
                &audio_tag_id, 
                0, 
                30000, 
                2
            ).await.unwrap();

            while let Some(res) = audio_data_stream.next().await {
                let audio_data = match res {
                    Ok(data) => data,
                    Err(e) => {
                        println!("err: {}", e);
                        break;
                    }
                };

                last_idx = audio_data.packet_idx;
                _packet_buf.write().await.insert(audio_data);
                // packet_buf.insert(audio_data);
            }
        });

        Ok(())
    }

    pub fn process_buf(&mut self) {
        let mut packet_idx = 0;

        loop {
            // buffer has not enough spaces to fill buffer
            // wait for consumer consumes buffer
            let processed_sample_len = self.resampler.resampler.output_frames_max() * 2;
            if processed_sample_len > self.audio_stream_buf_producer.free_len() {
                // println!("buffer has not enough spaces to fill buffer");
                std::thread::sleep(Duration::from_millis(50));

                continue;
            }

            // audio data is not fetched  
            let _packet_buf_guard = self.packet_buf.blocking_read();
            let data = _packet_buf_guard.frame_buf.get(&packet_idx);
            if data.is_none() {
                // println!("data is not fetched");

                continue;
            }

            let data = data.unwrap();

            let es_ptr = data.encoded_samples.as_ptr();
            let es_len = data.encoded_samples.len();

            let es = unsafe {
                std::slice::from_raw_parts(es_ptr, es_len)
            };

            let sp_frame_num = data.sp_frame_num;

            drop(_packet_buf_guard);

            let mut decoded_samples = vec![0.; (sp_frame_num*2).try_into().unwrap()];
            let mut decoded_samples = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);

            if let Err(err) = self.packet_decoder.decode_float(
                es,
                &mut decoded_samples.as_interleaved_mut(),
                false
            ) {
                println!("{:?}", err);
            }

            self.resampler.resample(decoded_samples);

            let mut resampled_output = audio::buf::Interleaved::<f32>::with_topology(
                2,
                960
            );

            for ch_idx in 0..2 {
                for (c, s) in resampled_output
                    .get_mut(ch_idx)
                    .unwrap()
                    .iter_mut()
                    .zip(&self.resampler.output_buf[ch_idx])
                {
                    *c = *s;
                }
            }
            
            self.audio_stream_buf_producer.push_slice(resampled_output.as_interleaved());

            packet_idx += 1;

            std::thread::sleep(Duration::from_millis(10));
        }
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
    pub output_buf: Vec<Vec<f32>>,
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

    fn resample<'a>(&mut self, decoded_samples: Interleaved<&'a mut [f32]>) {
        let audio_buf_reader = audio::io::Read::new(decoded_samples);

        for ch_idx in 0..2 {
            let audio_ch_buf = audio_buf_reader
                .get(ch_idx)
                .unwrap()
                .iter()
                .collect::<Vec<_>>();

            self.input_buf[ch_idx] = audio_ch_buf;
        }

        self.resampler.process_into_buffer(
            &self.input_buf,
            &mut self.output_buf,
            None
        ).unwrap();
    }
}