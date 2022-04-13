# Cirrus

Cirrus is an audio service that plays audio file remotly from server.

This repository provides:

* cirrus-app: desktop application
* cirrus-server: manage audio library and serve audio data

At now, supported audio format is restricted as `AIFF`.

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

TODO: add service logic diagram

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

### How audio player works?

* data

* command

### What core audio player does?

### How audio library 

### Why MongoDB is used for DB?

### Why gRPC is used for API than REST?
