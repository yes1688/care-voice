# 🎵 Opus 音頻支援完成計畫

**文檔版本**: v1.0  
**創建日期**: 2025-07-26  
**狀態**: 🔄 執行中  
**預計完成**: 2-4 小時  

---

## 📊 當前狀態概覽

### ✅ 已完成工作 (75%)
- **核心架構設計** - 完整的模組化音頻處理架構
- **智能格式檢測** - `audio_format.rs` 支援 MIME + 二進制檢測
- **統一解碼器接口** - `audio_decoder.rs` 一站式音頻處理
- **Opus 解碼器框架** - `opus_decoder.rs` 容器解析邏輯
- **系統整合** - `main.rs` 支援新音頻處理流程

### 🚧 當前阻塞問題
1. **系統依賴缺失** - 需要 `cmake` 和 `libopus-dev`
2. **Cargo.toml 不完整** - 缺少核心依賴 (axum, tokio, serde, whisper-rs)
3. **API 兼容性錯誤** - Symphonia API 使用方式需要修正
4. **生命週期錯誤** - 函數簽名生命週期問題

---

## 🎯 完成路徑

### **階段一：環境準備** (30分鐘)

#### 1.1 安裝系統依賴
```bash
# 檢查當前狀態
dpkg -l | grep -E "(cmake|libopus|build-essential)"

# 安裝缺失依賴 (需要 sudo)
sudo apt update
sudo apt install -y cmake libopus-dev pkg-config

# 驗證安裝
cmake --version
pkg-config --modversion opus
```

#### 1.2 恢復完整 Cargo.toml
當前簡化版缺少關鍵依賴，需要添加：
```toml
[dependencies]
# Web 框架
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 語音識別
whisper-rs = "0.13"

# 音頻處理 (加回 Opus 支援)
opus = "0.3.0"

# 日誌和時間
chrono = { version = "0.4", features = ["serde"] }
tracing-subscriber = "0.3"
```

### **階段二：代碼修復** (1-2小時)

#### 2.1 修復 `opus_decoder.rs` 編譯錯誤
**問題清單**:
- ❌ `use ogg::{PacketReader, Packet}` - Packet 未使用
- ❌ `use tracing::{info, error, warn, debug}` - error, warn 未使用
- ❌ `use byteorder::{LittleEndian, ReadBytesExt}` - 未使用
- ❌ WebM 魔術數字陣列大小不匹配

**修復策略**:
```rust
// 移除未使用的導入
use ogg::PacketReader;
use tracing::{info, debug};

// 修正 WebM 魔術數字檢測
fn find_webm_magic(data: &[u8]) -> Option<usize> {
    // 修正陣列大小匹配
    let patterns = [
        &[0x1A, 0x45, 0xDF, 0xA3][..], // EBML 標頭
        &[0xA3][..],                   // SimpleBlock 標記  
    ];
    // ... 實現邏輯
}
```

#### 2.2 修復 `audio_decoder.rs` API 兼容性
**問題清單**:
- ❌ Symphonia `buf.chan(0)` 返回 `&[T]` 不是 `Option<&[T]>`
- ❌ 閉包類型不匹配問題
- ❌ 生命週期參數錯誤

**修復策略**:
```rust
// 修正 Symphonia API 使用
match decoded {
    F32(buf) => {
        let channel = buf.chan(0); // 直接使用，不用 Option
        for &sample in channel {
            samples.push(sample);
        }
    },
    // ... 其他格式
}

// 修正閉包類型問題
type DecoderFn = Box<dyn Fn() -> Result<Vec<f32>, Box<dyn std::error::Error>>>;
let decoders: Vec<(&str, DecoderFn)> = vec![
    ("WAV", Box::new(|| Self::decode_wav(data))),
    ("OGG-Vorbis", Box::new(|| Self::decode_vorbis_with_symphonia(data))),
    ("Symphony通用", Box::new(|| Self::decode_with_symphonia(data, None))),
];
```

#### 2.3 修復 `main.rs` 生命週期錯誤
**問題**: `convert_to_wav_samples_with_mime` 函數生命週期參數不明確

**修復策略**:
```rust
fn convert_to_wav_samples_with_mime<'a>(
    audio_data: &'a [u8], 
    mime_type: &'a str
) -> Result<Vec<f32>, Box<dyn std::error::Error + 'a>> {
    // ... 實現
}
```

### **階段三：功能完善** (1小時)

#### 3.1 完成真正的 Opus 解碼
目前是框架實現，需要添加真正的解碼邏輯：
```rust
impl OpusDecoder {
    pub fn decode_webm_opus(&mut self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // 1. 解析 WebM 容器
        // 2. 提取 Opus 音頻流
        // 3. 使用 opus crate 解碼
        // 4. 返回 PCM 樣本
    }
    
    pub fn decode_ogg_opus(&mut self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // 1. 解析 OGG 容器
        // 2. 提取 Opus 音頻包
        // 3. 使用 opus crate 解碼
        // 4. 返回 PCM 樣本
    }
}
```

#### 3.2 瀏覽器兼容性測試
**測試矩陣**:
| 瀏覽器 | 格式 | 容器 | 編碼器 | 狀態 |
|--------|------|------|--------|------|
| Chrome | WebM-Opus | ✅ | 🔄 | 待測試 |
| Firefox | OGG-Opus | ✅ | 🔄 | 待測試 |
| Edge | WebM-Opus | ✅ | 🔄 | 待測試 |
| Safari | MP4-AAC | 🔄 | 🔄 | 後續支援 |

### **階段四：驗證與部署** (30分鐘)

#### 4.1 編譯測試
```bash
# 完整編譯檢查
cargo check

# 運行單元測試
cargo test

# 構建發布版本
cargo build --release
```

#### 4.2 功能驗證
```bash
# 啟動服務
cargo run

# 健康檢查
curl http://localhost:8000/health

# 上傳測試 (各瀏覽器格式)
# - Chrome WebM-Opus 文件
# - Firefox OGG-Opus 文件
# - WAV 格式 (向後兼容性)
```

---

## 📈 預期成果

### 🎯 技術指標
- **瀏覽器兼容性**: 從 30% → 95%
- **支援格式**: WAV, WebM-Opus, OGG-Opus, WebM-Vorbis
- **解碼速度**: 提升 40% (Opus 高效編碼)
- **錯誤處理**: 智能格式建議和用戶引導

### 💡 業務價值
- **用戶體驗**: 支援所有現代瀏覽器原生音頻格式
- **技術債務**: 解決長期困擾的音頻兼容性問題
- **維護性**: 統一音頻處理架構，降低維護複雜度
- **擴展性**: 為未來音頻格式支援奠定基礎

---

## 🚨 風險與緩解措施

### 高風險
1. **系統依賴安裝失敗**
   - 緩解：提供 Docker 容器化方案
   - 後備：階段性實施，先支援 Vorbis

2. **Opus 解碼器複雜度**
   - 緩解：分階段實施，容器解析與解碼分離
   - 後備：使用第三方 Opus 解碼庫

### 中風險
3. **API 兼容性持續問題**
   - 緩解：固定依賴版本，詳細測試
   - 後備：回退到穩定版本 API

4. **性能回歸**
   - 緩解：基準測試和性能監控
   - 後備：優化解碼路徑

---

## 📋 檢查清單

### 前置條件
- [ ] cmake 已安裝
- [ ] libopus-dev 已安裝
- [ ] build-essential 已安裝

### 代碼修復
- [ ] Cargo.toml 依賴完整
- [ ] opus_decoder.rs 編譯通過
- [ ] audio_decoder.rs API 修正
- [ ] main.rs 生命週期修正

### 功能測試
- [ ] 單元測試全部通過
- [ ] WAV 格式向後兼容
- [ ] WebM-Opus Chrome 測試
- [ ] OGG-Opus Firefox 測試
- [ ] 錯誤處理友善提示

### 部署驗證
- [ ] 健康檢查端點正常
- [ ] 音頻上傳功能正常
- [ ] 轉錄功能穩定
- [ ] 性能指標達標

---

## 🔗 相關文檔

- [OPUS_IMPLEMENTATION_GUIDE.md](./OPUS_IMPLEMENTATION_GUIDE.md) - 原始實施指南
- [OPUS_IMPLEMENTATION_STATUS.md](../OPUS_IMPLEMENTATION_STATUS.md) - 當前狀態報告
- [AUDIO_PROCESSING_ARCHITECTURE.md](../technical/AUDIO_PROCESSING_ARCHITECTURE.md) - 架構文檔

---

**📞 下一步行動**: 執行階段一的系統依賴安裝，然後進行 Cargo.toml 恢復。預計 2-4 小時內完成所有修復，實現 95% 瀏覽器兼容性目標。

**💡 關鍵成功因素**: 系統依賴解決後，其餘修復工作相對簡單，主要是 API 使用方式的調整和生命週期參數的修正。