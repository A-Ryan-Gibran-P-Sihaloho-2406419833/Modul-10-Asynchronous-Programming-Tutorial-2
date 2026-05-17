use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

type Tx = mpsc::UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, (Tx, String)>>>;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Gagal melakukan bind TCP");
    println!("🦀 Server Bonus (Rust) berjalan di: ws://{}", addr);

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }
}

async fn handle_connection(state: PeerMap, stream: TcpStream, addr: SocketAddr) {
    let ws_stream = accept_async(stream).await.expect("Error selama handshake WebSocket");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Meneruskan pesan dari channel mpsc ke websocket klien sesungguhnya
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut username = String::new();

    // Loop mendengarkan pesan dari klien Yew
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                match ws_msg.message_type {
                    MsgTypes::Register => {
                        if let Some(name) = ws_msg.data {
                            username = name.clone();
                            state.lock().await.insert(addr, (tx.clone(), name));
                            broadcast_users(&state).await;
                        }
                    }
                    MsgTypes::Message => {
                        if let Some(content) = ws_msg.data {
                            // Bungkus pesan ke dalam struct MessageData sebelum dikirim
                            let msg_data = MessageData {
                                from: username.clone(),
                                message: content,
                            };
                            let out_msg = WebSocketMessage {
                                message_type: MsgTypes::Message,
                                data_array: None,
                                data: Some(serde_json::to_string(&msg_data).unwrap()),
                            };
                            broadcast_msg(&state, &out_msg).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Jika klien terputus (disconnect)
    state.lock().await.remove(&addr);
    broadcast_users(&state).await;
}

// Fungsi bantu untuk menyiarkan daftar user online
async fn broadcast_users(state: &PeerMap) {
    let users: Vec<String> = state.lock().await.values().map(|(_, name)| name.clone()).collect();
    let out_msg = WebSocketMessage {
        message_type: MsgTypes::Users,
        data_array: Some(users),
        data: None,
    };
    broadcast_msg(state, &out_msg).await;
}

// Fungsi bantu untuk menyiarkan JSON ke seluruh klien
async fn broadcast_msg(state: &PeerMap, msg: &WebSocketMessage) {
    let serialized = serde_json::to_string(msg).unwrap();
    let peers = state.lock().await;
    for (tx, _) in peers.values() {
        let _ = tx.send(Message::Text(serialized.clone()));
    }
}