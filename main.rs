use rand::Rng;
use sha256;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Server listening on 127.0.0.1:7878");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        tokio::spawn(handle_client(stream, addr));
    }
}

async fn handle_client(mut stream: TcpStream, addr: std::net::SocketAddr) {
    let mut buf = [0u8; 1024];
    
    loop {
        match stream.read(&mut buf).await {
            Ok(n) => {
                let input = &buf[..n];
                println!("Received from {}: {:?}", addr, String::from_utf8_lossy(input));
                if input == "POST\n".as_bytes() {
                    // Handle POST mode directly here instead of moving stream
                    handle_client_post(&mut stream, addr).await;
                    
                } else {
                    println!("Client {} didnot start POST mode", addr);
                }
                break; // Exit the main loop after handling POST
            }
            Err(e) => {
                eprintln!("Read error from {}: {}", addr, e);
                break;
            }
        }
    }
}

async fn handle_client_post(mut stream: &mut TcpStream, addr: std::net::SocketAddr) {
    let mut buf = [0u8; 1024];
    let mut buffer = String::new();
    
    // Generate random 32-byte challenge
    let mut challenge_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut challenge_bytes);
    let challenge = hex::encode(challenge_bytes);


    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                println!("Client {}: disconnected from POST mode", addr);
                break;
            }
            Ok(n) => {
                let input = &buf[..n];
                
                if input == "SUBMIT\n".as_bytes() {
                    println!("Challenge: {}", challenge);
                    // Handle POST mode directly here instead of moving stream
                    if let Err(e) = stream.write_all(format!("CHALLENGE: {}\n", challenge).as_bytes()).await {
                        eprintln!("Write error to {}: {}", addr, e);
                        break;
                    }
                    handle_client_accept(&mut stream, addr, buffer, challenge).await;
                    break; // Exit the main loop after handling POST
                }
                println!("Received from {}: {:?}", addr, String::from_utf8_lossy(input));
                buffer.push_str(&String::from_utf8_lossy(input));
                // Handle the file content here
                // For now, just echo back what was received
                
            }
            Err(e) => {
                eprintln!("Read error from {}: {}", addr, e);
                break;
            }
        }
    }
}

async fn handle_client_accept(stream: &mut TcpStream, addr: std::net::SocketAddr, buffer: String, challenge: String) {
    let mut buf = [0u8; 1024];

    let mut file_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut file_bytes);
    let file_id = hex::encode(file_bytes);


    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                println!("Client {}: disconnected from ACCEPT mode", addr);
                break;
            }
            Ok(n) => {
                let input = String::from_utf8_lossy(&buf[..n]);
                if input.contains("ACCEPTED") {
                    println!("Client {}: in accept mode {}", addr, input);
                    let prefix = String::from(&input[9..]);
                    let combined = format!("{}{}{}\n", prefix, buffer, challenge);
                    // println!("combined:---{}---", combined);
                    let hash = sha256::digest(combined);
                    if hash.starts_with("000000") {
                        if let Err(e) = std::fs::write(&file_id, &buffer) {
                            eprintln!("Failed to create file {}: {}", file_id, e);
                        } else {
                            println!("Created file: {}", file_id);
                            if let Err(e) = stream.write_all(format!("Challenge Accepted... Creating File\nFILE_ID: {}\n", file_id).as_bytes()).await {
                                eprintln!("Write error to {}: {}", addr, e);
                                break;
                            }
                        }
                    }
                }
                break;
            }
            Err(e) => {
                eprintln!("Read error from {}: {}", addr, e);
                break;
            }
        }
    }
}
