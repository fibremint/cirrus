use std::fs::File;

use anyhow::anyhow;
use itertools::Itertools;
use symphonia::core::{formats::{FormatReader, SeekMode, SeekTo}, io::MediaSourceStream, probe::Hint, codecs::{CODEC_TYPE_NULL, Decoder}, audio::SampleBuffer};


pub struct SampleFrames {
    media_reader: Box<dyn FormatReader>,
    audio_decoder: Box<dyn Decoder>,

    pub codec_sample_rate: u32,
    seek_start_frame_idx: usize,
    seek_end_frame_idx: usize,

    frame_cnt: usize,
    frame_len: usize,
    frame_buf: Vec<f32>,

    curr_frame_start_ts: u64,
    curr_frame_dur: u64,
    resolved_first_offset: bool,
}

impl SampleFrames {
    pub fn new(
        source: File,
        seek_start_frame_idx: usize,
        seek_end_frame_idx: usize,
        seek_start_sample_ts: u64,
    ) -> Result<Self, anyhow::Error> {
        let mss = MediaSourceStream::new(Box::new(source), Default::default());
        let hint = Hint::new();

        let probe_res = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &Default::default()).unwrap();

        let mut format = probe_res.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        let codec_sample_rate = track.codec_params.sample_rate.unwrap();

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("unsupported codec");

        format.seek(
            SeekMode::Coarse, 
            SeekTo::TimeStamp {
                ts: seek_start_sample_ts,
                track_id: track.id,
            }
        )?;

        Ok(Self {
            media_reader: format,
            audio_decoder: decoder,

            codec_sample_rate,
            seek_start_frame_idx,
            seek_end_frame_idx,

            frame_len: 0,
            frame_cnt: 0,
            frame_buf: Default::default(),

            curr_frame_start_ts: seek_start_sample_ts,
            curr_frame_dur: 0,
            resolved_first_offset: false,
        })
    }

    pub fn get_curr_frame_idx(&self) -> usize {
        self.seek_start_frame_idx + self.frame_cnt
    }

    pub fn set_frame_len(&mut self, frame_len: usize) {
        self.frame_len = frame_len;
    }

    fn read_samples(&mut self) -> Result<(), anyhow::Error> {
        while self.frame_buf.len() / 2 < self.frame_len {
            let mut read_start_offset = 0;

            let packet = match self.media_reader.next_packet() {
                Ok(packet) => packet,
                Err(err) => {
                    return Err(anyhow!(err));
                },
            };

            let curr_frame_start_ts = packet.ts();
            let curr_frame_dur = packet.dur();
            // assert_eq!(self.curr_frame_start_ts, curr_frame_start_ts);

            self.curr_frame_start_ts = curr_frame_start_ts;
            self.curr_frame_dur = curr_frame_dur;

            let min_seek_start_frame_len = self.frame_len * self.seek_start_frame_idx;

            if self.curr_frame_start_ts + self.curr_frame_dur <= min_seek_start_frame_len.try_into().unwrap() {
                continue;
            }

            if !self.resolved_first_offset {
                read_start_offset = self.frame_len * self.seek_start_frame_idx - self.curr_frame_start_ts as usize;
            }

            let audio_buf = match self.audio_decoder.decode(&packet) {
                Ok(buf_ref) => buf_ref,
                Err(err) => {
                    return Err(anyhow::anyhow!(err));
                },
            };

            let mut sample_buf = SampleBuffer::<f32>::new(
                audio_buf.capacity() as u64, 
                *audio_buf.spec()
            );

            sample_buf.copy_interleaved_ref(audio_buf);
            let (_, frames) = sample_buf.samples().split_at(read_start_offset*2);

            self.frame_buf.extend_from_slice(frames);
            
            self.resolved_first_offset = true;
        }

        Ok(())
    }
}

impl Iterator for SampleFrames {
    type Item = Result<SampleFrame, anyhow::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame_len == 0 {
            return Some(Err(anyhow::anyhow!("frame length is not set")));
        }

        if self.get_curr_frame_idx() == self.seek_end_frame_idx {
            return None;
        }

        if let Err(err) = self.read_samples() {
            return Some(Err(err));
        }
        
        let samples = self.frame_buf
            .drain(..self.frame_len*2)
            .collect_vec();

        let next_frame_seek_start_ts =
            if self.frame_buf.len() > 0
                { self.curr_frame_start_ts }
            else 
                { self.curr_frame_start_ts + self.curr_frame_dur };

        let frame = SampleFrame {
            idx: self.seek_start_frame_idx + self.frame_cnt,
            next_seek_ts: next_frame_seek_start_ts,
            samples,
        };

        self.frame_cnt += 1;

        Some(Ok(frame))
    }
}

pub struct SampleFrame {
    pub idx: usize,
    pub next_seek_ts: u64,
    pub samples: Vec<f32>,
}
