use cpal::traits::{HostTrait, DeviceTrait};

pub struct AudioContext {
    pub device: cpal::Device,
    pub stream_config: cpal::StreamConfig,
}

impl AudioContext {
    pub fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
    
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
        
        println!("Output device: {}", device.name()?);
    
        let config: cpal::StreamConfig = device.default_output_config()?.into();

        println!("Output stream properties: sample_rate: {}, channel(s): {}", 
                 config.sample_rate.0, config.channels);

        Ok(Self {
            device,
            stream_config: config,
        })
    }

    pub fn host_stream_config(&self) -> HostStreamConfig {
        HostStreamConfig { 
            sample_rate: self.stream_config.sample_rate.0, 
            output_channels: self.stream_config.channels 
        }
    }
}

pub struct HostStreamConfig {
    pub sample_rate: u32,
    pub output_channels: u16
}

