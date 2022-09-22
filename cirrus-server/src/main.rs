#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50000";

    cirrus_server_lib::run_server(addr).await?;

    Ok(())
}