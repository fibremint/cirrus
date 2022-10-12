use std::{
    env,
    path::PathBuf
};
use dunce;

fn main() -> Result<(), anyhow::Error> {
    let project_path: PathBuf = [
        env::var("CARGO_MANIFEST_DIR").unwrap().as_str(), 
            "..", 
            ".."
        ]
        .iter()
        .collect::<PathBuf>();
    let project_path = dunce::canonicalize(&project_path).unwrap();
    let proto_path = project_path.join("proto");

    let mut tonic_builder = tonic_build::configure()
        .type_attribute(".cirrus.api.AudioTagRes", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_server(false)
        .build_client(false);

    if env::var("CARGO_FEATURE_SERVER").is_ok() {
        tonic_builder = tonic_builder.build_server(true);
    }
    
    if env::var("CARGO_FEATURE_CLIENT").is_ok() {
        tonic_builder = tonic_builder.build_client(true);
    }

    tonic_builder.compile(
        &["cirrus.proto"],
        &[proto_path]
    )?;

    Ok(())
}