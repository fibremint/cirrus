use aiff::reader::AiffReader;
use std::fs::File;

pub mod audio_meta {
    tonic::include_proto!("audio");
}

// use audio_meta::AudioMetaRes;

use crate::server::audio_meta::AudioMetaRes;

pub fn read_meta(filepath: &str) -> Result<AudioMetaRes, Box<dyn std::error::Error>> {
    let file = File::open(filepath)?;

    let mut reader = AiffReader::new(file);
    reader.read().unwrap();

    let reader_form_ref = reader.form().as_ref().unwrap();
    let common = reader_form_ref.common().as_ref().unwrap();

    let audio_meta_res = AudioMetaRes {
        bit_rate: common.bit_rate as u32,
        channels: common.num_channels as u32,
        sample_frames: common.num_sample_frames as u32,
        sample_rate: common.sample_rate as u32,
    };

    Ok(audio_meta_res)
}

fn read_data() {

}

pub fn check_file_exists(filename: &str) {
    println!("filename: {}", filename);
}