use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    // Kirim pesan selamat datang ke client baru
    ws_stream.send(Message::text("Welcome to chat! Type a message")).await?;

    // Buat receiver (subscriber) untuk channel broadcast
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            // Task 1: Menerima pesan dari client ini, lalu broadcast ke semua
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From client {addr:?} {text:?}");
                            bcast_tx.send(text.into())?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            // Task 2: Menerima pesan dari channel broadcast, lalu kirim ke client ini
            msg = bcast_rx.recv() => {
                let text = msg?;
                ws_stream.send(Message::text(text)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            match ServerBuilder::new().accept(socket).await {
                Ok((_req, ws_stream)) => {
                    if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                        println!("Error in connection {addr:?}: {e}");
                    }
                }
                Err(e) => println!("Error accepting connection {addr:?}: {e}"),
            }
        });
    }
}