use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[tokio::main]
async fn main() {
    use std::io::{self, Write};
    
    eprintln!("Starting simple care-voice backend...");
    io::stderr().flush().unwrap();
    
    let app = Router::new()
        .route("/health", get(health_check));
    
    eprintln!("Binding to 0.0.0.0:8000...");
    io::stderr().flush().unwrap();
    
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8000").await {
        Ok(l) => {
            eprintln!("Successfully bound to port 8000");
            io::stderr().flush().unwrap();
            l
        },
        Err(e) => {
            eprintln!("Failed to bind: {}", e);
            io::stderr().flush().unwrap();
            std::process::exit(1);
        }
    };
    
    eprintln!("Server running on http://0.0.0.0:8000");
    io::stderr().flush().unwrap();
    
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        io::stderr().flush().unwrap();
    }
    eprintln!("Server exited");
    io::stderr().flush().unwrap();
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Simple backend is running".to_string(),
    })
}