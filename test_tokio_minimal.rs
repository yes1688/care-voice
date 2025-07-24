// 最小化 tokio 測試程序 - 排除異步運行時問題
// 用於診斷 whisper-rs 是否因為 tokio 運行時問題而退出

use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting minimal tokio test...");
    
    // 測試基本的異步功能
    println!("⏰ Testing async sleep...");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("✅ Async sleep completed");
    
    // 測試 spawn
    println!("🔧 Testing tokio spawn...");
    let handle = tokio::spawn(async {
        println!("📝 Inside spawned task");
        "task completed"
    });
    
    let result = handle.await?;
    println!("✅ Spawn result: {}", result);
    
    // 測試網路綁定 (類似 whisper-rs 的用法)
    println!("🌐 Testing TCP listener...");
    match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => {
            println!("✅ TCP listener created: {}", listener.local_addr()?);
        },
        Err(e) => {
            println!("❌ TCP listener failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("🎉 All tokio tests passed!");
    println!("🔍 If this works but whisper-rs fails, the problem is in whisper-rs/C++ bindings");
    
    Ok(())
}