

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../cirrus-lib/proto/audio.proto")?;
    Ok(())
}