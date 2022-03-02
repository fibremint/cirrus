// use std::{time::SystemTime, sync::Arc};

// use tokio::{self, io::AsyncWriteExt};
// use futures::future;

// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use cpal::{Data, Sample, SampleFormat};

// use std::collections::VecDeque;

// pub mod audio;

pub mod audio;

fn main() -> anyhow::Result<()> {
    let stream = audio::stream_setup_for(audio::sample_next)?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(30000));
    Ok(())
}

// #[tokio::main]
// async fn run() -> Result<(), Box<dyn std::error::Error>> {
//     // Initialize the hose
//     let audio_host = cpal::default_host();

//     // Choose to availble device
//     let device = audio_host.default_output_device().expect("none of the output devices are available");

//     // Configuration of the audio stream
//     // let mut supported_configs_range = device.supported_output_configs()
//     //     .expect("error while querying configs");
//     // let supported_config = supported_configs_range.next()
//     //     .expect("no supported config")
//     //     .with_max_sample_rate();
//     // let sample_format = supported_config.sample_format();
//     // let config = supported_config.into();
//     let config = device.default_output_config().unwrap();

//     // let err_fn = |err| eprintln!("an error occured on the output audio");
//     // let stream = match sample_format {
//     //     SampleFormat::F32 => device.build_output_stream(&config, write_silence::<f32>, err_fn),
//     //     SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn),
//     //     SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn),
//     // }.unwrap();

//     match config.sample_format() {
//         cpal::SampleFormat::F32 => run_audio::<f32>(&device, &config.into()),
//         cpal::SampleFormat::I16 => run_audio::<i16>(&device, &config.into()),
//         cpal::SampleFormat::U16 => run_audio::<u16>(&device, &config.into()),
//     }.unwrap();

//     // fn write_silence<T: Sample>(data: &mut [T], cb_info: &cpal::OutputCallbackInfo) {
//     //     println!("cb_info {:?}", cb_info);
//     //     for sample in data.iter_mut() {
//     //         *sample = Sample::from(&0.0);
//     //     }
//     // }

//     // stream.play().unwrap();

//     Ok(())
// }

// pub fn run_audio<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
// where 
//     T: cpal::Sample 
// {
//     let smaple_rate = config.sample_rate.0 as f32;
//     let channels = config.channels as usize;

//     let mut sample_clock = 0f32;
//     let mut next_value = move || {
//         sample_clock = (sample_clock + 1.0) % smaple_rate;
//         // (22050.0 * 440.0 * 1.33 * std::f32::consts::PI / smaple_rate).sin()
//         (sample_clock * 440.0 * 1.16 * std::f32::consts::PI / smaple_rate).sin()
//     };

//     // let mut audio_buffer: VecDeque<f32> = VecDeque::new();
//     // for _ in 0..441000 {
//     //     audio_buffer.push_back(next_value());
//     // }

//     let err_fn = |err| eprint!("an error occured on stream: {}", err);

//     let mut frame_idx = 0;
//     let stream = device.build_output_stream(
//         config, 
//         move|data: &mut [T], output_cb_info| {
//             println!("output cb info {:?}", output_cb_info);
//             // println!("frame idx: {}", frame_idx);
//             // println!("data len {:?}", data.len());
//             // frame_idx += 1;
//             // write_data(data, channels, &mut audio_buffer)
//             write_data(data, channels, &mut next_value)
//         },
//         err_fn, 
//     )?;

//     stream.play()?;

//     std::thread::sleep(std::time::Duration::from_millis(50000));

//     Ok(())
// }

// fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32) 
// where 
//     T: cpal::Sample
// {
//     for frame in output.chunks_mut(channels) {
//         let value: T = cpal::Sample::from::<f32>(&next_sample());
//         for sample in frame.iter_mut() {
//             *sample = value;
//         }
//     }
// }

// // fn write_data<T>(output: &mut [T], channels: usize, audio_buffer: &mut VecDeque<f32>) 
// // where 
// //     T: cpal::Sample
// // {
// //     for frame in output.chunks_mut(channels) {
// //         // let value: T = cpal::Sample::from::<f32>(&next_sample());
// //         // let data = audio_buffer.pop_front();
// //         // let sample_value = cpal::Sample::from::<f32>(data.as_ref().unwrap());
// //         for sample in frame.iter_mut() {
// //             let data = audio_buffer.pop_front();
// //             let sample_value = cpal::Sample::from::<f32>(data.as_ref().unwrap());
// //             *sample = sample_value;
// //             // *sample = value;
// //         }
// //         // println!("a");
// //     }
// //     // println!("b");
// // }

// fn main() {
//     run();
//     // println!("Hello, world!");
// }

use cpal::traits::StreamTrait;

