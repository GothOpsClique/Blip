use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use protocol::{decode_message, encode_message};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::task;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;

type ClientId = usize;
type Tx = mpsc::UnboundedSender<Vec<u8>>;
type ClientInfo = (Tx, i32);
type Clients = Arc<Mutex<HashMap<ClientId, ClientInfo>>>;

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

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let next_client_id = Arc::new(AtomicUsize::new(1));

    while let Ok((stream, peer)) = listener.accept().await {
        let client_id = next_client_id.fetch_add(1, Ordering::Relaxed);
        let clients = clients.clone();
        info!("Client connected: {} (id={})", peer, client_id);

        task::spawn(async move {
            if let Err(e) = handle_client(stream, client_id, clients).await {
                error!("Connection error for client {}: {}", client_id, e);
            }
        });
    }
    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    client_id: ClientId,
    clients: Clients,
) -> std::io::Result<()> {
    let peer_addr = stream
        .peer_addr()
        .map_or_else(|_| "unknown".parse().unwrap(), |addr| addr.to_string());

    let ws_stream = accept_async(stream)
        .await
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
    info!(
        "WebSocket handshake completed for client {} from {}",
        client_id, peer_addr
    );

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Start each client on channel 1 by default.
    clients.lock().await.insert(client_id, (tx, 1));

    let write_task = task::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            if ws_sender.send(WsMessage::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(WsMessage::Binary(bytes)) => match decode_message(&bytes) {
                Ok(mut chat) => {
                    if chat.sender.is_empty() {
                        chat.sender = peer_addr.clone();
                    }
                    if chat.id.is_empty() {
                        chat.id = format!("{}-{}", client_id, current_timestamp_millis());
                    }
                    if chat.timestamp == 0 {
                        chat.timestamp = current_timestamp_millis();
                    }

                    // Track the client's current channel from their last message.
                    {
                        let mut clients_guard = clients.lock().await;
                        if let Some(client_info) = clients_guard.get_mut(&client_id) {
                            client_info.1 = chat.channel;
                        }
                    }

                    if chat.content.is_empty() && chat.attachments.is_empty() {
                        info!("Client {} switched to channel {}", client_id, chat.channel);
                        continue;
                    }

                    info!(
                        "Broadcasting from {} on channel {}: {}",
                        chat.sender, chat.channel, chat.content
                    );
                    broadcast_message(&clients, chat.channel, encode_message(&chat)).await;
                }
                Err(err) => {
                    error!(
                        "Invalid protobuf message from client {}: {}",
                        client_id, err
                    );
                }
            },
            Ok(WsMessage::Close(_)) => {
                info!("Client {} disconnected", client_id);
                break;
            }
            Ok(_) => {
                // Ignore ping/pong/text frames for this prototype
            }
            Err(err) => {
                error!("Error reading from client {}: {}", client_id, err);
                break;
            }
        }
    }

    clients.lock().await.remove(&client_id);
    write_task.await.expect("write task join failed");
    Ok(())
}

async fn broadcast_message(clients: &Clients, channel: i32, message: Vec<u8>) {
    let mut clients = clients.lock().await;
    clients.retain(|client_id, (tx, client_channel)| {
        if *client_channel != channel {
            return true;
        }

        match tx.send(message.clone()) {
            Ok(_) => true,
            Err(err) => {
                error!("Removing disconnected client {}: {}", client_id, err);
                false
            }
        }
    });
}

fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
