use log::{error, info};
use protocol::Message;
use std::env;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:6666".to_string());

    let stream = TcpStream::connect(&addr).await?;
    info!("Connected to server at {}", addr);

    let (mut reader, mut writer) = stream.into_split();

    tokio::spawn(async move {
        loop {
            match protocol::read_message(&mut reader).await {
                Ok(Some(msg)) => println!("Server: {}", msg),
                Ok(None) => {
                    info!("Server disconnected");
                    break;
                }
                Err(e) => {
                    error!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });

    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type messages and press Enter to send:");

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(e) = protocol::send_message(&mut writer, Message { msg: &trimmed.to_string(), channel: 1 }).await {
            eprintln!("Error sending message: {}", e);
            break;
        }
    }

    Ok(())
}
