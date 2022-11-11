use std::fs::File;

use itertools::Itertools;
use symphonia::core::{formats::{FormatReader, SeekMode, SeekTo}, io::MediaSourceStream, probe::Hint, codecs::{CODEC_TYPE_NULL, Decoder}, audio::SampleBuffer, units::Time, errors::Error};

pub struct SampleFrames {
    media_reader: Box<dyn FormatReader>,
    audio_decoder: Box<dyn Decoder>,
    track_id: u32,

    pub codec_sample_rate: u32,
    seek_start_frame_idx: usize,
    seek_end_frame_idx: usize,

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
    ) -> Result<Self, anyhow::Error> {
        let mss = MediaSourceStream::new(Box::new(source), Default::default());
        let hint = Hint::new();

        let probe_res = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &Default::default()).unwrap();

        let format = probe_res.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        let track_id = track.id;

        let codec_sample_rate = track.codec_params.sample_rate.unwrap();

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("unsupported codec");

        Ok(Self {
            media_reader: format,
            audio_decoder: decoder,
            track_id,

            codec_sample_rate,
            seek_start_frame_idx,
            seek_end_frame_idx,

            frame_len: 0,
            frame_buf: Default::default(),

            curr_frame_start_ts: 0,
            curr_frame_dur: 0,
            resolved_first_offset: false,
        })
    }

    pub fn seek(&mut self, ts: u64) -> Result<(), anyhow::Error> {
        if ts < self.frame_len as u64 {
            return Err(anyhow::anyhow!("timestamp is not enough to seek"));
        }

        self.media_reader.seek(
            SeekMode::Coarse, 
            SeekTo::TimeStamp {
                ts: ts - self.frame_len as u64,
                track_id: self.track_id,
            }
        )?;

        self.read_samples()?;

        Ok(())
    }
    
    pub fn get_curr_frame_idx(&self) -> i64 {
        // should call this function after read samples
        let remain_frame_len = self.frame_buf.len() / 2;
        let curr_frame_size = (self.curr_frame_start_ts + self.curr_frame_dur) as usize - remain_frame_len;

        (curr_frame_size / self.frame_len) as i64 -1
    }

    pub fn set_frame_len(&mut self, frame_len: usize) {
        self.frame_len = frame_len;
    }

    fn read_samples(&mut self) -> Result<(), Error> {
        while self.frame_buf.len() / 2 < self.frame_len {
            let mut read_start_offset = 0;

            let packet = match self.media_reader.next_packet() {
                Ok(packet) => packet,
                Err(err) => {
                    return Err(err);
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
                    return Err(err);
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

        if self.get_curr_frame_idx() as usize == self.seek_end_frame_idx {
            return None;
        }

        match self.read_samples() {
            Ok(_) => (),
            Err(Error::IoError(_err)) => {
                return None;
            }
            Err(err) => {
                return Some(Err(anyhow::anyhow!(err)));
            },
        }
        
        let samples = self.frame_buf
            .drain(..self.frame_len*2)
            .collect_vec();

        let next_frame_seek_start_ts =
            if self.frame_buf.len() > 0
                { self.curr_frame_start_ts }
            else 
                { self.curr_frame_start_ts + self.curr_frame_dur };

        let frame_idx = self.get_curr_frame_idx().try_into().unwrap();

        let frame = SampleFrame {
            idx: frame_idx,
            next_seek_ts: next_frame_seek_start_ts,
            samples,
        };

        Some(Ok(frame))
    }
}

pub struct SampleFrame {
    pub idx: usize,
    pub next_seek_ts: u64,
    pub samples: Vec<f32>,
}
