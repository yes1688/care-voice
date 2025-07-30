# 📦 Care Voice 安裝指南

## 🎯 系統需求

### 硬體需求
- **CPU**: 現代 x64 處理器 (Intel/AMD)
- **記憶體**: 8GB+ 系統記憶體 (建議 16GB)
- **GPU**: NVIDIA GTX 10xx 或更新 (GPU 版本)
- **VRAM**: 4GB+ (GPU 版本)
- **存儲**: 10GB+ 可用空間

### 軟體需求
- **作業系統**: Linux (Ubuntu 20.04+ 推薦)
- **容器運行時**: Podman 4.0+ 或 Docker 20.10+
- **GPU 支援**: NVIDIA Container Toolkit (GPU 版本)
- **網路**: 網際網路連接 (下載模型和依賴)

## 🚀 快速安裝 (推薦)

### 1. 安裝 Podman
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install podman

# Fedora/RHEL
sudo dnf install podman

# 驗證安裝
podman --version
```

### 2. 安裝 NVIDIA Container Toolkit (GPU 版本)
```bash
# 添加 NVIDIA 軟體源
distribution=$(. /etc/os-release;echo $ID$VERSION_ID) \
   && curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add - \
   && curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list

# 安裝 NVIDIA Container Toolkit
sudo apt update
sudo apt install nvidia-container-toolkit

# 重啟容器服務
sudo systemctl restart docker  # 如使用 Docker
```

### 3. 下載 whisper 模型
```bash
# 建立模型目錄
mkdir -p models

# 下載基礎模型 (約 150MB)
curl -L -o models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# 或下載中型模型 (約 1.5GB，更好準確度)
curl -L -o models/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

### 4. 建構並運行服務
```bash
# 建構 GPU 容器
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu .

# 運行服務
podman run -d --name care-voice --gpus all -p 8001:8001 \
  -v ./models:/app/models:ro care-voice:whisper-rs-gpu

# 驗證服務
curl http://localhost:8001/health
```

## 🛠️ 手動安裝 (開發環境)

### 1. 安裝 Rust 開發環境
```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安裝系統依賴
sudo apt install build-essential cmake clang libclang-dev pkg-config
```

### 2. 安裝 Node.js (前端開發)
```bash
# 使用 NodeSource 軟體源
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install nodejs

# 驗證安裝
node --version
npm --version
```

### 3. 設置 CUDA 環境 (GPU 版本)
```bash
# 檢查 CUDA 版本
nvcc --version

# 設置環境變數
export CUDA_HOME=/usr/local/cuda
export PATH=$CUDA_HOME/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_HOME/lib64:$LD_LIBRARY_PATH
```

### 4. 編譯後端
```bash
cd backend

# 編譯 CPU 版本
cargo build --release

# 編譯 GPU 版本
cargo build --release --features cuda
```

### 5. 編譯前端
```bash
cd frontend

# 安裝依賴
npm install

# 建構生產版本
npm run build
```

## 🔧 環境配置

### GPU 配置檢查
```bash
# 檢查 GPU 狀態
nvidia-smi

# 檢查 CUDA 版本
nvcc --version

# 測試容器 GPU 訪問
podman run --rm --gpus all nvidia/cuda:12.1.1-base-ubuntu20.04 nvidia-smi
```

### 防火牆設置
```bash
# Ubuntu/Debian
sudo ufw allow 8001

# Fedora/RHEL
sudo firewall-cmd --permanent --add-port=8001/tcp
sudo firewall-cmd --reload
```

### 記憶體限制設置 (可選)
```bash
# 限制容器記憶體使用
podman run -d --name care-voice --gpus all -p 8001:8001 \
  --memory=8g --memory-swap=8g \
  -v ./models:/app/models:ro care-voice:whisper-rs-gpu
```

## 🧪 驗證安裝

### 1. 服務健康檢查
```bash
# 基本健康檢查
curl http://localhost:8001/health

# 預期回應
{
  "status": "healthy",
  "service": "Care Voice whisper-rs",
  "gpu_available": true,
  "model_loaded": true
}
```

### 2. 音頻轉錄測試
```bash
# 使用測試音頻文件
curl -X POST -F "audio=@test.wav" http://localhost:8001/transcribe

# 或使用瀏覽器訪問
firefox http://localhost:8001
```

### 3. GPU 使用監控
```bash
# 監控 GPU 使用率
watch -n 1 'podman exec care-voice nvidia-smi'

# 檢查容器資源使用
podman stats care-voice
```

## 🚨 常見安裝問題

### GPU 不可用
```bash
# 檢查 NVIDIA 驅動
nvidia-smi

# 重新安裝 Container Toolkit
sudo apt remove nvidia-container-toolkit
sudo apt install nvidia-container-toolkit
sudo systemctl restart docker
```

### 埠口被佔用
```bash
# 檢查埠口使用
lsof -i :8001

# 停止佔用進程
sudo kill -9 <PID>
```

### 模型下載失敗
```bash
# 手動下載模型
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin

# 檢查文件完整性
file models/ggml-base.bin
ls -la models/
```

### 容器建構失敗
```bash
# 清理 Podman 快取
podman system prune -a

# 重新建構
podman build --no-cache -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu .
```

## 📈 效能調優

### GPU 記憶體優化
```bash
# 限制 GPU 記憶體增長
export CUDA_MEMORY_POOL_DISABLED=1

# 設置 GPU 可見性
export CUDA_VISIBLE_DEVICES=0
```

### 容器資源限制
```bash
# 設置資源限制
podman run -d --name care-voice --gpus all \
  --cpus=4 --memory=8g \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

### 模型選擇建議
- **base 模型**: 快速轉錄，適合實時應用
- **medium 模型**: 平衡準確度和速度
- **large 模型**: 最高準確度，需要更多 VRAM

---

**安裝完成後**，建議閱讀 [快速開始指南](./quick-start.md) 了解基本使用方法。