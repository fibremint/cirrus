mod packet;
mod sample;

use std::{
    fs::File, 
};

use bson::oid::ObjectId;

use cirrus_protobuf::api::AudioMetaRes;

use mongodb::bson;

use crate::model::{crud, document};
use crate::settings::Settings;

use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

use self::packet::Packets;

pub struct AudioFile {
    crud_audio_file: crud::AudioFile,
}

impl Default for AudioFile {
    fn default() -> Self {
        Self { 
            crud_audio_file: Default::default(),
        }
    }
}

impl AudioFile {
    pub async fn read_meta(
        &self,
        db: mongodb::Client,
        audio_tag_id: &str
    ) -> Result<AudioMetaRes, anyhow::Error> {        
        let settings = Settings::get()?;

        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();

        let audio_file = self.crud_audio_file
            .single
            .get(
                db.clone(),
                None,
                Some(
                    document::audio::query_audio_tag_referer(&audio_tag_id)
                ),
            )
            .await?;
            
        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(anyhow::anyhow!("failed to retrieve audio file information")),
        };

        let file = match File::open(audio_file.get_os_path()) {
            Ok(file) => file,
            Err(_) => return Err(anyhow::anyhow!("failed to load file")),
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

        let sample_frame_packet_num = (content_length / sample_frame_packet_dur).floor() as u32;

        Ok(AudioMetaRes {
            content_length,
            sp_packets: sample_frame_packet_num,
            packet_dur: sample_frame_packet_dur,
            orig_sample_rate: sample_rate,
            orig_bit_rate: bit_rate,
            channels: channels.try_into().unwrap(),
        })
    }

    pub async fn get_audio_sample_iterator(
        &self,
        db: mongodb::Client,
        audio_tag_id: &str,
        packet_start_idx: usize,
        packet_num: usize,
        _channels: u32,
    ) -> Result<Packets, anyhow::Error> {        
        let settings = Settings::get()?;
        
        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();

        let audio_file = self.crud_audio_file
            .single
            .get(
                db.clone(),
                None,
                Some(
                    document::audio::query_audio_tag_referer(&audio_tag_id)
                )
            ).await?;

        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(anyhow::anyhow!("failed to retrieve audio file information")),
        };

        let file = match File::open(audio_file.get_os_path()) {
            Ok(file) => file,
            Err(_err) => return Err(anyhow::anyhow!("failed to load file")),
        };

        let packets = Packets::new(
            file,
            packet_start_idx,
            packet_num,
            settings.audio_sample_frame_packet.len.try_into().unwrap(),
            settings.audio_sample_frame_packet.sample_rate.try_into().unwrap(),
        )?;

        Ok(packets)
    }
}