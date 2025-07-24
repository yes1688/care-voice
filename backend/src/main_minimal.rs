use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    eprintln!("Starting minimal HTTP server...");
    
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(l) => {
            eprintln!("Successfully bound to 0.0.0.0:8000");
            l
        },
        Err(e) => {
            eprintln!("Failed to bind: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("Server listening on http://0.0.0.0:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\",\"message\":\"Minimal server running\"}";

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}