FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:edge

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y protobuf-compiler libprotobuf-dev