use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use protocol::{Attachment, ChatMessage, decode_message, encode_message};
use std::env;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use url::Url;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:6666".to_string());
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "anonymous".to_string());

    let url = Url::parse(&format!("ws://{}", addr)).expect("Invalid WebSocket URL");
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket server");

    info!("Connected to server at {}", addr);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(frame) = ws_receiver.next().await {
            match frame {
                Ok(WsMessage::Binary(bytes)) => match decode_message(&bytes) {
                    Ok(message) => {
                        println!("{}: {}", message.sender, message.content);
                        for attachment in message.attachments {
                            println!("  attachment: {}", attachment.url);
                        }
                    }
                    Err(err) => {
                        error!("Failed to decode server message: {}", err);
                    }
                },
                Ok(WsMessage::Text(text)) => {
                    println!("Server text: {}", text);
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Server closed the connection");
                    break;
                }
                _ => {}
            }
        }
    });

    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type messages and press Enter to send. Use /attach <url> to send attachments.");

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut message = ChatMessage::default();
        message.sender = username.clone();
        message.channel = 1;
        message.timestamp = current_timestamp_millis();

        if let Some(url) = trimmed.strip_prefix("/attach ") {
            let attachment = Attachment {
                url: url.trim().to_string(),
                size: 0,
                mime_type: String::new(),
            };
            message.attachments.push(attachment);
            message.content = String::from("sent an attachment");
        } else {
            message.content = trimmed.to_string();
        }

        let bytes = encode_message(&message);
        if ws_sender.send(WsMessage::Binary(bytes)).await.is_err() {
            eprintln!("Error sending message");
            break;
        }
    }

    Ok(())
}

fn current_timestamp_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
