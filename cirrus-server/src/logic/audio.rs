use std::{
    fs::File,
    path::{Path, PathBuf}, collections::{HashMap, HashSet}, io::{BufReader, Read, Seek, SeekFrom}, env, sync::{Mutex, Arc},
};
use itertools::Itertools;

use aiff::reader::AiffReader;
use bson::oid::ObjectId;
use chrono::{Utc, TimeZone};
use cirrus_protobuf::api::{
    AudioMetaRes, AudioTagRes
};

use mongodb::bson;
use rubato::Resampler;
use tonic::IntoRequest;
use walkdir::{DirEntry, WalkDir};

use crate::{
    util, 
    model::{self, document}, settings::Settings,
};

use symphonia::core::{codecs::{CODEC_TYPE_NULL, DecoderOptions, Decoder}, audio::{AudioBufferRef, Signal, SampleBuffer, RawSampleBuffer}, formats::{FormatReader, SeekMode, SeekTo}, units::{TimeStamp, Time}, probe::ProbeResult};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

use opus;
use audio;
use audio::ReadBuf as _;
use audio::{io, wrap, WriteBuf, ExactSizeBuf, ChannelMut, Channels, Channel};

use super::packet::Packets;

pub struct AudioFile {}

impl AudioFile {
    pub async fn read_meta(
        mongodb_client: mongodb::Client,
        audio_tag_id: &str
    ) -> Result<AudioMetaRes, String> {
        let current_dir = env::current_dir().unwrap();
        let server_config_path = current_dir.join("configs/cirrus/server.toml");
        
        let settings = Settings::new(&server_config_path).unwrap();

        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();
        let audio_file = model::audio::AudioFile::find_by_audio_tag_id(mongodb_client.clone(), audio_tag_id).await.unwrap();

        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(String::from("failed to retrieve audio file information")),
        };

        let file = match File::open(audio_file.get_path()) {
            Ok(file) => file,
            Err(_) => return Err(String::from("failed to load file")),
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let hint = Hint::new();

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts).unwrap();

        let format = probed.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        let bit_rate = track.codec_params.bits_per_sample.unwrap();
        let channels = track.codec_params.channels.unwrap().count();
        let sample_rate = track.codec_params.sample_rate.unwrap();
        let content_length = 
            track.codec_params.n_frames.unwrap() as f64 / sample_rate as f64;

        let sample_frame_packet_dur = 
            settings.audio_sample_frame_packet.len as f64 
                / settings.audio_sample_frame_packet.sample_rate as f64;

        let sample_frame_packet_num = (content_length / sample_frame_packet_dur).ceil() as u32;

        Ok(AudioMetaRes {
            content_length,
            sp_packets: sample_frame_packet_num,
            packet_dur: sample_frame_packet_dur,
            orig_sample_rate: sample_rate,
            orig_bit_rate: bit_rate,
            channels: channels.try_into().unwrap(),
        })
    }

    // pub async fn get_audio_source(
    //     mongodb_client: mongodb::Client,
    //     audio_tag_id: &str,
    // ) -> File {

    // }

    pub async fn get_audio_sample_iterator(
        mongodb_client: mongodb::Client,
        audio_tag_id: &str,
        packet_start_idx: usize,
        packet_num: usize,
        channels: u32,
        // packet_start_ts: u64,
        seek_start_pkt_idx: usize,
        seek_start_pkt_ts: u64,
        // pkt_read_start_offset: u32,
        opus_encoder: Arc<Mutex<opus::Encoder>>,
    ) -> Result<Packets, anyhow::Error> {
        let current_dir = env::current_dir().unwrap();
        let server_config_path = current_dir.join("configs/cirrus/server.toml");
        
        let settings = Settings::new(&server_config_path).unwrap();
        
        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();
        let audio_file = model::audio::AudioFile::find_by_audio_tag_id(mongodb_client.clone(), audio_tag_id).await.unwrap();

        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(anyhow::anyhow!("failed to retrieve audio file information")),
        };

        let file = match File::open(audio_file.get_path()) {
            Ok(file) => file,
            Err(_err) => return Err(anyhow::anyhow!("failed to load file")),
        };

        // let file = File::open("D:\\tmp\\file_example_WAV_10MG.wav").unwrap();

        let sample_frame_packet_dur = 
        settings.audio_sample_frame_packet.len as f64 
            / settings.audio_sample_frame_packet.sample_rate as f64;

        let packets = Packets::new(
            file,
            opus_encoder,
            packet_start_idx,
            packet_num,
            seek_start_pkt_idx,
            seek_start_pkt_ts,
            settings.audio_sample_frame_packet.len.try_into().unwrap(),
            settings.audio_sample_frame_packet.sample_rate.try_into().unwrap(),
        )?;

        Ok(packets)

        // let audio_sample_iter = AudioSampleIterator::new(
        //     file,
        //     packet_start_idx,
        //     packet_num,
        //     sample_frame_packet_dur,
        //     settings.audio_sample_frame_packet.len,
        //     settings.audio_sample_frame_packet.sample_rate,
        //     channels.try_into().unwrap(),
        //     // packet_start_ts
        //     seek_start_pkt_idx,
        //     seek_start_pkt_ts,
        //     // pkt_read_start_offset,
        //     opus_encoder,
        // )?;

        // Ok(audio_sample_iter)
    }
}

pub struct SampleFramePacket {
    pub packet_idx: u32,
    pub sample_frame_duration: f64,
    pub sample_frame_num: u32,

    pub packet_start_ts: u64,
    // pub packet_read_start_offset: u32,
    // pub remain_frames: i32,
    // pub original_frame_len: u16,
    // pub padded_frame_start_pos: u16,
    pub encoded_data: Vec<u8>
}

pub struct AudioSampleIterator {
    // duration: f64,
    packet_num: u32,
    packet_dur: f64,
    packet_sample_frame_num: u32,
    packet_sample_rate: u32,
    channel_size: usize,

    ch_sample_buf: Vec<Vec<f32>>,
    decoder: Box<dyn Decoder>,
    // probed: ProbeResult,
    format: Box<dyn FormatReader>,
    resampler: rubato::FftFixedOut<f32>,
    resampler_in_buf: Vec<Vec<f32>>,
    resampler_out_buf: Vec<Vec<f32>>,

    orig_sample_rate: u32,
    
    // opus_encoder: opus::Encoder,
    decoded_samples: Vec<Vec<f32>>,
    
    seek_packet_cnt: u32,
    packet_seek_start_idx: u32,

    // read_start_offset: usize,

    seek_start_pkt_idx: u64,
    seek_start_pkt_ts: u64,

    packet_start_idx: u32,
    seek_ts: u64,

    iter_os_val: u32,

    resolved_first_read_offset: bool,
    opus_encoder: Arc<Mutex<opus::Encoder>>,

    pkt_start_ts: u64,
    pkt_dur: u64,
}

impl AudioSampleIterator {
    pub fn new(
        source: File,
        packet_start_idx: u32,
        packet_num: u32,
        packet_dur: f64,
        packet_sample_frame_num: u32,
        packet_sample_rate: u32,
        channels: u32,
        // packet_start_ts: u64,
        seek_start_pkt_idx: u64,
        seek_start_pkt_ts: u64,
        // pkt_read_start_offset: u32,
        opus_encoder: Arc<Mutex<opus::Encoder>>,
    ) -> Result<Self, anyhow::Error> {
        // source.seek(SeekFrom::Start(1453024)).unwrap();

        let mss = MediaSourceStream::new(Box::new(source), Default::default());
        let hint = Hint::new();

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts).unwrap();
        // let t = probed.format;
        let mut format = probed.format;
            // Find the first audio track with a known (decodeable) codec.
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");
            

        // Use the default options for the decoder.
        let dec_opts: DecoderOptions = Default::default();
        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)
                                            .expect("unsupported codec");

        let mut ch_sample_buf = Vec::with_capacity(channels.try_into().unwrap());
        for _ in 0..channels {
            ch_sample_buf.push(vec![0.; 0]);
        }

        let codec_sample_rate = track.codec_params.sample_rate.unwrap();

        let resampler = rubato::FftFixedOut::new(
            codec_sample_rate.try_into().unwrap(), 
            packet_sample_rate.try_into().unwrap(),
            packet_sample_frame_num.try_into().unwrap(), 
            2,
            2
        )?;

        // let sample_frame_start_sec = packet_start_idx as f64 * packet_dur;

        // format.seek(
        //     SeekMode::Accurate,
        //     SeekTo::Time { 
        //         time: Time::from(sample_frame_start_sec), 
        //         track_id: Some(track.id) 
        //     }
        // )?;

        
        format.seek(
            SeekMode::Coarse,
            SeekTo::TimeStamp {
                ts: seek_start_pkt_ts,
                track_id: track.id,
             }
        )?;

        // self.format.seek(
        //     SeekMode::Coarse,
        //     SeekTo::TimeStamp { 
        //         ts: start_ts,
        //         track_id: track.id,
        //      }
        // )?;
        
        // source.seek(SeekFrom::Start(1453024)).unwrap();

        let resampler_in_buf = resampler.input_buffer_allocate();
        let resampler_out_buf = resampler.output_buffer_allocate();

        // let opus_encoder = opus::Encoder::new(packet_sample_rate, opus::Channels::Stereo, opus::Application::Audio)?;

        let mut decoded_samples = Vec::with_capacity(2);
        for _ in 0..2 {
            decoded_samples.push(Vec::new());
        }

        let mut audio_sample_iter = Self {
            // duration,
            packet_num,
            packet_dur,
            packet_sample_frame_num,
            packet_sample_rate,
            channel_size: channels.try_into().unwrap(),

            ch_sample_buf,
            // mss,
            decoder,
            // probed,
            format,
            resampler,
            resampler_in_buf,
            resampler_out_buf,
            orig_sample_rate: codec_sample_rate,
            // opus_encoder,
            decoded_samples,
            seek_packet_cnt: Default::default(),
            packet_seek_start_idx: packet_start_idx,

            // read_start_offset: pkt_read_start_offset as usize,

            seek_start_pkt_idx,
            seek_start_pkt_ts,
            packet_start_idx,

            seek_ts: seek_start_pkt_ts,

            iter_os_val: 0,

            resolved_first_read_offset: false,
            opus_encoder,

            pkt_start_ts: 0,
            pkt_dur: 0,
        };

        Ok(audio_sample_iter)

    }
}

impl Iterator for AudioSampleIterator {
    type Item = SampleFramePacket;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seek_packet_cnt == self.packet_num {
            return None;
        }

        let mut enc_output = Vec::new();
        let rs_input_frame_next = self.resampler.input_frames_next();

        // let pkt_idx = self.packet_seek_start_idx;

        let mut packet_start_ts = 0;
        let mut packet_dur = 0;

        // let mut read_offset = 0;
        // let mut first_frame_ts = u32::MAX;

        // let prev_decoded_samples_len = self.decoded_samples[0].len();

        // let mut read_start_offset = 0;

        while self.decoded_samples[0].len() < rs_input_frame_next {
        // while self.decoded_samples[0].len() - self.read_start_offset <= rs_input_frame_next  {
            let mut read_start_offset = 0;
            // self.read_start_offset = 0;

            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(e) => {
                    break
                },
            };

            packet_start_ts = packet.ts();
            packet_dur = packet.dur();
            assert_eq!(self.seek_ts, packet_start_ts);

            self.seek_ts += packet_dur;
            self.pkt_start_ts = packet_start_ts;
            self.pkt_dur = packet_dur;

            let des_pkt_size_at_idx = self.resampler.input_frames_max() as u32 * (self.packet_start_idx);

            // if !self.resolved_first_read_offset && self.packet_start_idx > 0 {
            //     self.read_start_offset = (self.read_start_offset + self.resampler.input_frames_max()) % packet_dur as usize;
            // }

            if packet_start_ts + packet_dur <= des_pkt_size_at_idx as u64 {
                // self.read_start_offset = (self.read_start_offset + self.resampler.input_frames_max()) % packet_dur as usize;
                
                continue;
                // self.read_start_offset = self.resampler.input_frames_max();
                // read_start_offset = self.resampler.input_frames_max();
            }

            if !self.resolved_first_read_offset {
                read_start_offset = self.resampler.input_frames_max() * self.packet_start_idx as usize - self.pkt_start_ts as usize;
            }

            // if self.pkt_start_ts + self.pkt_dur < ((self.packet_start_idx+1) * self.resampler.input_frames_max() as u32) as u64 {
            //     continue;
            // }

            // let sample_frame_ts = (packet_start_ts + packet_dur) as u64;
            // let sample_idx = sample_frame_ts as f32 / self.resampler.input_frames_max() as f32;
            // let sample_idx = sample_idx.floor() as u32;

            // if sample_idx < self.packet_start_idx {
            //     continue;
            // }

            // if first_frame_ts == u32::MAX {
            //     first_frame_ts = packet_start_ts as u32;

            //     if !self.resolved_first_read_offset {
            //         read_offset = self.resampler.input_frames_max() as u32 * pkt_idx - first_frame_ts;
            //     }
            // }

            // if !self.resolved_first_read_offset && self.packet_start_idx > 0 {

            //     // let packet_pos = packet_start_ts + packet_dur;
            //     // let resampler_req_samples = self.resampler.input_frames_max() as u32 * (self.packet_seek_start_idx + self.seek_packet_cnt);
            //     // let prev_pkt_size = self.resampler.input_frames_max() as u32 * (self.seek_start_pkt_idx) as u32;
            //     // read_start_offset = (packet_pos as u32 - prev_pkt_size).try_into().unwrap();
            //     // read_start_offset = packet_start_ts -
            //     let calc = self.resampler.input_frames_max() as u32 * (self.packet_start_idx) - packet_start_ts as u32;
            //     // let calc = (packet_start_ts+packet_dur) as u32 - (self.resampler.input_frames_max() as u32 * (self.packet_start_idx-1) as u32) as u32;
            //     // let calc = self.resampler.input_frames_max() as u32 - calc;
            //     read_start_offset = calc.try_into().unwrap();
                
            //     // self.read_start_offset += (packet_pos - resampler_req_samples as u64) as usize;
            //     // self.read_start_offset = (self.resampler.input_frames_max() as u32 * self.packet_seek_start_idx - packet_start_ts as u32) as usize;
            // }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(_) => break,
            };

            let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            sample_buf.copy_planar_ref(decoded);

            let samples = sample_buf.samples();
            let sample_frame_len = samples.len() / 2;

            self.decoded_samples[0].extend_from_slice(&samples[..sample_frame_len]);
            self.decoded_samples[0].drain(..read_start_offset as usize);
            self.decoded_samples[1].extend_from_slice(&samples[sample_frame_len..]);
            self.decoded_samples[1].drain(..read_start_offset as usize);

            // read_offset = 0;
            // read_start_offset = 0;
            // self.read_start_offset = 0;

            self.resolved_first_read_offset = true;
        }

        // let pkt_idx = self.packet_seek_start_idx+self.seek_packet_cnt;

        // let next_pkt_seek_ts = self.pkt_start_ts;
        // let next_pkt_seek_ts = 
        //     if {  }
        // let next_pkt_read_offset = 
        //     (pkt_idx+1) * self.resampler.input_frames_max() as u32 - (next_pkt_seek_ts + packet_dur) as u32;


        let mut rs_input = Vec::with_capacity(2);
        let dc_sp_frame_len = self.decoded_samples[0].len() as i32;

        let zero_pad_len = std::cmp::max(rs_input_frame_next as i32 - dc_sp_frame_len, 0);
        
        for ch_idx in 0..2 {
            let ch_sp_drain_len = std::cmp::min(rs_input_frame_next, dc_sp_frame_len.try_into().unwrap());

            let mut ch_rs_input = self.decoded_samples[ch_idx].drain(..ch_sp_drain_len).collect_vec();
            
            if zero_pad_len > 0 {
                ch_rs_input.extend_from_slice(&vec![0.; zero_pad_len.try_into().unwrap()]);
            }

            rs_input.push(ch_rs_input);
        }

        // let next_pkt_seek_ts = self.pkt_start_ts;

        // let next_pkt_seek_ts = 
        //     if self.decoded_samples[0].len() == self.resampler.input_frames_max() || (pkt_idx > 0 && self.decoded_samples[0].len() == 0)
        //         { self.pkt_start_ts + self.pkt_dur }
        //     else
        //         { self.pkt_start_ts };

        let next_pkt_seek_ts = 
            if self.decoded_samples[0].len() > 0
                { self.pkt_start_ts }
            else
                { self.pkt_start_ts + self.pkt_dur };

        // let next_pkt_read_offset = self.decoded_samples[0].len() as u32;
        // let next_pkt_read_offset = self.decoded_samples[0].len() % self.resampler.input_frames_max(); 
        // let next_pkt_read_offset = 0;
        // let next_pkt_read_offset = self.resampler.input_frames_max() * (pkt_idx+1) as usize - next_pkt_seek_ts as usize;
        // let next_pkt_read_offset = next_pkt_read_offset % self.resampler.input_frames_max();

        // let next_pkt_read_offset = 
        //     (self.pkt_start_ts + self.pkt_dur) as u32 - (pkt_idx+1) * self.resampler.input_frames_max() as u32;

        // let next_pkt_read_offset = 
        //     (pkt_idx+1) * self.resampler.input_frames_max() as u32 - self.pkt_start_ts as u32;

        // let next_pkt_read_offset = (next_pkt_seek_ts + packet_dur) as u32 - 
        //     (pkt_idx+1) * self.resampler.input_frames_max() as u32;


        // // for test
        // let packet_idx = self.packet_seek_start_idx + self.seek_packet_cnt;
        // println!("rs input values; pkt idx: {}", packet_idx);
        
        // for ch_idx in 0..2 {
        //     println!("ch {}", ch_idx);
        //     for (idx, item) in rs_input[ch_idx].iter().enumerate() {
        //         println!("idx {}: {}", idx, item);
        //     }
            
        //     println!("");
        // }

        self.resampler.process_into_buffer(
            &rs_input, 
            &mut self.resampler_out_buf, 
            None
        ).unwrap();

        let mut resampled_output = audio::Interleaved::<f32>::with_topology(2, self.resampler.output_frames_max());

        for ch_idx in 0..2 {
            for (c, s) in resampled_output
                .get_mut(ch_idx)
                .unwrap()
                .iter_mut()
                .zip(&self.resampler_out_buf[ch_idx])
            {
                *c = *s;
            }
        }

        let encoded = self.opus_encoder.lock().unwrap().encode_vec_float(resampled_output.as_slice(), 4000).unwrap();
        enc_output.extend(encoded);

        let packet_idx = self.packet_seek_start_idx + self.seek_packet_cnt;
        self.seek_packet_cnt += 1;

        Some(        
            SampleFramePacket {
                packet_idx,
                sample_frame_duration: self.packet_dur,
                sample_frame_num: 960,
                packet_start_ts: next_pkt_seek_ts as u64,
                // packet_start_ts: r,
                // packet_read_start_offset: remain_frames as u32,
                // packet_read_start_offset: next_pkt_read_offset.try_into().unwrap(),
                // remain_frames,
                encoded_data: enc_output,
            }
        )
    }
}

pub struct AudioLibrary {}

impl AudioLibrary {
    // * path not exist -> return not found
    // * path is added already -> return added already
    fn get_audio_library_entries(path: &Path, audio_types: &Vec<&str>) -> Vec<DirEntry> {
        let audio_library_entries: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|item| item.ok()
                .and_then(|entry| entry.path().is_dir().then(|| entry))
                .and_then(|entry2| {
                    let mut current_dir = std::fs::read_dir(entry2.path()).unwrap();
                    current_dir.any(|content_entry| {
                        match content_entry.unwrap().path().extension() {
                            Some(extension) => return audio_types.contains(&extension.to_str().unwrap()),
                            None => false,
                        }
                    }).then(|| entry2)
                })
            )
            .collect();

        audio_library_entries
    }

    fn get_audio_file_paths(current_path: &Path, audio_types: &Vec<&str>) -> Vec<PathBuf> {
        let audio_file_dir = std::fs::read_dir(current_path).unwrap();
        let audio_file_paths: Vec<_> = audio_file_dir
            .into_iter()
            .filter_map(|item| item.ok()
                .and_then(|entry| entry.path().is_file().then(|| entry.path()))
                .and_then(|pathbuf| {
                    match pathbuf.extension() {
                        Some(file_extension) => {
                            let file_extension = file_extension.to_str().unwrap();
                            return audio_types.contains(&file_extension).then(|| pathbuf)
                        },
                        None => None,
                    }
                })
            )
            .collect();
        
        audio_file_paths
    }

    pub async fn add_audio_library(
        mongodb_client: mongodb::Client,
        library_root: &Path
    ) -> Result<String, String> {
        if !library_root.exists() {
            return Err(String::from("not exists"))
        }

        if model::audio::AudioLibraryRoot::check_exists_by_path(mongodb_client.clone(), library_root).await {
            return Err(format!("path '{:?}' already exists", library_root))
        }

        let audio_types = vec!["aiff"];

        let audio_library_entries = Self::get_audio_library_entries(library_root, &audio_types);
        let audio_file_docs: Vec<_> = audio_library_entries
            .iter()
            .map(|item| Self::get_audio_file_paths(item.path(), &audio_types))
            .flat_map(|item| item)
            .map(|item| document::audio::AudioFile::create_from_path(&item))
            .collect();
        
        let library_docs: Vec<_> = audio_library_entries
            .iter()
            .map(|item| document::audio::AudioLibrary::create_from_path(&item.path()))
            .collect();

        let audio_library_root_doc = document::audio::AudioLibrary::create_from_path(&library_root);

        let library_create_res = model::audio::AudioLibraryRoot::create(mongodb_client.clone(), audio_library_root_doc).await;

        if !library_docs.is_empty() {
            model::audio::AudioLibrary::create_many(mongodb_client.clone(), library_docs).await.unwrap();
        }
        
        if !audio_file_docs.is_empty() {
            model::audio::AudioFile::create_many(mongodb_client.clone(), &audio_file_docs).await.unwrap();
        }

        match library_create_res {
            Ok(res) => return Ok(format!("{:?}", res.inserted_id)),
            Err(_err) => return Err(format!("failed to create library {:?}", library_root)),
        }
    }

    pub async fn remove_audio_library(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<String, String> {
        if !model::audio::AudioLibraryRoot::check_exists_by_path(mongodb_client.clone(), path).await {
            return Err(format!("path '{:?}' not exists", path))
        }

        let mut delete_tag_count = 0;
        let mut delete_file_count = 0;
        let mut delete_library_count = 0;

        let delete_audio_libraries = model::audio::AudioLibrary::get_by_path(mongodb_client.clone(), path).await.unwrap();
        for delete_audio_library in delete_audio_libraries.iter() {
            let delete_audio_library_path = util::path::materialized_to_path(&delete_audio_library.path.as_ref().unwrap());
            let delete_audio_library_path = Path::new(&delete_audio_library_path);
            let audio_files = model::audio::AudioFile::get_self_by_library_path(mongodb_client.clone(), delete_audio_library_path, false).await.unwrap();
            let delete_audio_tag_ids: Vec<_> = audio_files.iter()
                .filter_map(|item| item.audio_tag_refer)
                .collect();
    
            let audio_tag_delete_res = model::audio::AudioTag::delete_by_ids(mongodb_client.clone(), &delete_audio_tag_ids).await.unwrap();
            delete_tag_count += audio_tag_delete_res.deleted_count;
    
            let audio_file_delete_res = model::audio::AudioFile::delete_by_selfs(mongodb_client.clone(), &audio_files).await.unwrap();
            delete_file_count += audio_file_delete_res.deleted_count;
    
            let library_delete_res = model::audio::AudioLibrary::delete_by_path(mongodb_client.clone(), delete_audio_library_path).await.unwrap();
            delete_library_count += library_delete_res.deleted_count;
        }

        model::audio::AudioLibraryRoot::delete_by_path(mongodb_client.clone(), path).await;

        Ok(format!("deleted tag count: {}, deleted file count: {}, deleted library count: {}", delete_tag_count, delete_file_count, delete_library_count))
    }

    pub async fn analyze_audio_library(
        mongodb_client: mongodb::Client,
    ) -> Result<(), String> {
        let audio_libraries = model::audio::AudioLibraryRoot::get_all(mongodb_client.clone()).await;

        for audio_library in audio_libraries.into_iter() {
            let audio_files = model::audio::AudioFile::get_self_by_library_path(mongodb_client.clone(), Path::new(&audio_library.id), true).await.unwrap();

            for audio_file in audio_files.iter() {
                let parent_path = util::path::materialized_to_path(&audio_file.parent_path);
                let audio_tag = Self::create_audio_tag(None, &parent_path, &audio_file.filename);
                let audio_tag_id = audio_tag.id.clone();
                
                match model::audio::AudioTag::create(mongodb_client.clone(), audio_tag).await {
                    Ok(_) => (),
                    Err(err) => return Err(format!("{}", err)),
                }

                let update_res = model::audio::AudioFile::set_audio_tag_refer(mongodb_client.clone(), &audio_file.id.unwrap(), &audio_tag_id.unwrap()).await.unwrap();
                println!("ur: {:?}", update_res);
            }
        }

        Ok(())
    }

    pub async fn refresh_audio_library(
        mongodb_client: mongodb::Client,
    ) -> Result<(), String> {
        let audio_library_roots = model::audio::AudioLibraryRoot::get_all(mongodb_client.clone()).await;
        let audio_types = vec!["aiff"];

        for audio_library_root in audio_library_roots.iter() {
            let audio_libraries = model::audio::AudioLibrary::get_by_path(mongodb_client.clone(), Path::new(&audio_library_root.id)).await.unwrap();
            let audio_libraries: HashMap<_, _> = audio_libraries.iter()
                .map(|item| (item.id.as_str(), item))
                .collect();
            let local_audio_library_entreis = Self::get_audio_library_entries(Path::new(&audio_library_root.id), &audio_types);

            let audio_libraries_keys: HashSet<_> = audio_libraries
                .iter()
                .map(|(k, _)| util::path::replace_with_common_separator(k))
                .collect();
            let local_audio_libraries_keys: HashSet<_> = local_audio_library_entreis
                .iter()
                .map(|item| {
                    util::path::replace_with_common_separator(item.path().to_str().unwrap())
                })
                .collect();

            let new_library_pathstrs: HashSet<_> = local_audio_libraries_keys.difference(&audio_libraries_keys).cloned().collect();
            let deleted_library_pathstrs: HashSet<_> = audio_libraries_keys.difference(&local_audio_libraries_keys).cloned().collect();
            let managed_library_pathstrs: HashSet<_> = audio_libraries_keys.difference(&deleted_library_pathstrs).collect();
            let updated_local_libraries: Vec<_> = managed_library_pathstrs.into_iter()
                .filter_map(|item| audio_libraries.get(item.as_str())
                    .and_then(|audio_library| audio_library.check_modified().then(|| audio_library)))
                .collect();

            println!("nl: {:?}, dl: {:?}, ull: {:?}", new_library_pathstrs, deleted_library_pathstrs, updated_local_libraries);

            if !new_library_pathstrs.is_empty() {
                let new_audio_file_docs: Vec<_> = new_library_pathstrs
                    .iter()
                    .map(|item| Self::get_audio_file_paths(Path::new(item), &audio_types))
                    .flat_map(|item| item)
                    .map(|item| document::audio::AudioFile::create_from_path(&item))
                    .collect();

                let new_library_docs: Vec<_> = new_library_pathstrs
                    .iter()
                    .map(|item| document::audio::AudioLibrary::create_from_path(Path::new(&item)))
                    .collect();

                model::audio::AudioLibrary::create_many(mongodb_client.clone(), new_library_docs).await.unwrap();

                model::audio::AudioFile::create_many(mongodb_client.clone(), &new_audio_file_docs).await.unwrap();
    
            }

            if !deleted_library_pathstrs.is_empty() {
                for deleted_library_pathstr in deleted_library_pathstrs.iter() {
                    println!("sync delete audio library: {:?}", deleted_library_pathstr);
                    let delted_library_path = Path::new(deleted_library_pathstr);

                    let audio_files = model::audio::AudioFile::get_self_by_library_path(mongodb_client.clone(), delted_library_path, false).await.unwrap();
                    let delete_audio_tag_ids: Vec<_> = audio_files.iter()
                        .filter_map(|item| item.audio_tag_refer)
                        .collect();

                    let _audio_tag_delete_res = model::audio::AudioTag::delete_by_ids(mongodb_client.clone(), &delete_audio_tag_ids).await.unwrap();

                    let _audio_file_delete_res = model::audio::AudioFile::delete_by_selfs(mongodb_client.clone(), &audio_files).await.unwrap();

                    let _library_delete_res = model::audio::AudioLibrary::delete_by_path(mongodb_client.clone(), delted_library_path).await.unwrap();
                }
            }

            if !updated_local_libraries.is_empty() {
                println!("sync updated local libraries: {:?}", updated_local_libraries);
                
                for updated_local_library in updated_local_libraries.iter() {
                    let local_library_path = Path::new(&updated_local_library.id);
                    let audio_files = model::audio::AudioFile::get_self_by_library_path(mongodb_client.clone(), local_library_path.clone(), false).await.unwrap();
                    let audio_filenames: HashSet<_> = audio_files
                        .iter()
                        .map(|item| item.filename.to_owned())
                        .collect();
                    let mut audio_files: HashMap<String, document::audio::AudioFile> = audio_files
                        .into_iter()
                        .map(|item| (item.filename.to_owned(), item))
                        .collect();

                    let local_audio_file_paths = Self::get_audio_file_paths(Path::new(&updated_local_library.id), &audio_types);

                    let local_audio_filenames: HashSet<_> = local_audio_file_paths
                        .iter()
                        .filter_map(|item| item.file_name()
                            .and_then(|item2| item2.to_str()))
                        .map(|item| item.to_owned())
                        .collect();
                    
                    let new_audio_filenames: HashSet<_> = local_audio_filenames.difference(&audio_filenames).cloned().collect();
                    let deleted_audio_filenames: HashSet<_> = audio_filenames.difference(&local_audio_filenames).cloned().collect();
                    let managed_audio_filenames: HashSet<_> = audio_filenames.difference(&deleted_audio_filenames).cloned().collect();

                    let mut updated_audio_files: Vec<document::audio::AudioFile> = vec![];
                    let mut updated_audio_tags: Vec<document::audio::AudioTag> = vec![];

                    for managed_audio_filename in managed_audio_filenames.iter() {
                        let mut audio_file = audio_files.remove(managed_audio_filename).unwrap();
                        if audio_file.check_modified() {
                            match audio_file.audio_tag_refer {
                                Some(audio_tag_id) => {
                                    let parent_path = util::path::materialized_to_path(&audio_file.parent_path);
                                    let updated_audio_tag = Self::create_audio_tag(Some(audio_tag_id), &parent_path, &audio_file.filename);
                                    updated_audio_tags.push(updated_audio_tag);
                                },
                                None => (),
                            }

                            audio_file.update_modified_timestamp();

                            updated_audio_files.push(audio_file);
                        }
                    }

                    let new_audio_file_docs: Vec<_> = new_audio_filenames
                        .iter()
                        .map(|item| {
                            let mut target_path = local_library_path.clone().to_path_buf();
                            target_path.push(item);

                            document::audio::AudioFile::create_from_path(&target_path)
                        })
                        .collect();

                    let delete_audio_file_docs: Vec<_> = deleted_audio_filenames
                        .iter()
                        .filter_map(|item| audio_files.remove(item))
                        .collect();

                    if !new_audio_file_docs.is_empty() {
                        model::audio::AudioFile::create_many(mongodb_client.clone(), &new_audio_file_docs).await.unwrap();
                    }

                    if !delete_audio_file_docs.is_empty() {
                        let deleted_audio_tag_ids: Vec<_> = delete_audio_file_docs
                            .iter()
                            .filter_map(|item| item.audio_tag_refer)
                            .collect();

                        model::audio::AudioTag::delete_by_ids(mongodb_client.clone(), &deleted_audio_tag_ids).await.unwrap();

                        model::audio::AudioFile::delete_by_selfs(mongodb_client.clone(), &delete_audio_file_docs).await.unwrap();
                    }

                    if !updated_audio_files.is_empty() {
                        model::audio::AudioFile::update_self(mongodb_client.clone(), &updated_audio_files).await;
                    }

                    if !updated_audio_tags.is_empty() {
                        model::audio::AudioTag::update_self(mongodb_client.clone(), &updated_audio_tags).await;
                    }

                    let local_library_modified_timestamp = util::path::get_timestamp(&local_library_path);
                    let _update_local_library_res = model::audio::AudioLibrary::update_modified_timestamp(mongodb_client.clone(), &updated_local_library.id, local_library_modified_timestamp).await;
                }
            }

        }
        // collection; libraries - audio library root
        //             libraries-detail - actual contents (sub_dirs, audio_files)



        // filter updated path (by paths' modified datetime)

        Ok(())
    }

    fn create_audio_tag(
        id: Option<ObjectId>,
        parent_path: &str,
        filename: &str,
    ) -> document::audio::AudioTag {
        let mut audio_file_path = Path::new(parent_path).to_path_buf();
        audio_file_path.push(filename);

        let audio_file = File::open(audio_file_path).unwrap();
        let mut aiff = AiffReader::new(audio_file);
        // aiff.read().unwrap();
        aiff.parse().unwrap();

        // let id3v2 = aiff.read_chunk::<aiff::chunks::ID3v2Chunk>(true, false, aiff::ids::AIFF).unwrap();

        let id = match id {
            Some(id) => Some(id),
            None => Some(ObjectId::new())
        };

        let mut id_id3v2 = aiff::ids::ID3.to_vec();
        id_id3v2.push(0);

        let _audio_metadata = if let Some(id3v2) = aiff.read_chunk::<aiff::chunks::ID3v2Chunk>(true, false, &id_id3v2) {
            let date_recorded = match id3v2.tag.date_recorded() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            let date_released = match id3v2.tag.date_released() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            let pictures: Vec<_> = id3v2.tag.pictures()
                .into_iter()
                .map(|item| document::audio::AudioFileMetadataPicture {
                    description: item.description.clone(),
                    mime_type: item.mime_type.clone(),
                    picture_type: item.picture_type.to_string(),
                    data: item.data.to_owned(),
                })
                .collect();

            let artist = match id3v2.tag.artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album = match id3v2.tag.album() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album_artist = match id3v2.tag.album_artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let genre = match id3v2.tag.genre() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let title = match id3v2.tag.title() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let mut audio_tag = document::audio::AudioTag {
                id,
                property_hash: None,
                artist: artist,
                album: album,
                album_artist: album_artist,
                date_recorded,
                date_released,
                disc: id3v2.tag.disc(),
                duration: id3v2.tag.duration(),
                genre: genre,
                pictures: pictures,
                title: title,
                total_discs: id3v2.tag.total_discs(),
                total_tracks: id3v2.tag.total_tracks(),
                track: id3v2.tag.track(),
                year: id3v2.tag.year(),
            };

            audio_tag.property_hash = Some(util::hash::get_hashed_value(&audio_tag));

            return audio_tag

        } else {
            return document::audio::AudioTag {
                id,
                property_hash: None,
                title: Some(filename.clone().to_owned()),
                ..Default::default()
            };
        };
    }
}

pub struct AudioTag {}

impl AudioTag {
    pub async fn list_audio_tags(
        mongodb_client: mongodb::Client,
        max_item_num: u64,
        page: u64,
    ) -> Result<Vec<AudioTagRes>, String> {
        let get_all_res = model::audio::AudioTag::get_all(mongodb_client.clone(), max_item_num as i64, page).await;

        let res: Vec<_> = get_all_res
            .iter()
            .map(|item| AudioTagRes {
                id: item.id.as_ref().unwrap().to_string(),
                artist: item.artist.as_ref().unwrap().to_string(),
                genre: item.genre.as_ref().unwrap().to_string(),
                title: item.title.as_ref().unwrap().to_string(),
            })
            .collect();

        Ok(res)
    }
}