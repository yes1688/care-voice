# 🛠️ Opus 後端處理實施指南

## 📋 實施概要

**目標**: 實施業界標準的 Opus 後端處理方案  
**基於**: [Opus 後端處理解決方案](../technical/OPUS_BACKEND_SOLUTION.md)  
**預計時間**: 2-3 天  
**難度等級**: 中等  
**成功標準**: 95% 瀏覽器音頻格式相容性

---

## 📚 實施前準備

### 環境要求
```bash
# 系統要求
- Rust 1.70+
- CUDA 12.9.1
- Podman/Docker
- 開發環境已設置 (參考: environment-setup.md)

# 檢查當前環境
rustc --version  # 確保 >= 1.70
cargo --version
podman --version
```

### 備份當前系統
```bash
# 1. 備份當前運行容器
podman commit care-voice-ultimate care-voice:backup-before-opus

# 2. 備份配置文件
cp backend/Cargo.toml backend/Cargo.toml.backup
cp backend/src/main.rs backend/src/main.rs.backup

# 3. 記錄當前狀態
curl -s http://localhost:8001/health > current-status.json
```

### 依賴研究
```bash
# 查看可用的 Opus 相關 crate
cargo search opus
cargo search ogg
cargo search webm

# 檢查版本相容性
cargo info opus
cargo info ogg
```

---

## 🚀 階段 1: 依賴配置 (30分鐘)

### 1.1 更新 Cargo.toml
```toml
# backend/Cargo.toml - 新增音頻解碼依賴

[dependencies]
# 現有依賴保持不變
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1.0", features = ["full"] }
whisper-rs = { version = "0.14.3", features = ["cuda"] }

# 音頻處理 - 新增 Opus 支援
symphonia = { version = "0.5", features = [
    "mkv", "vorbis", "flac", "wav"    # 保持現有功能
] }
hound = "3.5"

# 新增: Opus 音頻解碼器
opus = "0.3.0"                       # 原生 Opus 解碼
ogg = "0.9.0"                        # OGG 容器支援 (Firefox)

# 新增: WebM 容器支援 (Chrome/Edge)
# 注意: 可能需要尋找合適的 WebM parser crate
# matroska = "0.1.0"  # 如果可用

# 音頻樣本處理工具
byteorder = "1.4"                    # 字節序處理
```

### 1.2 依賴測試編譯
```bash
# 進入後端目錄
cd backend

# 清理並測試新依賴
cargo clean
cargo check

# 如果編譯失敗，檢查 crate 可用性
cargo tree
```

---

## 🔧 階段 2: 核心實現 (4-6小時)

### 2.1 音頻格式檢測
```rust
// backend/src/audio_format.rs (新文件)
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    WebmOpus,   // Chrome/Edge
    OggOpus,    // Firefox
    Mp4Aac,     // Safari  
    Wav,        // 通用格式
    Unknown,
}

impl AudioFormat {
    pub fn detect_from_data(data: &[u8]) -> Self {
        if data.len() < 16 {
            return AudioFormat::Unknown;
        }

        // WebM 魔術數字檢測
        if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            return AudioFormat::WebmOpus;
        }

        // OGG 魔術數字檢測
        if data.starts_with(b"OggS") {
            return AudioFormat::OggOpus;
        }

        // MP4 魔術數字檢測 (ftyp box)
        if data.len() >= 8 && &data[4..8] == b"ftyp" {
            return AudioFormat::Mp4Aac;
        }

        // WAV 魔術數字檢測
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WAVE" {
            return AudioFormat::Wav;
        }

        AudioFormat::Unknown
    }

    pub fn detect_from_mime(mime_type: &str) -> Self {
        match mime_type {
            "audio/webm" | "audio/webm;codecs=opus" => AudioFormat::WebmOpus,
            "audio/ogg" | "audio/ogg;codecs=opus" => AudioFormat::OggOpus,
            "audio/mp4" | "audio/mp4;codecs=mp4a.40.2" => AudioFormat::Mp4Aac,
            "audio/wav" => AudioFormat::Wav,
            _ => AudioFormat::Unknown,
        }
    }
}
```

### 2.2 Opus 解碼器實現
```rust
// backend/src/opus_decoder.rs (新文件)
use opus::{Decoder, Channels, Application};
use ogg::{PacketReader, Packet};
use std::io::Cursor;

pub struct OpusDecoder {
    decoder: Decoder,
    sample_rate: u32,
    channels: Channels,
}

impl OpusDecoder {
    pub fn new(sample_rate: u32, channels: Channels) -> Result<Self, Box<dyn std::error::Error>> {
        let decoder = Decoder::new(sample_rate, channels)?;
        Ok(Self {
            decoder,
            sample_rate,
            channels,
        })
    }

    pub fn decode_ogg_opus(&mut self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(data);
        let mut packet_reader = PacketReader::new(&mut cursor);
        let mut samples = Vec::new();

        // 讀取 OGG 頁面和數據包
        while let Ok(packet) = packet_reader.read_packet() {
            if let Some(packet_data) = packet.data {
                // 解碼 Opus 數據包
                let mut output = vec![0f32; self.sample_rate as usize]; // 1秒緩衝區
                let len = self.decoder.decode_float(&packet_data, &mut output, false)?;
                output.truncate(len);
                samples.extend(output);
            }
        }

        Ok(samples)
    }

    pub fn decode_webm_opus(&mut self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // WebM 解析較複雜，可能需要專用庫
        // 暫時先實現簡化版本，後續可以改進
        
        // 尋找 Opus 數據包在 WebM 中的位置
        // 這是一個簡化的實現，生產環境需要更完整的 WebM parser
        
        // 嘗試從數據中提取音頻部分
        let opus_data = self.extract_opus_from_webm(data)?;
        
        // 解碼提取的 Opus 數據
        let mut output = vec![0f32; self.sample_rate as usize * 10]; // 10秒緩衝區
        let len = self.decoder.decode_float(&opus_data, &mut output, false)?;
        output.truncate(len);
        
        Ok(output)
    }

    fn extract_opus_from_webm(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 簡化實現：尋找可能的 Opus 數據
        // 生產實現需要完整的 WebM/Matroska 解析器
        
        // 暫時返回原始數據，讓 Opus 解碼器嘗試處理
        // TODO: 實現完整的 WebM 解析
        Ok(data.to_vec())
    }
}
```

### 2.3 統一解碼器介面
```rust
// backend/src/audio_decoder.rs (新文件)
use crate::audio_format::AudioFormat;
use crate::opus_decoder::OpusDecoder;

pub trait AudioDecoder {
    fn decode(&mut self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>>;
}

pub struct UnifiedAudioDecoder;

impl UnifiedAudioDecoder {
    pub fn decode_audio(format: AudioFormat, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        match format {
            AudioFormat::WebmOpus => {
                info!("解碼 WebM Opus 格式");
                let mut decoder = OpusDecoder::new(48000, opus::Channels::Mono)?;
                decoder.decode_webm_opus(data)
            },
            AudioFormat::OggOpus => {
                info!("解碼 OGG Opus 格式");
                let mut decoder = OpusDecoder::new(48000, opus::Channels::Mono)?;
                decoder.decode_ogg_opus(data)
            },
            AudioFormat::Mp4Aac => {
                info!("解碼 MP4 AAC 格式");
                // TODO: 實現 AAC 解碼器 (可使用 FFmpeg 或專用庫)
                Err("AAC 解碼尚未實現".into())
            },
            AudioFormat::Wav => {
                info!("解碼 WAV 格式");
                // 使用現有的 WAV 解碼邏輯
                try_read_as_wav(data)
            },
            AudioFormat::Unknown => {
                warn!("未知音頻格式，嘗試現有解碼器");
                // 回退到現有的 symphonia 解碼
                try_decode_with_symphonia(data)
            }
        }
    }
}

// 重用現有函數
fn try_read_as_wav(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // 現有實現保持不變
    // ... (從 main.rs 複製現有實現)
}

fn try_decode_with_symphonia(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // 現有實現保持不變  
    // ... (從 main.rs 複製現有實現)
}
```

---

## 🔌 階段 3: 系統整合 (2-3小時)

### 3.1 更新 main.rs
```rust
// backend/src/main.rs - 添加新模組和使用

// 新增模組聲明
mod audio_format;
mod opus_decoder;  
mod audio_decoder;

use audio_format::AudioFormat;
use audio_decoder::UnifiedAudioDecoder;

// 更新音頻轉換函數
fn convert_to_wav_samples(audio_data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error + '_>> {
    info!("開始音頻格式轉換，數據大小: {} bytes", audio_data.len());
    
    // 1. 檢測音頻格式
    let format = AudioFormat::detect_from_data(audio_data);
    info!("檢測到音頻格式: {:?}", format);
    
    // 2. 使用統一解碼器
    match UnifiedAudioDecoder::decode_audio(format, audio_data) {
        Ok(samples) => {
            info!("音頻解碼成功，樣本數: {}", samples.len());
            Ok(samples)
        },
        Err(e) => {
            error!("音頻解碼失敗: {}", e);
            Err(e)
        }
    }
}

// 更新上傳處理函數以傳遞 MIME 類型
async fn upload_audio(
    State(whisper_service): State<Arc<WhisperService>>,
    mut multipart: Multipart,
) -> Result<Json<TranscriptResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("收到音頻上傳請求");
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("讀取 multipart 欄位錯誤: {}", e);
        (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "無效的 multipart 數據".to_string() }))
    })? {
        
        if field.name() == Some("audio") {
            info!("處理音頻欄位");
            
            // 獲取 MIME 類型
            let content_type = field.content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            info!("音頻 MIME 類型: {}", content_type);
            
            let data = field.bytes().await.map_err(|e| {
                error!("讀取音頻位元組錯誤: {}", e);
                (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "讀取音頻數據失敗".to_string() }))
            })?;
            
            info!("收到音頻數據: {} bytes", data.len());
            
            // 轉換音頻格式，使用 MIME 類型資訊
            let audio_samples = convert_to_wav_samples_with_mime(&data, &content_type).map_err(|e| {
                error!("音頻轉換失敗: {}", e);
                CONVERSION_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
                
                // 根據錯誤類型提供友善的錯誤信息
                let user_message = if e.to_string().contains("Opus") {
                    "✅ Opus 格式支援已啟用！如果仍有問題，請檢查容器版本。"
                } else if e.to_string().contains("WebM") {
                    "WebM 容器解析問題，正在改進中。建議暫時使用 Firefox (OGG 格式)。"
                } else {
                    "音頻格式轉換失敗。支援格式：WAV, WebM (Opus), OGG (Opus)"
                };
                
                (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse { 
                    error: user_message.to_string() 
                }))
            })?;
            
            // 轉換成功統計
            CONVERSION_SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
            
            // 其餘邏輯保持不變...
        }
    }
    
    // ... 其餘代碼不變
}

// 新增支援 MIME 類型的轉換函數
fn convert_to_wav_samples_with_mime(
    audio_data: &[u8], 
    mime_type: &str
) -> Result<Vec<f32>, Box<dyn std::error::Error + '_>> {
    info!("開始音頻格式轉換，數據大小: {} bytes，MIME: {}", audio_data.len(), mime_type);
    
    // 1. 優先使用 MIME 類型檢測
    let format = AudioFormat::detect_from_mime(mime_type);
    
    // 2. 如果 MIME 檢測失敗，使用數據檢測
    let format = if format == AudioFormat::Unknown {
        AudioFormat::detect_from_data(audio_data)
    } else {
        format
    };
    
    info!("檢測到音頻格式: {:?} (來源: MIME={})", format, mime_type);
    
    // 3. 解碼音頻
    UnifiedAudioDecoder::decode_audio(format, audio_data)
}
```

### 3.2 更新模組結構
```rust
// backend/src/lib.rs (如果使用) 或在 main.rs 中

pub mod audio_format;
pub mod opus_decoder;
pub mod audio_decoder;

// 重新導出主要介面
pub use audio_format::AudioFormat;
pub use audio_decoder::UnifiedAudioDecoder;
```

---

## 🧪 階段 4: 測試和驗證 (2-3小時)

### 4.1 單元測試
```rust
// backend/src/audio_format.rs - 添加測試

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webm_detection() {
        let webm_header = [0x1A, 0x45, 0xDF, 0xA3, 0x00, 0x00, 0x00, 0x20];
        assert_eq!(AudioFormat::detect_from_data(&webm_header), AudioFormat::WebmOpus);
    }

    #[test]
    fn test_ogg_detection() {
        let ogg_header = b"OggS\x00\x02\x00\x00";
        assert_eq!(AudioFormat::detect_from_data(ogg_header), AudioFormat::OggOpus);
    }

    #[test]
    fn test_mime_detection() {
        assert_eq!(AudioFormat::detect_from_mime("audio/webm;codecs=opus"), AudioFormat::WebmOpus);
        assert_eq!(AudioFormat::detect_from_mime("audio/ogg;codecs=opus"), AudioFormat::OggOpus);
    }
}
```

### 4.2 編譯測試
```bash
# 在 backend 目錄中
cargo test
cargo check
cargo build --release
```

### 4.3 容器重建
```bash
# 返回專案根目錄
cd ..

# 重建容器
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:opus-support .

# 如果建構成功，部署測試
podman stop care-voice-ultimate 2>/dev/null || true
podman rm care-voice-ultimate 2>/dev/null || true

podman run -d \
  --name care-voice-ultimate \
  --device /dev/nvidia0 \
  --device /dev/nvidiactl \
  --device /dev/nvidia-uvm \
  --device /dev/nvidia-modeset \
  -p 8001:8001 \
  -v ./backend/models:/app/models:ro \
  -e LD_LIBRARY_PATH="/usr/local/cuda/lib64:/usr/local/cuda-12.9/compat:${LD_LIBRARY_PATH}" \
  -e CUDA_VISIBLE_DEVICES=0 \
  care-voice:opus-support
```

### 4.4 功能測試
```bash
# 等待服務啟動
sleep 20

# 檢查健康狀態
curl -s http://localhost:8001/health | jq

# 檢查服務日誌
podman logs --tail 20 care-voice-ultimate
```

---

## 🌐 階段 5: 瀏覽器測試 (1-2小時)

### 5.1 Chrome 測試
1. 開啟 Chrome 瀏覽器
2. 訪問 http://localhost:8001
3. 點擊錄音按鈕
4. 錄製 5-10 秒音頻
5. 點擊轉換為文字
6. **預期結果**: 成功轉錄，無 422 錯誤

### 5.2 Firefox 測試
1. 開啟 Firefox 瀏覽器
2. 重複上述步驟
3. **預期結果**: 成功轉錄

### 5.3 Edge 測試
1. 開啟 Edge 瀏覽器
2. 重複上述步驟  
3. **預期結果**: 成功轉錄

### 5.4 Safari 測試 (如有 macOS)
1. 確保使用 HTTPS 或設置為安全上下文
2. 開啟 Safari 瀏覽器
3. 重複上述步驟
4. **預期結果**: 成功轉錄 (AAC 格式)

### 5.5 錯誤監控
```bash
# 實時監控錯誤日誌
podman logs -f care-voice-ultimate | grep -E "(錯誤|ERROR|failed)"

# 檢查統計計數
podman exec care-voice-ultimate grep -E "(CONVERSION_|COUNT)" /var/log/supervisor/whisper-rs.log
```

---

## 🛠️ 故障排除

### 常見問題 1: Opus 依賴編譯失敗
```bash
# 症狀: cargo build 失敗，Opus 相關錯誤
# 解決方案: 檢查系統依賴

# Ubuntu/Debian
sudo apt update
sudo apt install libopus-dev

# 或者嘗試不同版本的 opus crate
# 在 Cargo.toml 中:
opus = "0.2.0"  # 如果 0.3.0 有問題
```

### 常見問題 2: OGG 解析失敗
```bash
# 症狀: OGG 格式檢測或解碼失敗
# 檢查: OGG crate 相容性

# 嘗試替代實現
# 在 Cargo.toml 中:
ogg = "0.8.0"  # 如果 0.9.0 有問題
```

### 常見問題 3: WebM 解析複雜
```bash
# 症狀: WebM 格式無法正確解析
# 暫時解決方案: 專注於 OGG 支援

# 階段性實施:
# 1. 先完成 OGG Opus (Firefox)
# 2. 後續添加 WebM 支援 (Chrome/Edge)
```

### 常見問題 4: 容器啟動失敗
```bash
# 檢查依賴衝突
podman logs care-voice-ultimate

# 回退到備份版本
podman stop care-voice-ultimate
podman rm care-voice-ultimate
podman run -d --name care-voice-ultimate care-voice:backup-before-opus
```

---

## 📈 成功驗證清單

### 技術指標
- [ ] **編譯成功**: cargo build --release 無錯誤
- [ ] **容器建構**: Dockerfile 建構成功
- [ ] **服務啟動**: 健康檢查返回正常狀態
- [ ] **OGG Opus**: Firefox 錄音轉錄成功
- [ ] **WebM Opus**: Chrome 錄音轉錄成功 (如實現)
- [ ] **錯誤處理**: 未支援格式有友善錯誤信息

### 性能指標
- [ ] **記憶體使用**: 未顯著增加 (< 100MB)
- [ ] **處理時間**: 音頻轉換 < 1秒
- [ ] **成功率**: 錯誤率 < 5%

### 用戶體驗指標
- [ ] **瀏覽器支援**: 至少 2 個瀏覽器正常工作
- [ ] **錯誤信息**: 清晰且可操作
- [ ] **向後相容**: 現有 WAV 功能未受影響

---

## 🔄 後續優化方向

### 短期優化 (1週內)
1. **WebM 解析改進**: 尋找更好的 WebM 解析庫
2. **AAC 支援**: 添加 Safari MP4 格式支援
3. **錯誤處理增強**: 更詳細的診斷信息

### 中期優化 (1月內)
1. **性能調優**: 解碼器緩存和復用
2. **格式檢測改進**: 更準確的魔術數字檢測
3. **監控指標**: 添加格式使用統計

### 長期規劃 (3月內)
1. **即時解碼**: 串流音頻支援
2. **多編碼器**: 支援更多音頻格式
3. **智能優化**: 基於瀏覽器的解碼策略

---

## 🔗 相關文檔

### 技術參考
- **[Opus 後端處理解決方案](../technical/OPUS_BACKEND_SOLUTION.md)** - 方案設計
- **[瀏覽器音頻錄製分析](../technical/BROWSER_AUDIO_RECORDING_ANALYSIS.md)** - 技術背景
- **[音頻處理架構](../technical/AUDIO_PROCESSING_ARCHITECTURE.md)** - 系統架構

### 實施支援
- **[環境配置指南](./environment-setup.md)** - 開發環境設置
- **[故障排除指南](../user-guide/troubleshooting.md)** - 問題解決
- **[系統架構](../technical/architecture.md)** - 整體架構

---

## 📝 實施記錄

**開始日期**: 2025-07-26  
**預計完成**: 2025-07-30  
**實施者**: [您的名稱]  
**技術審查**: [審查者]

### 實施檢查點
- [ ] **Day 1**: 依賴配置和基礎實現
- [ ] **Day 2**: 系統整合和初步測試  
- [ ] **Day 3**: 瀏覽器測試和優化
- [ ] **完成**: 所有測試通過，文檔更新

---

*本實施指南基於 [Opus 後端處理解決方案](../technical/OPUS_BACKEND_SOLUTION.md) 的業界標準設計，提供詳細的實施步驟和故障排除方案*