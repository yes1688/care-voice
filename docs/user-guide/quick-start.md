# 🚀 Care Voice 快速開始指南

## 🎯 核心問題與解決方案

### 1. CUDA 映像問題
```bash
❌ 錯誤: manifest for nvidia/cuda:12.8-devel-ubuntu24.04 not found
✅ 解決: 使用完整版本號
FROM nvidia/cuda:12.1.1-devel-ubuntu20.04
```

### 2. CMake 版本問題
```bash
❌ 錯誤: CMake 3.18 or higher is required. You are running version 3.16.3
✅ 解決: 安裝 Kitware CMake
RUN wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc | apt-key add -
```

### 3. whisper-rs 編譯配置
```dockerfile
ENV WHISPER_DONT_GENERATE_BINDINGS=1
ENV CMAKE_CUDA_ARCHITECTURES="60;61;70;75;80;86;89;90;120"
RUN CFLAGS="-DGGML_CUDA=ON" cargo build --release --features cuda
```

## 🚀 一鍵建構命令

### GPU 加速版本 (推薦)

```bash
# 建構 whisper-rs GPU 容器
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu .

# 運行 GPU 加速服務
podman run -d --name care-voice-whisper-rs \
  --gpus all \
  -p 8001:8001 \
  -v ./models:/app/models:ro \
  care-voice:whisper-rs-gpu

# 驗證服務狀態
curl http://localhost:8001/health
```

### 標準版本 (相容性)

```bash
# 建構標準容器
podman-compose up --build -d

# 檢查服務狀態
podman-compose ps
curl http://localhost:3000  # 前端
curl http://localhost:8000/health  # 後端
```

## 🔧 前置需求

### GPU 加速版本
- NVIDIA GPU (GTX 10xx 或更新)
- NVIDIA Container Runtime
- Podman 4.0+
- 系統記憶體 8GB+ (建議)

### 標準版本
- Podman 4.0+
- podman-compose
- 4GB+ 系統記憶體

## 📋 故障排除檢查清單

### 容器建構問題
- [ ] CUDA 映像使用完整版本號
- [ ] CMake 版本 >= 3.18
- [ ] libclang 路徑配置正確
- [ ] CUDA 編譯標誌已設置

### GPU 執行問題
- [ ] GPU 運行時可用 (`nvidia-smi`)
- [ ] `--gpus all` 參數正確
- [ ] NVIDIA Container Toolkit 已安裝
- [ ] 足夠的 VRAM 可用 (>4GB)

### 服務問題
- [ ] 埠口未被佔用 (8001, 8000, 3000)
- [ ] 模型文件存在且可讀
- [ ] 網路連接正常
- [ ] 防火牆設置允許訪問

## 💡 使用方式

### GPU 加速版本
1. 🚀 **啟動服務**: 使用上述 GPU 建構命令
2. 🌐 **訪問界面**: http://localhost:8001
3. 🎤 **開始錄音**: 點擊錄音按鈕
4. ⚡ **GPU 轉錄**: whisper-rs CUDA 自動處理
5. 📊 **查看結果**: 實時顯示轉錄和摘要

### 標準版本
1. 🎤 **錄音**: 點擊「開始錄音」→ 說話 → 「停止錄音」
2. 📤 **上傳**: 點擊「轉換為文字」上傳音頻
3. 🤖 **AI 處理**: 後端轉錄分析
4. 📝 **結果**: 查看完整逐字稿和關懷重點摘要

## 📊 效能預期

| 指標 | whisper-rs GPU | 標準版 | 改善 |
|------|----------------|---------|------|
| **VRAM 使用** | ~3GB | ~6GB | -50% |
| **啟動時間** | <30s | ~60s | -50% |
| **轉錄速度** | Rust 原生 | Python 依賴 | 更快 |
| **記憶體效率** | 優化 | 標準 | 顯著改善 |

## 🚨 常見錯誤解決

### GPU 不可用
```bash
# 檢查 GPU 狀態
nvidia-smi

# 安裝 NVIDIA Container Toolkit
sudo apt install nvidia-container-toolkit
sudo systemctl restart docker
```

### 埠口衝突
```bash
# 檢查埠口使用
lsof -i :8001
sudo netstat -tulpn | grep 8001

# 停止衝突服務
podman stop $(podman ps -q)
```

### 模型載入失敗
```bash
# 檢查模型文件
ls -la models/
file models/ggml-base.bin

# 重新下載模型
mkdir -p models
curl -L -o models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

## 🔗 進階參考

- **[安裝指南](./installation.md)** - 詳細安裝步驟
- **[故障排除](./troubleshooting.md)** - 完整故障排除手冊
- **[系統架構](../technical/architecture.md)** - 技術實現細節
- **[GPU 配置](../technical/gpu-configuration.md)** - CUDA 優化設置

---

**最後更新**：2025-07-25  
**適用版本**：whisper-rs 0.14.3 + CUDA 12.1.1