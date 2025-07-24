use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Starting sync server...");
    
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(listener) => {
            println!("✓ Successfully bound to 0.0.0.0:8000");
            listener
        }
        Err(e) => {
            println!("✗ Failed to bind: {}", e);
            std::process::exit(1);
        }
    };

    println!("🚀 Sync server ready on http://0.0.0.0:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    if let Ok(_) = stream.read(&mut buffer) {
        let response = "HTTP/1.1 200 OK\r\n\r\n{\"status\":\"healthy\",\"service\":\"Sync Server\"}";
        let _ = stream.write(response.as_bytes());
        let _ = stream.flush();
    }
}