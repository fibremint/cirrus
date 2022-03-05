pub mod audio;
pub mod client;
pub mod request;

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    let mut audio_player = audio::AudioPlayer::new();
    audio_player.add_audio("test-source.aiff").await?;
    println!("main: done add audio");
    audio_player.play().await;
    std::thread::sleep(std::time::Duration::from_millis(100_000));

    // let res = client::get_audio_data("test-source.aiff", 0, 1323000).await.unwrap();
    // let content = res.into_inner().content;

    // println!("{}", content[0]);
    Ok(())
}

fn main() {
    // // let stream = audio::stream_setup_for(audio::sample_next)?;
    // // stream.play()?;
    // let mut audio_player = audio::AudioPlayer::new();
    // audio_player.add_audio("test-source.aiff").await?;
    // // audio_player.add_audio(String::from("bar"))?;
    // // audio_player.add_stream(String::from("foo"));
    // // audio_player.add_stream(String::from("bar"));
    // audio_player.play();

    // std::thread::sleep(std::time::Duration::from_millis(100_000));

    // audio_player.pause();
    // // let source_lists = AudioRequest::list_audio_sources();

    // Ok(())
    run();
    std::thread::sleep(std::time::Duration::from_millis(100_000));
    // let test = vec![1, 2, 3, 4, 5, 6];
    // let res = test.chunks(2)
    //     .map(|chunk| chunk.iter().sum::<i32>())
    //     .collect::<Vec<i32>>();

    // println!("{:?}", res);
}
