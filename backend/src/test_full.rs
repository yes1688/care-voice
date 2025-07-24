use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tracing::{info, error};
use whisper_rs::{WhisperContext, WhisperContextParameters};

// 簡化版 WhisperService
struct WhisperService {
    context: WhisperContext,
}

impl WhisperService {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Creating Whisper service...");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let model_path = "./models/ggml-base.bin";
        println!("Loading model from: {}", model_path);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )?;
        
        println!("✓ Whisper service created successfully!");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        Ok(Self { context: ctx })
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "Test Care Voice with whisper-rs"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Starting Full Test Application ===");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    // 初始化日誌
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    
    info!("Tracing initialized");
    
    println!("Step 1: Initializing Whisper service...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let whisper_service = match WhisperService::new() {
        Ok(service) => {
            println!("✓ Whisper service initialized");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            Arc::new(service)
        }
        Err(e) => {
            println!("✗ Failed to initialize Whisper service: {}", e);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            return Err(e);
        }
    };
    
    println!("Step 2: Setting up router...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(tower_http::cors::Any);
    
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(whisper_service);
    
    println!("Step 3: Binding to port 8000...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8000").await {
        Ok(listener) => {
            println!("✓ Successfully bound to 0.0.0.0:8000");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            listener
        }
        Err(e) => {
            println!("✗ Failed to bind to port 8000: {}", e);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            return Err(e.into());
        }
    };
    
    println!("Step 4: Starting server...");
    println!("🚀 Server ready on http://0.0.0.0:8000");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    info!("Server starting on http://0.0.0.0:8000");
    
    if let Err(e) = axum::serve(listener, app).await {
        println!("✗ Server error: {}", e);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        return Err(e.into());
    }
    
    Ok(())
}