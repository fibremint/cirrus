pub mod audio;
pub mod client;

#[tokio::main]
async fn run() -> Result<(), anyhow::Error> {
    let mut audio_player = audio::AudioPlayer::new();
    audio_player.add_audio("test-source2.aiff").await?;
    println!("main: done add audio");
    audio_player.play();
    std::thread::sleep(std::time::Duration::from_secs(5000));

    Ok(())
}

fn main() {
    run().unwrap();
}
