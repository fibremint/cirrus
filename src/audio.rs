use aiff::reader::AiffReader;
use std::fs::File;

// use crate::server::audio_meta::AudioMetaRes;
use crate::server::audio_proto::{AudioMetaRes, AudioDataRes};

pub fn read_meta(filepath: &str) -> Result<AudioMetaRes, Box<dyn std::error::Error>> {
    let file = File::open(filepath)?;

    let mut reader = AiffReader::new(file);
    reader.read().unwrap();

    let reader_form_ref = reader.form().as_ref().unwrap();
    let common = reader_form_ref.common().as_ref().unwrap();
    let sound_meta = reader_form_ref.sound().as_ref().unwrap();

    let audio_meta_res = AudioMetaRes {
        bit_rate: common.bit_rate as u32,
        channels: common.num_channels as u32,
        sample_frames: common.num_sample_frames as u32,
        sample_rate: common.sample_rate as u32,
        size: sound_meta.size as u32,
        offset: sound_meta.offset,
        block_size: sound_meta.block_size,
    };

    Ok(audio_meta_res)
}

pub fn read_data(filepath: &str, byte_start: usize, byte_end: usize) -> Result<AudioDataRes, Box<dyn std::error::Error>> {
    let file = File::open(filepath)?;

    let mut reader = AiffReader::new(file);
    reader.read().unwrap();

    let reader_form_ref = reader.form().as_ref().unwrap();
    let data = reader_form_ref.sound().as_ref().unwrap();
    let mut audio_data_part = Vec::<u8>::new();
    audio_data_part.extend_from_slice(&data.sound_data[4*byte_start..4*byte_end]);

    let audio_data_res = AudioDataRes {
        content: audio_data_part,
    };

    Ok(audio_data_res)
}

pub fn check_file_exists(filename: &str) {
    println!("filename: {}", filename);
}