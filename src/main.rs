use cpal::traits::StreamTrait;

pub mod audio;
pub mod request;

use crate::request::AudioRequest;

fn main() -> anyhow::Result<()> {
    // let stream = audio::stream_setup_for(audio::sample_next)?;
    // stream.play()?;
    let mut audio_player = audio::AudioPlayer::new();
    audio_player.add_audio(String::from("foo"))?;
    audio_player.add_audio(String::from("bar"))?;
    // audio_player.add_stream(String::from("foo"));
    // audio_player.add_stream(String::from("bar"));
    audio_player.play();

    std::thread::sleep(std::time::Duration::from_millis(10_000));

    audio_player.pause();
    // let source_lists = AudioRequest::list_audio_sources();

    Ok(())
}
