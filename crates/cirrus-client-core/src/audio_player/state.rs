#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize)]
pub enum PlaybackStatus {
    Play,
    Pause,
    Stop,
    Error,
}

impl From<usize> for PlaybackStatus {
    //ref: https://gist.github.com/polypus74/eabc7bb00873e6b90abe230f9e632989
    fn from(value: usize) -> Self {
        use self::PlaybackStatus::*;
        match value {
            0 => Play,
            1 => Pause,
            2 => Stop,
            3 => Error,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AudioSampleBufferStatus {
    StartFillBuffer,
    StartedFillBuffer,
    StopFillBuffer,
    StoppedFillBuffer,
    // StopFillBuffer,
}

impl From<usize> for AudioSampleBufferStatus {
    fn from(value: usize) -> Self {
        use self::AudioSampleBufferStatus::*;
        match value {
            0 => StartFillBuffer,
            1 => StartedFillBuffer,
            2 => StopFillBuffer,
            3 => StoppedFillBuffer,
            _ => unreachable!(),
        }
    }
}