use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
            .connect()
            .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Task 1: Membaca input dari terminal user (stdin), kirim ke server
            line = stdin.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        ws_stream.send(Message::text(text)).await?;
                    }
                    Ok(None) => break,
                    Err(err) => {
                        println!("Error reading stdin: {}", err);
                        break;
                    }
                }
            }
            // Task 2: Menerima pesan dari server, tampilkan di terminal
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From server: {}", text);
                        }
                    }
                    Some(Err(err)) => {
                        println!("Error receiving message: {}", err);
                        break;
                    }
                    None => {
                        println!("Connection closed by server.");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}