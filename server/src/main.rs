use log::error;
use log::info;
use std::env;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;

#[tokio::main]
async fn main() {
    env_logger::init();
    let result = handle_connections().await;
    if let Err(e) = result {
        error!("Server error: {}", e);
    }
}

async fn handle_connections() -> std::io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:6666".to_string());

    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    while let Ok((stream, peer)) = listener.accept().await {
        info!("Client connected: {}", peer);
        task::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                error!("Connection error: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let peer_addr = stream
        .peer_addr()
        .map_or_else(|_| "unknown".parse().unwrap(), |addr| addr.to_string());
    info!("Handling client connection from {}", peer_addr);

    let mut buffer = [0; 1024];
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            info!("Client {} disconnected", peer_addr);
            return Ok(());
        }

        let received = String::from_utf8_lossy(&buffer[..n]);
        info!("Received from {}: {}", peer_addr, received);

        if let Err(e) = stream.write_all(&buffer[..n]).await {
            error!("Failed to send data to {}: {}", peer_addr, e);
            return Err(e);
        }
    }
}
