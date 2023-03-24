use audio::{Buf, buf::Interleaved, BufMut};
use rubato::Resampler;

pub struct AudioResampler {
    resampler: rubato::FftFixedOut<f32>,
    resampler_output_buf: Vec<Vec<f32>>,

    input_buf: audio::wrap::Dynamic<Vec<Vec<f32>>>,
    output_buf: audio::buf::Interleaved<f32>,

    channels: usize,
}

impl AudioResampler {
    pub fn new(
        output_sample_rate: usize,
        channels: usize
    ) -> Result<Self, anyhow::Error> {
        let chunk_size_out = output_sample_rate / 50;

        let resampler = rubato::FftFixedOut::<f32>::new(
            48_000,
            output_sample_rate,
            chunk_size_out,
            2,
            channels
        )?;

        let resampler_output_buf = resampler.output_buffer_allocate();

        let input_buf = audio::wrap::dynamic(vec![vec![0.; 960]; 2]);
        let output_buf = audio::buf::Interleaved::with_topology(
            channels,
            resampler.output_frames_max()
        );

        Ok(Self {
            resampler,
            resampler_output_buf,
            input_buf,
            output_buf,
            channels,
        })
    }

    pub fn resample(
        &mut self,
        decoded_samples: &Interleaved<f32>
    ) -> Result<&Interleaved<f32>, anyhow::Error> {
        let decoded_samples = audio::io::Read::new(decoded_samples);

        for ch_idx in 0..self.channels {
            audio::channel::copy(
                decoded_samples.get(ch_idx).unwrap(),
                self.input_buf.get_mut(ch_idx).unwrap()
            );
        }

        self.resampler.process_into_buffer(
            self.input_buf.as_ref(),
            &mut self.resampler_output_buf,
            None
        ).unwrap();

        for ch_idx in 0..self.channels {
            audio::channel::copy(
                audio::channel::LinearChannel::new(self.resampler_output_buf.get(ch_idx).unwrap()), 
                self.output_buf.get_mut(ch_idx).unwrap(),
            )
        }

        Ok(&self.output_buf)
    }

    pub fn get_processed_sample_len(&self) -> usize {
        self.resampler.output_frames_max() * self.channels
    }
}