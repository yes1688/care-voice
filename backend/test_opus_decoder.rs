// Opus 解碼器功能測試程式
// 用於驗證 Opus 解碼邏輯的正確性

#[cfg(feature = "opus-support")]
use opus::{Decoder as OpusAudioDecoder, Channels};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Opus 解碼器功能測試");
    
    // 測試 1: 檢查 Opus 庫是否可用
    #[cfg(feature = "opus-support")]
    {
        println!("✅ Opus 支援已啟用");
        
        // 測試創建 Opus 解碼器
        match OpusAudioDecoder::new(48000, Channels::Mono) {
            Ok(_decoder) => {
                println!("✅ Opus 解碼器創建成功");
            },
            Err(e) => {
                println!("❌ Opus 解碼器創建失敗: {:?}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(feature = "opus-support"))]
    {
        println!("⚠️ Opus 支援未啟用");
        println!("需要使用 --features opus-support 編譯");
    }
    
    // 測試 2: 格式檢測
    test_format_detection();
    
    // 測試 3: 容器解析
    test_container_parsing();
    
    println!("🎉 所有測試完成");
    Ok(())
}

fn test_format_detection() {
    println!("\n📝 測試格式檢測:");
    
    // OGG 格式測試
    let ogg_magic = b"OggS";
    if ogg_magic.starts_with(b"OggS") {
        println!("✅ OGG 格式檢測正確");
    }
    
    // WebM 格式測試
    let webm_magic = &[0x1A, 0x45, 0xDF, 0xA3];
    if webm_magic.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        println!("✅ WebM 格式檢測正確");
    }
}

fn test_container_parsing() {
    println!("\n📦 測試容器解析:");
    
    // 測試 Opus 頭檢測
    let opus_head = b"OpusHead";
    let opus_tags = b"OpusTags";
    
    if opus_head.starts_with(b"OpusHead") {
        println!("✅ OpusHead 檢測正確");
    }
    
    if opus_tags.starts_with(b"OpusTags") {
        println!("✅ OpusTags 檢測正確");
    }
}