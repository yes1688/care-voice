# 🛠️ WebM 音頻格式問題實施步驟指南

## 📋 實施概要

**實施目標**: 解決 Chrome/Edge 瀏覽器 WebM Opus 音頻格式轉換失敗問題  
**實施方案**: symphonia 依賴更新 + 錯誤處理改進  
**預計時間**: 2-3 小時  
**風險級別**: 🟢 低風險 (向後兼容)

---

## ✅ 實施前準備清單

### 環境準備
- [ ] 確認當前系統狀態正常
- [ ] 備份重要配置文件
- [ ] 準備測試音頻文件 (各格式)
- [ ] 記錄當前容器版本

### 工具準備
```bash
# 檢查必要工具
podman --version          # 容器管理
cargo --version           # Rust 編譯器
nvidia-smi               # GPU 狀態
curl                     # API 測試
```

### 備份準備
```bash
# 備份當前運行的容器
podman commit care-voice-ultimate care-voice:backup-before-webm-fix

# 備份配置文件
cp backend/Cargo.toml backend/Cargo.toml.backup
cp unified-nginx.conf unified-nginx.conf.backup
```

---

## 🔧 階段 1: 依賴配置更新

### 步驟 1.1: 更新 Cargo.toml

**檔案**: `backend/Cargo.toml`  
**修改內容**: 添加 opus 編解碼器支援

#### 原始配置
```toml
# 音頻處理 (用於格式轉換)
symphonia = { version = "0.5", features = [
    "mkv",          # WebM/Matroska 容器支援
    "vorbis"        # Vorbis 編解碼器 (Firefox/Chrome WebM)
] }
```

#### 更新後配置
```toml
# 音頻處理 (用於格式轉換) - 添加 Opus 支援
symphonia = { version = "0.5", features = [
    "mkv",          # WebM/Matroska 容器支援
    "vorbis",       # Vorbis 編解碼器 (Firefox WebM)
    "opus",         # Opus 編解碼器 (Chrome WebM) ← 新增
    "flac",         # FLAC 無損格式支援 (可選)
    "mp3"           # MP3 格式支援 (可選)
] }
```

#### 執行命令
```bash
# 編輯配置文件
vim backend/Cargo.toml

# 或使用 sed 自動更新
sed -i 's/"vorbis"/"vorbis", "opus", "flac", "mp3"/' backend/Cargo.toml
```

### 步驟 1.2: 驗證配置更新
```bash
# 檢查更新結果
grep -A5 symphonia backend/Cargo.toml

# 應該看到包含 opus 的配置
```

---

## 🏗️ 階段 2: 代碼改進

### 步驟 2.1: 改進錯誤處理

**檔案**: `backend/src/main.rs`  
**位置**: `try_decode_with_symphonia` 函數

#### 增強格式探測日誌
在 `main.rs` 第 279 行附近添加：

```rust
// 探測格式 - 增強錯誤信息
let probe = get_probe();
let probed = probe
    .format(&hint, media_source, &FormatOptions::default(), &MetadataOptions::default())
    .map_err(|e| {
        error!("格式探測失敗: {}", e);
        
        // 提供更詳細的錯誤信息
        let data_preview = if data.len() >= 16 {
            format!("{:02x?}", &data[..16])
        } else {
            format!("{:02x?}", data)
        };
        
        error!("音頻數據前16位元組: {}", data_preview);
        
        // 區分不同類型的格式錯誤
        match e {
            symphonia::core::errors::Error::IoError(ref io_err) 
                if io_err.kind() == std::io::ErrorKind::UnexpectedEof => {
                "音頻文件可能已完全解析，但缺少尾部信息".to_string()
            },
            symphonia::core::errors::Error::Unsupported(_) => {
                "不支援的音頻編解碼器，請確認已安裝所需的 symphonia 特性".to_string()
            },
            _ => format!("無法識別音頻格式: {}", e)
        }
    })?;
```

### 步驟 2.2: 添加格式統計

在 `main.rs` 文件開頭添加統計結構：

```rust
use std::sync::atomic::{AtomicU64, Ordering};

// 全域統計計數器
static WAV_COUNT: AtomicU64 = AtomicU64::new(0);
static WEBM_OPUS_COUNT: AtomicU64 = AtomicU64::new(0);
static WEBM_VORBIS_COUNT: AtomicU64 = AtomicU64::new(0);
static CONVERSION_SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static CONVERSION_FAILURE_COUNT: AtomicU64 = AtomicU64::new(0);
```

### 步驟 2.3: 改進用戶錯誤信息

在 `upload_audio` 函數中更新錯誤處理：

```rust
// 轉換音頻格式 (WebM/OGG -> WAV samples)
let audio_samples = convert_to_wav_samples(&data).map_err(|e| {
    error!("Audio conversion failed: {}", e);
    CONVERSION_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
    
    let user_message = if e.to_string().contains("opus") {
        "不支援 Chrome WebM Opus 格式。建議解決方案：\n1. 使用 Firefox 瀏覽器 (支援 Vorbis)\n2. 使用 Safari 瀏覽器 (支援 WAV)\n3. 等待系統更新以支援 Opus 格式"
    } else if e.to_string().contains("Unsupported") {
        "不支援的音頻格式。支援的格式：WAV, WebM (Vorbis)"
    } else {
        "音頻格式轉換失敗，請嘗試其他瀏覽器或音頻格式"
    };
    
    (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse { 
        error: user_message.to_string() 
    }))
})?;

// 轉換成功統計
CONVERSION_SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
```

---

## 🐳 階段 3: 容器重建

### 步驟 3.1: 清理編譯緩存
```bash
# 進入後端目錄
cd backend

# 清理 Cargo 緩存
cargo clean

# 返回根目錄
cd ..
```

### 步驟 3.2: 重建容器
```bash
# 停止當前容器
podman stop care-voice-ultimate
podman rm care-voice-ultimate

# 重建容器映像
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:webm-opus-support .

# 檢查建構結果
echo "建構完成，容器標籤: care-voice:webm-opus-support"
```

### 步驟 3.3: 啟動新容器
```bash
# 啟動更新後的容器
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
  care-voice:webm-opus-support

# 等待啟動
sleep 20
```

---

## 🧪 階段 4: 功能測試

### 步驟 4.1: 基本健康檢查
```bash
# 檢查服務狀態
curl -s http://localhost:8001/health

# 預期結果: {"service":"Care Voice with whisper-rs","status":"healthy","version":"1.0.0"}
```

### 步驟 4.2: 容器日誌檢查
```bash
# 檢查 whisper-rs 服務日誌
podman logs --tail 20 care-voice-ultimate

# 檢查是否有啟動錯誤
podman exec care-voice-ultimate cat /var/log/supervisor/whisper-rs.log | tail -10
```

### 步驟 4.3: symphonia 功能驗證
```bash
# 進入容器檢查 symphonia 支援
podman exec care-voice-ultimate /app/care-voice --help 2>&1 | grep -i opus

# 檢查編解碼器載入
podman exec care-voice-ultimate ldd /app/care-voice | grep -E "(opus|vorbis)"
```

### 步驟 4.4: 前端錄音測試

#### 手動測試步驟
1. **開啟瀏覽器**: 訪問 http://localhost:8001
2. **Chrome 測試**:
   - 點擊 "🎤 開始錄音"
   - 錄製 5-10 秒音頻
   - 點擊 "⏹️ 停止錄音"
   - 點擊 "📤 轉換為文字"
   - ✅ 預期: 成功轉錄，無 422 錯誤

3. **Firefox 測試**:
   - 重複上述步驟
   - ✅ 預期: 繼續正常工作

4. **Safari 測試** (如有):
   - 重複上述步驟
   - ✅ 預期: 繼續正常工作

#### 自動化測試 (可選)
```bash
# 使用測試音頻文件 (如果有)
if [ -f "test-audio.webm" ]; then
    curl -X POST "http://localhost:8001/api/upload" \
      -F "audio=@test-audio.webm" \
      -H "accept: application/json"
fi
```

---

## 📊 階段 5: 監控和驗證

### 步驟 5.1: 錯誤率監控
```bash
# 檢查錯誤日誌
podman exec care-voice-ultimate grep -c "Audio conversion failed" /var/log/supervisor/whisper-rs.log

# 如果大於 0，需要進一步調查
```

### 步驟 5.2: 格式支援驗證
```bash
# 檢查各格式轉換日誌
podman exec care-voice-ultimate grep -E "(Vorbis|Opus|WAV)" /var/log/supervisor/whisper-rs.log | tail -10
```

### 步驟 5.3: 性能基準測試
```bash
# GPU 使用率檢查
nvidia-smi

# 轉錄速度測試 (錄音+轉錄總時間)
time (echo "測試開始" && curl -s http://localhost:8001/health)
```

---

## 🛠️ 階段 6: 問題排除

### 常見問題和解決方案

#### 問題 1: 編譯失敗
```bash
# 症狀: cargo build 失敗
# 解決: 檢查 Rust 版本和依賴
rustc --version  # 確保 >= 1.70
cargo update     # 更新依賴版本
```

#### 問題 2: 容器啟動失敗
```bash
# 症狀: 容器立即退出
# 解決: 檢查依賴映射
podman logs care-voice-ultimate
podman exec care-voice-ultimate ldd /app/care-voice
```

#### 問題 3: 仍然有 Opus 轉換錯誤
```bash
# 症狀: Chrome WebM 仍然失敗
# 解決: 驗證 symphonia 特性
podman exec care-voice-ultimate find /usr/local/cargo -name "*opus*"
```

#### 問題 4: 性能下降
```bash
# 症狀: 轉錄時間增加
# 解決: 檢查 GPU 訪問
podman exec care-voice-ultimate nvidia-smi
```

---

## 🔄 回退程序

### 緊急回退 (如果嚴重問題)
```bash
# 停止問題容器
podman stop care-voice-ultimate
podman rm care-voice-ultimate

# 啟動備份容器
podman run -d \
  --name care-voice-ultimate \
  --device /dev/nvidia0 \
  --device /dev/nvidiactl \
  --device /dev/nvidia-uvm \
  -p 8001:8001 \
  -v ./backend/models:/app/models:ro \
  care-voice:backup-before-webm-fix

# 恢復配置文件
cp backend/Cargo.toml.backup backend/Cargo.toml
```

### 部分回退 (如果部分問題)
```bash
# 僅回退 symphonia 配置
sed -i 's/"vorbis", "opus", "flac", "mp3"/"vorbis"/' backend/Cargo.toml

# 重新編譯
cargo build --release --features gpu
```

---

## ✅ 完成清單

### 實施完成檢查
- [ ] Cargo.toml 更新完成
- [ ] 代碼改進完成
- [ ] 容器重建成功
- [ ] 基本功能測試通過
- [ ] 各瀏覽器錄音測試通過
- [ ] 錯誤處理改進驗證
- [ ] 性能無明顯降級
- [ ] 日誌監控正常

### 文檔更新檢查
- [ ] 實施記錄完整
- [ ] 問題解決方案文檔化
- [ ] 回退程序測試
- [ ] 監控指標建立

### 後續維護準備
- [ ] 錯誤監控告警設置
- [ ] 定期健康檢查計劃
- [ ] 用戶使用情況統計
- [ ] 格式支援覆蓋率跟蹤

---

## 📈 成功指標

### 技術指標
- ✅ Chrome WebM Opus 轉換成功率 > 95%
- ✅ Firefox WebM Vorbis 維持 100% 成功率
- ✅ Safari WAV 維持 100% 成功率
- ✅ 總體錯誤率 < 2%

### 用戶體驗指標
- ✅ 所有主流瀏覽器錄音功能正常
- ✅ 錯誤信息用戶友好
- ✅ 轉錄時間無明顯增加

### 維護指標
- ✅ 實施時間 < 3 小時
- ✅ 零停機時間部署
- ✅ 完整的問題記錄和解決方案

---

*本實施指南建立於 2025-07-26，提供 Care Voice WebM 音頻格式問題的詳細解決步驟*