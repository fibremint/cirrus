use std::iter::Copied;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

pub fn sample_next(o: &mut SampleRequestOption) -> f32 {
    o.tick();
    o.tone(440.0) * 0.1 + o.tone(880.) * 0.1
}

pub struct SampleRequestOption {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub channels: usize,
}

impl SampleRequestOption {
    fn tone(&self, freq: f32) -> f32 {
        (self.sample_clock * freq * 2.0 * std::f32::consts::PI / self.sample_rate).sin()
    }

    fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
    }
}

pub fn stream_setup_for<F>(on_sample: F) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOption) -> f32 + std::marker::Send + Copy + 'static,
{
    let (_, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample),

    }
}

fn host_device_setup() -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config: {:?}", config);

    Ok((host, device, config))
}

fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOption) -> f32 + std::marker::Send + Copy + 'static,
{
    let sample_rate = config.sample_rate.0 as f32;
    let sample_clock = 0f32;
    let channels = config.channels as usize;
    let mut request = SampleRequestOption {
        sample_rate,
        sample_clock,
        channels,
    };

    let err_fn = |err| println!("Error building output soud stream: {}", err);

    let stream = device.build_output_stream(
        config, 
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            on_window(output, &mut request, on_sample)
        }, 
        err_fn
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOption, mut on_sample: F)
where 
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOption) -> f32 + std::marker::Send + Copy,
{
    for frame in output.chunks_mut(request.channels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}



// pub struct AudioPlayer<'a> {
//     audio_stream: &'a cpal::Stream
// }

// impl<'a> AudioPlayer<'a> {
//     fn new() -> Self {
//         Self 
//     }
// }

// struct PlayerStatus {

// }