use std::fs::File;

use audio::Buf;
use rubato::Resampler;

use super::sample::SampleFrames;

const MIN_ENCODER_PRESYNC_PKT_MS: i32 = 80;

pub struct Packets {
    sample_frames: SampleFrames,

    resampler: rubato::FftFixedOut<f32>,
    resampler_input_buf: Vec<Vec<f32>>,
    resampler_output_buf: Vec<Vec<f32>>,

    packet_encoder: opus::Encoder,
    packet_dur_ms: u32,

    packet_start_idx: usize,
    packet_len: usize,
    packet_dur: f64,
}

impl Packets {
    pub fn new(
        source: File,
        pkt_start_idx: usize,
        pkt_num: usize,
        pkt_len: usize,
        sample_rate: usize,
    ) -> Result<Self, anyhow::Error> {
        let packet_dur = pkt_len as f64 / sample_rate as f64;
        let packet_dur_ms = (packet_dur * 1000 as f64) as u32;

        let seek_start_frame_idx = 
            if pkt_start_idx > 4
                { pkt_start_idx - 4 }
            else
                { pkt_start_idx };

        let mut sample_frames = SampleFrames::new(
            source,
            seek_start_frame_idx,
            pkt_start_idx + pkt_num-1,
        )?;

        let resampler = rubato::FftFixedOut::new(
            sample_frames.codec_sample_rate.try_into().unwrap(), 
            sample_rate,
            pkt_len, 
            2,
            2
        )?;

        sample_frames.set_frame_len(resampler.input_frames_max());

        if pkt_start_idx > 4 {
            let seek_start_idx = pkt_start_idx - 4;
            let seek_start_ts = seek_start_idx * resampler.input_frames_max();

            sample_frames.seek(seek_start_ts.try_into().unwrap())?;
        }

        let mut resampler_input_buf = resampler.input_buffer_allocate();

        for input_buf_ch in resampler_input_buf.iter_mut() {
            input_buf_ch.extend(vec![0.; resampler.input_frames_max()]);
        }
        let resampler_output_buf = resampler.output_buffer_allocate();

        let packet_encoder = opus::Encoder::new(48_000, opus::Channels::Stereo, opus::Application::Audio)?;

        let mut packets = Self{
            sample_frames,

            resampler,
            resampler_input_buf,
            resampler_output_buf,

            packet_encoder,
            packet_dur_ms,

            packet_start_idx: pkt_start_idx,
            packet_len: pkt_len,
            packet_dur,
        };

        packets.resovle_encoder_frame_sync();

        Ok(packets)
    }

    fn resovle_encoder_frame_sync(&mut self) {
        if self.packet_start_idx == 0 {
            return;
        }

        loop {
            let frame_idx_delta = self.packet_start_idx as i64 - self.sample_frames.get_curr_frame_idx();
            if frame_idx_delta as u32 * self.packet_dur_ms <= MIN_ENCODER_PRESYNC_PKT_MS.try_into().unwrap() {
                break;
            }

            self.sample_frames.next().unwrap().unwrap();
        }

        while self.packet_start_idx -1 > self.sample_frames.get_curr_frame_idx() as usize {
            let frame = self.sample_frames.next().unwrap().unwrap();
            self.create_packet(frame.samples);
        }
    }

    fn create_packet(&mut self, samples: Vec<f32>) -> Vec<u8> {
        let samples = audio::wrap::interleaved(samples.as_slice(), 2);
        let samples_reader = audio::io::Read::new(samples);

        for ch_idx in 0..samples_reader.channels() {
            let sample_ch_buf = samples_reader
                .get(ch_idx)
                .unwrap()
                .iter()
                .collect::<Vec<_>>();

            self.resampler_input_buf[ch_idx] = sample_ch_buf;
        }

        self.resampler.process_into_buffer(
            &self.resampler_input_buf, 
            &mut self.resampler_output_buf,
            None
        ).unwrap();

        let mut resampled_output = audio::buf::Interleaved::<f32>::with_topology(
            2, 
            self.packet_len
        );

        for ch_idx in 0..2 {
            for (c, s) in resampled_output
                .get_mut(ch_idx)
                .unwrap()
                .iter_mut()
                .zip(&self.resampler_output_buf[ch_idx])
            {
                *c = *s;
            }
        }

        let packet = self.packet_encoder
            .encode_vec_float(resampled_output.as_slice(), 4000)
            .unwrap();

        packet
    }
}

impl Iterator for Packets {
    type Item = Packet;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.sample_frames.next();

        if frame.is_none() {
            return None;
        }

        let frame = frame.unwrap().unwrap();
        let encoded_frame = self.create_packet(frame.samples);

        Some(Packet {
            idx: frame.idx,

            frame_ts: frame.idx as f64 * self.packet_dur,
            frame: encoded_frame,
            frame_len: self.packet_len,
            frame_dur: self.packet_dur,

            next_pkt_seek_ts: frame.next_seek_ts, 
        })
    }
}

pub struct Packet {
    pub idx: usize,

    pub frame_ts: f64,
    pub frame: Vec<u8>,
    pub frame_len: usize,
    pub frame_dur: f64,
    
    pub next_pkt_seek_ts: u64,
}