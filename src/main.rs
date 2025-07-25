use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::OpenOptions;
use tokio::sync::Mutex;
use chrono::Local;
use std::path::Path;
use std::sync::Arc;

async fn handle_client(mut socket: TcpStream, log_file_path_mutex: Arc<Mutex<String>>) {
    let peer_addr = match socket.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown".to_string(),
    };
    
    println!("Client connected: {}", peer_addr);
    
    let mut buffer = [0u8; 1024];
    
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected: {}", peer_addr);
                break;
            },
            Ok(n) => {
                if let Ok(message) = String::from_utf8(buffer[0..n].to_vec()) {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
                    let log_entry = format!("[{}] [{}]: {}\n", timestamp, peer_addr, message.trim());
                    print!("{}", log_entry);
                    if let Err(e) = write_to_log(&log_file_path_mutex, &log_entry).await {
                        eprintln!("Error writing to log file: {}", e);
                    }
                    
                    let response = format!("Message logged at {}\n", timestamp);
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Error writing to client: {}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}

async fn write_to_log(log_file_path_mutex: &Arc<Mutex<String>>, log_entry: &str) -> Result<(), Box<dyn std::error::Error>> {
    // On verrouille le mutex avant d'ouvrir/écrire dans le fichier
    let log_file_path = {
        let guard = log_file_path_mutex.lock().await;
        guard.clone()
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&log_file_path)
        .await?;
    
    file.write_all(log_entry.as_bytes()).await?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let log_file_path = Arc::new(Mutex::new("server_logs.txt".to_string()));
    
    // On récupère la valeur dans le mutex pour vérifier le chemin
    {
        let guard = log_file_path.lock().await;
        if let Some(parent) = Path::new(&*guard).parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create log directory: {}", e);
                    return;
                }
            }
        }
    }
    
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            return;
        }
    };
    
    {
        let guard = log_file_path.lock().await;
        println!("Async logging server started on {}", addr);
        println!("Logs will be written to {}", *guard);
    }
    
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                let log_file_path_clone = Arc::clone(&log_file_path);
                
                tokio::spawn(async move {
                    handle_client(socket, log_file_path_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
