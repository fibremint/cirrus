#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50000";

    cirrus_server_lib::server::run_server(addr).await?;
    
    Ok(())
}