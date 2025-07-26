// 純音頻處理測試程式 - 不依賴 whisper-rs
// 用於驗證我們的 Opus 音頻處理邏輯

mod audio_format;
mod opus_decoder;
mod audio_decoder;

use audio_format::AudioFormat;
use audio_decoder::UnifiedAudioDecoder;

fn main() {
    println!("🎵 Care Voice 音頻處理測試");
    println!("=======================");

    // 測試格式檢測
    test_format_detection();
    
    // 測試支援資訊
    test_support_info();
    
    // 測試解碼器創建
    test_decoder_creation();
    
    println!("\n✅ 音頻處理邏輯測試完成");
}

fn test_format_detection() {
    println!("\n📋 測試音頻格式檢測:");
    
    let test_cases = [
        ("WebM 頭", vec![0x1A, 0x45, 0xDF, 0xA3, 0x00, 0x00, 0x00, 0x20]),
        ("OGG 頭", b"OggS\x00\x02\x00\x00".to_vec()),
        ("WAV 頭", b"RIFF\x24\x08\x00\x00WAVE".to_vec()),
        ("MP4 頭", b"\x00\x00\x00\x20ftypM4A ".to_vec()),
    ];

    for (name, data) in test_cases.iter() {
        let format = AudioFormat::detect_from_data(data);
        println!("  {} -> {}", name, format.friendly_name());
    }
}

fn test_support_info() {
    println!("\n📊 音頻格式支援狀態:");
    
    let support_info = UnifiedAudioDecoder::get_format_support_info();
    for (format, status) in support_info {
        println!("  {} - {}", format.friendly_name(), status);
    }
}

fn test_decoder_creation() {
    println!("\n🔧 測試解碼器創建:");
    
    // 測試 OGG 數據
    let ogg_data = b"OggS\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00";
    match UnifiedAudioDecoder::decode_audio(AudioFormat::OggOpus, ogg_data) {
        Ok(_) => println!("  OGG-Opus: 解碼成功"),
        Err(e) => println!("  OGG-Opus: {} ", e),
    }
    
    // 測試 WebM 數據
    let webm_data = [0x1A, 0x45, 0xDF, 0xA3, 0x00, 0x00, 0x00, 0x20];
    match UnifiedAudioDecoder::decode_audio(AudioFormat::WebmOpus, &webm_data) {
        Ok(_) => println!("  WebM-Opus: 解碼成功"),
        Err(e) => println!("  WebM-Opus: {}", e),
    }
}