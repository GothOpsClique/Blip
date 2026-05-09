use log::error;
use log::info;
use protocol::Message;
use protocol::{read_message, send_message};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio::task;

type ClientId = usize;
type Tx = mpsc::UnboundedSender<String>;
type Clients = Arc<Mutex<HashMap<ClientId, Tx>>>;

type ChannelId = usize;
type Channels = Arc<Mutex<HashMap<ChannelId, Channel>>>;
struct Channel {
    name: String,
}


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

    let channel_names: Vec<String> = vec!["general".to_string()];
    let channels: Channels = Arc::new(Mutex::new(HashMap::new()));
    let next_channel_id = Arc::new(AtomicUsize::new(1));

    for name in channel_names {
        let channel_id = next_channel_id.fetch_add(1, Ordering::Relaxed);
        channels.lock().await.insert(channel_id, Channel { name: name });
    }

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
    info!("Handling client {} from {}", client_id, peer_addr);

    let (mut reader, mut writer) = stream.into_split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    clients.lock().await.insert(client_id, tx);

    let write_task = task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(err) = send_message(&mut writer, Message { msg: &msg, channel: 1 }).await {
                error!("Failed to write to client {}: {}", client_id, err);
                break;
            }
        }
    });

    loop {
        match read_message(&mut reader).await {
            Ok(Some(message)) => {
                let formatted = format!("{}: {}", peer_addr, message);
                info!("Broadcasting from {}: {}", peer_addr, formatted);
                broadcast_message(&clients, Message { msg: &formatted, channel: 1 }).await;
            }
            Ok(None) => {
                info!("Client {} disconnected", client_id);
                break;
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

async fn broadcast_message(clients: &Clients, message: Message<'_>) {
    let mut clients = clients.lock().await;
    clients.retain(|client_id, tx| match tx.send(message.msg.clone()) {
        Ok(_) => true,
        Err(err) => {
            error!("Removing disconnected client {}: {}", client_id, err);
            false
        }
    });
}
