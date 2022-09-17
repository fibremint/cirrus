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
