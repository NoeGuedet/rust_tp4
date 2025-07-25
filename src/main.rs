use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use futures::{SinkExt, StreamExt};
use std::env;
use url::Url;

async fn websocket_server(addr: &str) {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind WebSocket server");

    println!("WebSocket server running on {}", addr);

    let (tx, _rx) = broadcast::channel::<String>(100);

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Accept error: {}", e);
                continue;
            }
        };
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        println!("New client: {}", peer_addr);

        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake error: {}", e);
                    return;
                }
            };
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let send_task = tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    if ws_sender
                        .send(Message::Text(msg))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });

            while let Some(Ok(msg)) = ws_receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        println!("Received: {}", text);
                        let _ = tx.send(text);
                    }
                    Message::Binary(bin) => {
                        println!("Received binary ({} bytes)", bin.len());
                        let _ = tx.send(format!("[binary msg: {} bytes]", bin.len()));
                    }
                    Message::Close(_) => {
                        println!("Client disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            send_task.abort();
        });
    }
}

async fn websocket_client(url: &str) {
    let url = Url::parse(url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket client connected!");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let send_task = tokio::spawn(async move {
        use tokio::io::{self, AsyncBufReadExt, BufReader};

        let stdin = io::stdin();
        let mut lines = BufReader::new(stdin).lines();

        println!("Type messages, enter 'quit' to exit.");
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim() == "quit" {
                let _ = ws_sender.send(Message::Close(None)).await;
                break;
            } else {
                let _ = ws_sender.send(Message::Text(line)).await;
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => println!("Server: {}", text),
            Ok(Message::Binary(bin)) => println!("Server sent binary ({} bytes)", bin.len()),
            Ok(Message::Close(_)) => {
                println!("Connection closed by server.");
                break;
            }
            _ => {}
        }
    }

    let _ = send_task.await;
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:");
        println!("  {} server [address:port]", args[0]);
        println!("  {} client [ws://address:port]", args[0]);
        return;
    }

    match args[1].as_str() {
        "server" => {
            let addr = args.get(2).cloned().unwrap_or_else(|| "127.0.0.1:8080".to_string());
            websocket_server(&addr).await;
        }
        "client" => {
            let url = args.get(2).cloned().unwrap_or_else(|| "ws://127.0.0.1:8080".to_string());
            websocket_client(&url).await;
        }
        _ => {
            println!("Unknown subcommand. Use 'server' or 'client'.");
        }
    }
}
