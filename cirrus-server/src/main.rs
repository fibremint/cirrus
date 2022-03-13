pub mod audio;
pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50000";

    server::run_server(addr).await?;
    
    Ok(())
}