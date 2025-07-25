use serde::{Deserialize, Serialize};
use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

/// Codes d'opération du protocole
#[derive(Serialize, Deserialize, Debug)]
enum OpCode {
    Message,      // Envoi d'un message texte
    Ack,          // Accusé de réception
    Error,        // Erreur de protocole
}

/// Structure d'un message du protocole
#[derive(Serialize, Deserialize, Debug)]
struct ProtocolMessage {
    opcode: OpCode,
    username: Option<String>,
    content: Option<String>,
    error: Option<String>,
}

// Gestion simple d'erreurs avec thiserror (custom, minimaliste ici)
#[derive(Debug)]
enum ProtoError {
    Io(io::Error),
    Serde(serde_json::Error),
}
impl From<io::Error> for ProtoError {
    fn from(e: io::Error) -> Self { ProtoError::Io(e) }
}
impl From<serde_json::Error> for ProtoError {
    fn from(e: serde_json::Error) -> Self { ProtoError::Serde(e) }
}

/// Sérialise un message et l’envoie sur la connexion TCP
fn send_message(stream: &mut TcpStream, msg: &ProtocolMessage) -> Result<(), ProtoError> {
    let serialized = serde_json::to_string(msg)?;
    stream.write_all(serialized.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

/// Lit un message d'une connexion TCP
fn read_message(stream: &mut TcpStream) -> Result<ProtocolMessage, ProtoError> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let msg = serde_json::from_str(&line)?;
    Ok(msg)
}

/// Lancer un serveur TCP : protocole simple d’échange de messages
fn start_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Serveur démarré sur {}", addr);

    for stream in listener.incoming() {
        thread::spawn(|| {
            match stream {
                Ok(mut stream) => {
                    println!("Client connecté: {}", stream.peer_addr().unwrap());
                    if let Err(e) = handle_client(&mut stream) {
                        println!("Erreur protocole: {:?}", e);
                    }
                }
                Err(e) => eprintln!("Erreur connexion: {}", e),
            }
        });
    }
    Ok(())
}

fn handle_client(stream: &mut TcpStream) -> Result<(), ProtoError> {
    loop {
        let msg = match read_message(stream) {
            Ok(m) => m,
            Err(_) => break, // Connexion fermée ou message mal formé
        };
        println!("Reçu: {:?}", msg);

        match msg.opcode {
            OpCode::Message => {
                // Accusé de réception
                let ack = ProtocolMessage {
                    opcode: OpCode::Ack,
                    username: msg.username.clone(),
                    content: Some("Message reçu".to_string()),
                    error: None,
                };
                send_message(stream, &ack)?;
            }
            _ => {
                let err = ProtocolMessage {
                    opcode: OpCode::Error,
                    username: None,
                    content: None,
                    error: Some("Opération non supportée".to_string()),
                };
                send_message(stream, &err)?;
            }
        }
    }
    Ok(())
}

/// Client : envoie des messages puis termine sur EOF (Ctrl+D)
fn start_client(addr: &str, username: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("Connecté au serveur: {}", addr);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let msg = ProtocolMessage {
            opcode: OpCode::Message,
            username: Some(username.to_string()),
            content: Some(line),
            error: None,
        };
        send_message(&mut stream, &msg).expect("Erreur envoi message");

        // Lecture de la réponse serveur
        match read_message(&mut stream) {
            Ok(rep) => println!("Réponse serveur: {:?}", rep),
            Err(_) => println!("Erreur ou serveur déconnecté."),
        }
    }
    Ok(())
}

/// CLI simple pour lancer comme serveur ou client
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage:");
        println!("  {} server [IP:port]", args[0]);
        println!("  {} client [IP:port] [username]", args[0]);
        return;
    }
    match args[1].as_str() {
        "server" => {
            let addr = args.get(2).map(|s| &**s).unwrap_or("127.0.0.1:9000");
            start_server(addr).expect("Erreur serveur");
        }
        "client" => {
            if args.len() < 4 {
                println!("client: usage: {} client [IP:port] [username]", args[0]);
                return;
            }
            let addr = &args[2];
            let username = &args[3];
            start_client(addr, username).expect("Erreur client");
        }
        _ => println!("Commande non reconnue"),
    }
}
