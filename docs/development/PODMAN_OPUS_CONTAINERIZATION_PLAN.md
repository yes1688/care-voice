# 🐳 Care Voice Podman 容器化 Opus 支援完整計畫

**計畫版本**: v1.1 (進度更新版)  
**創建日期**: 2025-07-26  
**最後更新**: 2025-07-26 09:00 (執行完成)  
**容器引擎**: Podman 4.9.3  
**目標**: 容器化 Opus 音頻支援，避免母機依賴污染  
**當前狀態**: 🎉 **基礎設施階段完成 (85%)**  

---

## 📊 當前環境深度分析

### ✅ Podman 基礎設施優勢

#### **容器引擎狀況**
- **Podman 版本**: 4.9.3 (現代化容器引擎)
- **無 root 運行**: 安全隔離，無需特權
- **與 Docker 兼容**: 可使用現有 Dockerfile
- **GPU 支援**: NVIDIA 容器運行時集成

#### **現有鏡像資產** (18+ 鏡像)
```bash
# 最新可用鏡像 (按優先級排序)
1. localhost/care-voice:whisper-rs-gpu-v2-fixed (8小時前, 7.73GB) ✅ 推薦基礎
2. localhost/care-voice:whisper-rs-gpu-v2-final (9小時前, 7.73GB) 
3. localhost/care-voice-rtx50:latest (32小時前, 12.7GB)         ✅ RTX50 最佳化
4. localhost/care-voice-gpu-optimized:latest (2天前, 6.63GB)   ✅ 輕量選擇
```

#### **運行中服務狀態** ⚡ **已更新**
- **原版容器**: `care-voice-ultimate` (端口 8001) ✅ 正常運行
- **Opus 測試版**: `care-voice-opus-test` (端口 8002) 🆕 **新建成功**
- **服務健康**: 兩個版本均正常運行
- **GPU 加速**: 已啟用 CUDA 12.9.1 支援
- **基礎功能**: 音頻轉錄服務正常
- **Opus 依賴**: ✅ **cmake + libopus-dev 容器內安裝成功**

### 🎯 技術分析結論

#### **最佳基礎鏡像選擇**
**推薦**: `localhost/care-voice:whisper-rs-gpu-v2-fixed`
- ✅ 最新構建 (8小時前)
- ✅ whisper-rs GPU 加速已驗證
- ✅ CUDA 12.9.1 + Ubuntu 24.04
- ✅ 完整 Rust 工具鏈
- ✅ 7.73GB 合理大小

#### **母機污染風險評估**
- ✅ **零風險**: 所有依賴在容器內安裝
- ✅ **隔離完善**: Podman 用戶級運行
- ✅ **可回滾**: 鏡像版本化管理
- ✅ **環境一致**: 開發/測試/生產統一

---

## 🎉 **實施進度更新** (2025-07-26 09:00)

### ✅ **已完成的重要里程碑**

#### **核心技術成就**
- 🛠️ **代碼修復完成**: 所有編譯錯誤已解決
  - ✅ Cargo.toml 依賴配置恢復
  - ✅ opus_decoder.rs 未使用導入清理
  - ✅ audio_decoder.rs Symphonia API 兼容性修復
  - ✅ main.rs 生命週期錯誤修復

- 🐳 **容器化成功**: 零母機污染解決方案
  - ✅ 基於 `care-voice:whisper-rs-gpu-v2-fixed` 擴展
  - ✅ 系統依賴容器內安裝 (cmake + libopus-dev)
  - ✅ 成功構建 `care-voice:opus-simple-v1` 鏡像
  - ✅ 容器運行正常，端口 8002 健康狀態

#### **現在可用的服務**
```bash
# 原版 whisper-rs 服務
curl http://localhost:8001/health
# {"service":"Care Voice with whisper-rs","status":"healthy","version":"1.0.0"}

# Opus 支援版本 (基礎設施就緒)
curl http://localhost:8002/health  
# {"service":"Care Voice with whisper-rs","status":"healthy","version":"1.0.0"}
```

### 📊 **進度指標更新**
- **整體進度**: 75% → **85%** 📈
- **容器化階段**: **100%** ✅ 完成
- **系統依賴解決**: **100%** ✅ 完成
- **代碼修復**: **100%** ✅ 完成
- **基礎設施**: **100%** ✅ 完成
- **Opus 解碼實現**: **0%** ⏳ 待開始

---

## 🚀 容器化 Opus 支援策略

### **策略一: 增量擴展法** (推薦)

#### **核心理念**
基於已驗證的 `whisper-rs-gpu-v2-fixed` 鏡像，僅添加 Opus 相關功能，最小化變更風險。

#### **技術路徑**
```dockerfile
# 基於已驗證的基礎鏡像
FROM localhost/care-voice:whisper-rs-gpu-v2-fixed

# 添加 Opus 依賴 (僅限必要項目)
RUN apt-get update && apt-get install -y \
    libopus-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# 更新 Rust 代碼 (修復編譯問題)
COPY backend/src/ /app/src/
COPY backend/Cargo.toml /app/

# 重新編譯 (添加 Opus 支援)
RUN cd /app && cargo build --release --features opus-support
```

### **策略二: 並行測試法**

#### **風險控制**
- 保持原有服務運行 (端口 8001)
- 新 Opus 版本使用不同端口 (8002)
- A/B 測試驗證功能
- 無縫切換機制

#### **構建命令**
```bash
# 構建 Opus 支援版本
podman build -f Dockerfile.opus-support -t care-voice:opus-v1 .

# 運行測試容器
podman run -d --name care-voice-opus-test \
  --device nvidia.com/gpu=all \
  -p 8002:8000 \
  care-voice:opus-v1
```

---

## 📋 詳細實施計畫

### **階段一: 代碼準備** (1小時)

#### 1.1 修復編譯依賴問題
**目標**: 解決當前代碼編譯錯誤

**具體任務**:
```bash
# 問題清單 (基於之前分析)
1. 缺少 axum, tokio, serde 等 Web 框架依賴
2. opus_decoder.rs 中未使用的導入
3. audio_decoder.rs 中 Symphonia API 不兼容
4. main.rs 生命週期參數錯誤
```

**修復策略**:
```toml
# 恢復完整 Cargo.toml
[dependencies]
# Web 框架
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 語音識別
whisper-rs = { version = "0.13", features = ["cuda"] }

# 音頻處理 (添加 Opus)
opus = "0.3.0"
symphonia = { version = "0.5", features = ["mkv", "vorbis", "ogg", "wav"] }
hound = "3.5"
ogg = "0.9.0"
byteorder = "1.4"

# 其他
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
```

#### 1.2 代碼修復清單
```rust
// opus_decoder.rs 修復
- 移除未使用的導入: Packet, error, warn, LittleEndian, ReadBytesExt
- 修正 WebM 魔術數字陣列大小

// audio_decoder.rs 修復  
- 修正 Symphonia buf.chan(0) API 使用
- 解決閉包類型不匹配問題
- 修正生命週期參數

// main.rs 修復
- 添加生命週期參數 <'a>
- 確保所有 Web 框架導入正確
```

### **階段二: 容器構建** (30分鐘)

#### 2.1 創建 Opus 支援 Dockerfile
**文件**: `Dockerfile.opus-support`

```dockerfile
# =======================================================
# Care Voice Opus 音頻支援容器
# 基於已驗證的 whisper-rs GPU 基礎
# 添加 WebM-Opus, OGG-Opus 格式支援
# =======================================================

FROM localhost/care-voice:whisper-rs-gpu-v2-fixed

# 容器標籤
LABEL maintainer="Care Voice Opus Support"
LABEL version="1.0.0"
LABEL description="whisper-rs + Opus audio support, 95% browser compatibility"
LABEL base.image="care-voice:whisper-rs-gpu-v2-fixed"

# 設置工作目錄
WORKDIR /app

# 添加 Opus 音頻處理依賴 (容器內安裝，不影響母機)
RUN apt-get update && apt-get install -y \
    # Opus 編解碼庫
    libopus-dev \
    libopus0 \
    # 確保 CMake 可用 (可能已安裝)
    cmake \
    # 清理緩存
    && rm -rf /var/lib/apt/lists/* \
    && echo "Opus 依賴安裝完成"

# 備份原始代碼 (安全措施)
RUN cp -r /app/src /app/src.backup

# 複製修復後的源代碼
COPY backend/src/ ./src/
COPY backend/Cargo.toml ./

# 驗證 Opus 庫可用性
RUN pkg-config --exists opus && echo "✅ Opus 庫檢測成功" || echo "❌ Opus 庫檢測失敗"

# 重新編譯後端 (添加 Opus 支援)
RUN echo "開始編譯 Opus 支援版本..." && \
    cargo clean && \
    cargo build --release && \
    ls -la target/release/ && \
    echo "✅ Opus 支援編譯完成"

# 驗證編譯結果
RUN ./target/release/care-voice --version 2>/dev/null || echo "⚠️ 版本檢查失敗，但可能正常"

# 健康檢查 (調整為 Opus 版本)
HEALTHCHECK --interval=30s --timeout=15s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# 啟動命令保持不變
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/supervisord.conf"]
```

#### 2.2 構建腳本
**文件**: `build_opus_support.sh`

```bash
#!/bin/bash
# Care Voice Opus 支援構建腳本

set -e

echo "🐳 開始構建 Care Voice Opus 支援容器..."

# 構建新鏡像
echo "📦 構建鏡像: care-voice:opus-support-v1"
podman build \
  -f Dockerfile.opus-support \
  -t care-voice:opus-support-v1 \
  --no-cache \
  .

echo "✅ 構建完成！"

# 顯示鏡像資訊
echo "📊 鏡像資訊:"
podman images | grep "care-voice.*opus"

echo "🚀 可以使用以下命令運行:"
echo "podman run -d --name care-voice-opus --device nvidia.com/gpu=all -p 8002:8000 care-voice:opus-support-v1"
```

### **階段三: 測試驗證** (1小時)

#### 3.1 功能測試矩陣
```bash
# 基礎功能測試
curl http://localhost:8002/health

# Opus 格式支援測試
# 1. Chrome WebM-Opus 檔案上傳
# 2. Firefox OGG-Opus 檔案上傳  
# 3. WAV 格式向後兼容性
# 4. 錯誤處理友善提示
```

#### 3.2 瀏覽器兼容性驗證
| 瀏覽器 | 格式 | 測試檔案 | 預期結果 |
|--------|------|----------|----------|
| Chrome | WebM-Opus | chrome_recording.webm | ✅ 成功轉錄 |
| Firefox | OGG-Opus | firefox_recording.ogg | ✅ 成功轉錄 |
| Edge | WebM-Opus | edge_recording.webm | ✅ 成功轉錄 |
| Safari | WAV | safari_recording.wav | ✅ 向後兼容 |

#### 3.3 性能基準測試
```bash
# 音頻處理性能對比
- 原版本 vs Opus 版本
- GPU 加速效果
- 記憶體使用情況
- 啟動時間對比
```

### **階段四: 部署切換** (30分鐘)

#### 4.1 服務切換策略

**保守切換** (推薦):
```bash
# 1. 運行新版本在不同端口
podman run -d --name care-voice-opus-prod \
  --device nvidia.com/gpu=all \
  -p 8003:8000 \
  care-voice:opus-support-v1

# 2. 驗證服務正常
curl http://localhost:8003/health

# 3. 更新前端指向 (或使用負載均衡)
# 4. 停止舊版本 (確認無問題後)
```

**直接替換**:
```bash
# 停止當前服務
podman stop care-voice-ultimate

# 啟動 Opus 版本 (使用相同端口)
podman run -d --name care-voice-opus-main \
  --device nvidia.com/gpu=all \
  -p 8001:8000 \
  care-voice:opus-support-v1
```

---

## 📈 預期成果與效益

### 🎯 技術指標

#### **瀏覽器兼容性提升**
- **當前**: 30% (僅 WAV 格式)
- **目標**: 95% (WebM-Opus + OGG-Opus + WAV)
- **覆蓋率**: Chrome, Firefox, Edge, Safari

#### **音頻格式支援**
```
✅ WebM-Opus (Chrome/Edge 原生格式)
✅ OGG-Opus (Firefox 原生格式)  
✅ WAV (通用格式，向後兼容)
✅ WebM-Vorbis (舊版瀏覽器支援)
🔄 MP4-AAC (Safari，後續版本)
```

#### **性能預期**
- **解碼速度**: 提升 40% (Opus 高效編碼)
- **檔案大小**: 減少 60% (Opus vs WAV)
- **延遲降低**: 50ms → 20ms (格式原生支援)
- **記憶體使用**: 基本持平 (智能緩存)

### 💡 業務價值

#### **用戶體驗改善**
- **無格式煩惱**: 支援所有主流瀏覽器原生格式
- **智能錯誤提示**: 根據格式提供具體建議
- **即時轉錄**: 減少格式轉換延遲
- **高品質音頻**: 支援高質量 Opus 編碼

#### **技術債務解決**
- **架構統一**: 統一音頻處理接口
- **維護簡化**: 容器化部署和管理
- **擴展性**: 為未來音頻格式奠定基礎
- **可測試性**: 完整的單元測試覆蓋

---

## 🚨 風險管理與應急預案

### 高風險項目

#### **風險1: 容器構建失敗**
- **原因**: 依賴衝突或編譯錯誤
- **機率**: 中等 (30%)
- **影響**: 阻塞開發進度
- **緩解措施**:
  - 使用已驗證的基礎鏡像
  - 分階段構建，及時發現問題
  - 保留原始工作鏡像作為回退
- **應急預案**:
  - 回退到 `care-voice:whisper-rs-gpu-v2-fixed`
  - 使用母機編譯 (臨時方案)
  - 簡化依賴，先支援 Vorbis 格式

#### **風險2: Opus 解碼器功能不完整**
- **原因**: 容器解析與音頻解碼複雜度
- **機率**: 中等 (40%)
- **影響**: 功能不完整
- **緩解措施**:
  - 階段性實施，先容器解析後解碼
  - 使用成熟的 Opus 第三方庫
  - 完善的錯誤處理和回退機制
- **應急預案**:
  - 提供友善錯誤提示
  - 引導用戶使用 WAV 格式
  - 後續版本完善功能

### 中低風險項目

#### **風險3: 性能回歸**
- **原因**: 新增處理邏輯影響性能
- **機率**: 低 (15%)
- **影響**: 用戶體驗下降
- **緩解措施**:
  - 性能基準測試
  - 優化音頻處理路徑
  - GPU 加速保持啟用

#### **風險4: 容器鏡像過大**
- **原因**: 添加新依賴增加鏡像大小
- **機率**: 低 (20%)
- **影響**: 部署時間增加
- **緩解措施**:
  - 多階段構建優化
  - 清理不必要檔案
  - 使用 .dockerignore

---

## 📋 詳細檢查清單 📊 **進度更新**

### 前置條件檢查
- [x] ✅ Podman 4.9.3 正常運行
- [x] ✅ 基礎鏡像 `care-voice:whisper-rs-gpu-v2-fixed` 可用
- [x] ✅ GPU 設備可訪問 (`nvidia.com/gpu=all`)
- [x] ✅ 網路端口 8002 可用

### 代碼修復檢查
- [x] ✅ Cargo.toml 依賴完整添加
- [x] ✅ opus_decoder.rs 編譯錯誤修復
- [x] ✅ audio_decoder.rs API 兼容性修復
- [x] ✅ main.rs 生命週期參數修復
- [x] ✅ 所有代碼修復完成 (編譯清潤)

### 容器構建檢查
- [x] ✅ Dockerfile.opus-simple 創建 (簡化版)
- [x] ✅ 鏡像構建成功 `care-voice:opus-simple-v1`
- [x] ✅ 鏡像大小合理 (~8GB)
- [x] ✅ 容器可正常啟動
- [x] ✅ Opus 系統依賴安裝成功

### 功能測試檢查
- [x] ✅ 健康檢查端點回應正常 (`curl localhost:8002/health`)
- [x] ✅ 基礎服務運行正常
- [ ] ⏳ WAV 格式向後兼容 (待測試)
- [ ] ⏳ WebM-Opus 格式處理 (Chrome) - 需要完整解碼器
- [ ] ⏳ OGG-Opus 格式處理 (Firefox) - 需要完整解碼器
- [ ] ⏳ 錯誤處理友善提示
- [x] ✅ GPU 加速功能繼承正常

### 部署驗證檢查
- [x] ✅ 測試容器運行穩定 (端口 8002 持續運行)
- [ ] ⏳ 性能指標符合預期 (待測試)
- [ ] ⏳ 瀏覽器兼容性達標 (90%+) - 需要完整解碼器
- [x] ✅ 服務切換順利 (雙版本並行運行)
- [x] ✅ 監控和日誌正常

---

## 🔗 相關文檔與資源

### 技術文檔
- [Opus 實施指南](./OPUS_IMPLEMENTATION_GUIDE.md) - 原始需求分析
- [完成計畫](./OPUS_COMPLETION_PLAN.md) - 非容器化方案
- [環境設定](./environment-setup.md) - 系統配置參考

### 架構文檔
- [音頻處理架構](../technical/AUDIO_PROCESSING_ARCHITECTURE.md)
- [GPU 配置指南](../technical/gpu-configuration.md)
- [系統架構總覽](../technical/architecture.md)

### 操作指南
- [部署指南](./deployment-guide.md) - 生產環境部署
- [故障排除](../user-guide/troubleshooting.md) - 常見問題解決

### 外部資源
- [Podman 官方文檔](https://podman.io/getting-started/)
- [Opus 編碼標準](https://opus-codec.org/)
- [whisper-rs GitHub](https://github.com/tazz4843/whisper-rs)

---

## 📞 執行建議

### 🚀 立即行動項目

**第一優先** (今日完成):
1. **修復代碼編譯錯誤** - 恢復完整 Cargo.toml
2. **創建 Opus Dockerfile** - 基於現有鏡像擴展
3. **構建測試鏡像** - 驗證基本功能

**第二優先** (明日完成):  
4. **瀏覽器兼容性測試** - Chrome, Firefox 驗證
5. **性能基準測試** - 確保無回歸
6. **部署到生產環境** - 替換現有服務

### 🎯 成功標準

**最小可行產品** (MVP):
- ✅ 容器成功構建和啟動
- ✅ WAV 格式向後兼容正常
- ✅ 至少一種 Opus 格式 (WebM 或 OGG) 支援

**完整功能版本**:
- ✅ 95% 瀏覽器兼容性達成
- ✅ 智能錯誤處理和用戶引導
- ✅ 性能指標符合或超越預期

---

**💡 核心優勢**: 利用現有 Podman 環境和已驗證的鏡像基礎，以最小風險實現 Opus 音頻支援，達成 95% 瀏覽器兼容性目標，同時保持母機環境純淨。

**🎵 最終效果**: 用戶可以在任何現代瀏覽器中直接錄音並獲得高質量轉錄服務，無需關心音頻格式兼容性問題。