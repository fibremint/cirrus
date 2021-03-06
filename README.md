# Cirrus

Cirrus is an audio player that reads audio file from remote server.

This repository provides:

* cirrus-app: desktop application
* cirrus-server: manage audio library and serve audio data

At now, supported audio format is restricted as `AIFF` and `16-bit, 2-channel`.

## Quickstart

### Server

* Install MongoDB
* Build Cirrus server with `cargo build --release` under `cirrus-server` directory
* Run Cirrus server with `cargo run --release`
* Add/Analyze/Refresh audio library with gRPC client (e.g. BloomRPC)

### Client

* Move to `cirrus-app` directory
* Install dependencies with `yarn`
* Build client with `yarn tauri build`
* Run client located at `src-tauri/target/release/cirrus`

## Architecture

### Overview
![architecture](assets/architecture-overview.png)

### Project Strucutre

* cirrus-app: Cirrus client frontend that provides UI and interact backend with Tauri plugin
* cirrus-lib/crates
  * aiff-rs: read idv3 tags and audio data from AIFF audio file
  * client: implementation of core audio player
  * grpc: contains protobuf definition and provide interoperability with Rust
  * server: implementation of server
  * tauri-plugin: Tauri plugin that initialize and utilize core audio player
* cirrus-server: Cirrus server wrapper

### Stack

* Client
  * Svelte: UI
  * Tauri: desktop application framework
  * cpal: low-level library for audio output
* Server
  * MongoDB: data source of audio metadata and audio library
  * aiff-rs: AIFF audio file reader
* Common
  * tonic: gRPC framework

## Q&A

### How is the audio player composed?

An audio player is composed of user interface and Cirrus audio core library (CACL). An UI interacts with CACL by register Cirrus Tauri plugin (CTP), which has an audio player interface and interactable via Tauri command to Tauri application for integrate on it. The CTP does initialize audio player instance from CACL and store this one within the Tauri State.

And an audio core library implements audio play that initializes audio output device, play audio from data (sample) and fetch audio data and fill to audio playback buffer. 

For example, if user clicks audio item from UI to play, UI (frontend) invokes audio load command (`plugin:cirrus|load_audio`) and plugin handles dispatched command by calling audio player instance's load audio method.

### How audio player plays audio?

An client creates audio playback object (`cpal::Stream`) with configuration (e.g. sample rate) and calls `play` method that run audio output thread. An audio play process is take audio samples from audio data buffer and give them to mutable array to output an audio, and is registered as audio play callback at stream creation.

And an audio buffer is filled by audio buffer thread that fetch data, process and fill to audio buffer queue. The audio data is part of PCM from audio file and is responsed from server as `u8` array. To read audio data, pre-process is required. As a case of the 16-bit audio, read as a step size 2 for each, convert to `i16` and divide by sample rate.

### How server reads and manages audio files?

In most cases, an audio directory has sub-directories that contain audio files. And Cirrus reads audio files from audio directories. Cirrus thinks that there is root (`library-root`) of sub-directories and sub-directory contains metadata of audio contents (`library`) such as timestamp of directory that used for check modification of directory at library refresh. An audio file metadata (`audio`) has field filename and path of audio sub-directory to point the actual path of audio file, and timestamp for check update of this one. And audio has tags such as title, artist, genre and so on. This information is stored at (`audio-tags`) collection.

A Cirrus server manages audio libraries with these behaviors:
* `add_audio_library`: insert `library-root` and `library` document to database
* `remove_audio_library`: remove documents of audio `library-root`, `library` and related audio data (`audio`) from database
* `analyze_audio_library`: create `audio-tags` document from `audio` and insert to database 
* `refesh_audio_library`: update `library`, `audio`, `audio-tags` documents.

## License
This project is licensed under the terms of the MIT license.
