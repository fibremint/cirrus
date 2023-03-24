use std::sync::Arc;

use cpal::traits::{HostTrait, DeviceTrait};

pub struct AudioDeviceContext {
    pub device: cpal::Device,
    pub output_stream_config: Arc<cpal::StreamConfig>,
}

impl AudioDeviceContext {
    pub fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
    
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
        
        println!("Output device: {}", device.name()?);
    
        let output_stream_config: cpal::StreamConfig = device.default_output_config()?.into();

        println!("Output stream properties: sample_rate: {}, channel(s): {}", 
                 output_stream_config.sample_rate.0, output_stream_config.channels);

        Ok(Self {
            device,
            output_stream_config: Arc::new(output_stream_config),
        })
    }

    // pub fn output_stream_config(&self) -> OutputStreamConfig {
    //     OutputStreamConfig { 
    //         sample_rate: self.output_stream_config.sample_rate.0, 
    //         channels: self.output_stream_config.channels 
    //     }
    // }
}

// pub struct OutputStreamConfig {
//     pub sample_rate: u32,
//     pub channels: u16
// }
