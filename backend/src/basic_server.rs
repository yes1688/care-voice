use axum::{
    response::Json,
    routing::get,
    Router,
};

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "Basic Care Voice Server"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Starting Basic Server ===");
    
    let app = Router::new()
        .route("/health", get(health_check));
    
    println!("Binding to 0.0.0.0:8000...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    
    println!("🚀 Basic server ready on http://0.0.0.0:8000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}