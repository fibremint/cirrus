use std::{collections::VecDeque, sync::{mpsc, Arc}, mem::MaybeUninit};
use audio::{InterleavedBufMut, wrap::Interleaved, InterleavedBuf, Buf};
use cirrus_protobuf::api::AudioDataRes;
use ringbuf::{SharedRb, Producer};
use rubato::Resampler;
use tokio::{runtime::Handle, sync::RwLock};
use tokio_stream::StreamExt;

use crate::{dto::AudioSource, request};

use super::{packet::EncodedBuffer, context::HostStreamConfig};

pub struct AudioSample {
    pub source: AudioSource,
    context: AudioSampleContext,

    sample_frame_buf: Vec<VecDeque<f32>>,
    packet_buf: Arc<RwLock<EncodedBuffer>>,

    packet_decoder: opus::Decoder,
    resampler: AudioResampler,

    ringbuf_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    // pub tx: mpsc::Sender<&'static str>,
    // rx: mpsc::Receiver<&'static str>,
    // audio_data_res_tx: mpsc::Sender<AudioDataRes>,
    // resampler: AudioResampler,
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
            packet_buf: Arc::new(RwLock::new(packet_buf)),
            packet_decoder,
            resampler: AudioResampler::new(host_stream_config.sample_rate.try_into()?, 960)?,
            context: AudioSampleContext::default(),
            ringbuf_producer,
            // tx,
            // rx,
            // audio_data_res_tx,
        })
    }

    pub fn fetch_buffer(
        &mut self,
        rt_handle: &Arc<Handle>,
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
            let mut audio_data_steram = request::get_audio_data_stream(
                "http://localhost:50000", 
                &None, 
                &audio_tag_id, 
                0, 
                30000, 
                2
            ).await.unwrap();

            while let Some(res) = audio_data_steram.next().await {
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
            let _packet_buf_guard = self.packet_buf.blocking_read();
            let data = _packet_buf_guard.frame_buf.get(&packet_idx);
            
            if data.is_none() {
                continue;
            }

            let a = self.resampler.resampler.output_frames_next();
            let b = self.ringbuf_producer.free_len();

            // if self.ringbuf_producer.is_full() {
            //     // println!("buf is full, continue");
            //     continue;
            // }

            if a * 2 > b {
                // println!("not enough spaces to fill buf, continue");
                continue;
            }

            let data = data.unwrap();

            // let decoded_samples = decode_opus_packet(data.unwrap(), &mut self.packet_decoder);

            let mut decoded_samples = vec![0.; (data.sp_frame_num*2).try_into().unwrap()];
            let mut decoded_samples = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);
        
            // let mut ds = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);
            // let mut ds = audio::wrap::interleaved(&mut[0.; (audio_data.sp_frame_num * 2).try_into().unwrap()], 2);
        
            if let Err(err) = self.packet_decoder.decode_float(
                &data.encoded_samples,
                &mut decoded_samples.as_interleaved_mut(),
                false
            ) {
                println!("{:?}", err);
            }

            // let audio_buf_reader = audio::io::Read::new(decoded_samples);
            // // let mut resampler_input_buf = self.resampler_input_buf.lock().unwrap();

            // for ch_idx in 0..2 {
            //     let audio_ch_buf = audio_buf_reader
            //         .get(ch_idx)
            //         .unwrap()
            //         .iter()
            //         .collect::<Vec<_>>();

            //     self.resampler.input_buf[ch_idx] = audio_ch_buf;
            // }

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
            
            self.ringbuf_producer.push_slice(resampled_output.as_interleaved());

            packet_idx += 1;
        }
    }
}

// fn decode_opus_packet<'a>(audio_data: &'a AudioDataRes, decoder: &'a mut opus::Decoder) -> audio::wrap::Interleaved<&'a mut [f32]> {
//     // const a: usize = audio_data.sp_frame_num * 2;
//     // let t = [0.; a];
//     let mut decoded_samples = vec![0.; (audio_data.sp_frame_num*2).try_into().unwrap()];
//     let mut ds = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);

//     // let mut ds = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);
//     // let mut ds = audio::wrap::interleaved(&mut[0.; (audio_data.sp_frame_num * 2).try_into().unwrap()], 2);

//     if let Err(err) = decoder.decode_float(
//         &audio_data.encoded_samples,
//         &mut ds.as_interleaved_mut(),
//         false
//     ) {
//         println!("{:?}", err);
//     }

//     ds
// }


// fn decode_opus_packet<'a>(audio_data: &'a AudioDataRes, decoder: &'a mut opus::Decoder) -> audio::wrap::Interleaved<&'a mut [f32]> {
//     // const a: usize = audio_data.sp_frame_num * 2;
//     // let t = [0.; a];
//     let mut decoded_samples = vec![0.; (audio_data.sp_frame_num*2).try_into().unwrap()];
//     let mut ds = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);

//     // let mut ds = audio::wrap::interleaved(decoded_samples.as_mut_slice(), 2);
//     // let mut ds = audio::wrap::interleaved(&mut[0.; (audio_data.sp_frame_num * 2).try_into().unwrap()], 2);

//     if let Err(err) = decoder.decode_float(
//         &audio_data.encoded_samples,
//         &mut ds.as_interleaved_mut(),
//         false
//     ) {
//         println!("{:?}", err);
//     }

//     ds
// }

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