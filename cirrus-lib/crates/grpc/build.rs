use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_output_path = "proto-rust";

    if Path::new(proto_output_path).exists() {
        fs::remove_dir_all(proto_output_path)?;
    }

    fs::create_dir(proto_output_path)?;

    tonic_build::configure()
        .out_dir(proto_output_path)
        .compile(
            &["proto/cirrus.proto"], 
            &["proto"])?;

    Ok(())
}