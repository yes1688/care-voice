# 🚀 Care Voice 部署指南

**文檔版本**: v2.0  
**最後更新**: 2025-07-25  
**適用版本**: whisper-rs 0.14.3 + CUDA 12.9.1

---

## 🎯 快速部署

### 核心部署命令

```bash
# 建構 CUDA 12.9.1 + Ubuntu 24.04 終極版本
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu-v2 .

# 運行 GPU 服務
podman run -d --name care-voice-ultimate --gpus all -p 8001:8001 \
  -v ./backend/models:/app/models:ro care-voice:whisper-rs-gpu-v2

# 驗證部署
curl http://localhost:8001/health

# GPU 診斷工具
podman exec care-voice-ultimate python3 /app/gpu_diagnostics.py
```

---

## 🏗️ 容器配置

### 容器映像規格
- **基礎映像**: Ubuntu 24.04 LTS + CUDA 12.9.1
- **映像大小**: ~950MB (GPU 版本)
- **whisper-rs**: 0.14.3 with CUDA features
- **記憶體優化**: 50% VRAM 節省

### 容器運行參數

```bash
# 完整配置運行
podman run -d \
  --name care-voice-ultimate \
  --gpus all \
  -p 8001:8001 \
  -p 3000:80 \
  -v ./backend/models:/app/models:ro \
  -v ./logs:/app/logs \
  -e RUST_LOG=info \
  -e CUDA_VISIBLE_DEVICES=0 \
  care-voice:whisper-rs-gpu-v2
```

### 環境變數
- `RUST_LOG`: 日誌級別 (debug, info, warn, error)
- `CUDA_VISIBLE_DEVICES`: 指定 GPU 設備
- `WHISPER_MODEL_PATH`: 模型檔案路徑

---

## 🎯 系統需求

### 硬體需求
- **GPU**: NVIDIA GTX 10xx+ 或 RTX 系列 (完整支援 RTX 50 系列)
- **記憶體**: 8GB+ 系統記憶體，4GB+ VRAM
- **儲存**: 10GB+ 可用空間 (含模型檔案)

### 軟體需求
- **作業系統**: Ubuntu 20.04+ (建議 24.04 LTS)
- **容器引擎**: Podman 4.0+ 或 Docker + NVIDIA Container Toolkit
- **CUDA**: 12.1+ (建議 12.9.1)
- **驅動程式**: NVIDIA 驅動 525+

---

## 📁 專案結構

```
care-voice/
├── docs/                        # 完整文檔系統
│   ├── development/             # 開發者文檔
│   ├── technical/               # 技術文檔  
│   └── user-guide/              # 用戶指南
├── Dockerfile.whisper-rs-gpu     # GPU 容器配置
├── backend/                     # Rust whisper-rs 後端
│   ├── src/main.rs             # 主程式
│   ├── models/                 # whisper 模型
│   └── Cargo.toml              # Rust 依賴
├── frontend/                    # Solid.js 前端
│   ├── src/                    # 源碼
│   └── dist/                   # 建構輸出
├── supervisord_whisper_rs.conf  # 進程管理
└── unified-nginx.conf           # Nginx 配置
```

---

## ⚡ 效能監控

### 容器狀態檢查
```bash
# 檢查容器狀態
podman ps
podman logs care-voice-ultimate

# 重啟服務
podman restart care-voice-ultimate
```

### GPU 使用率監控
```bash
# GPU 使用率
watch -n 1 'podman exec care-voice-ultimate nvidia-smi'

# 服務健康檢查
curl http://localhost:8001/health
```

### 效能指標
- **VRAM 使用**: ~3GB (vs 標準版 ~6GB)
- **啟動時間**: <30s (vs 標準版 ~60s)
- **轉錄速度**: Rust 原生性能
- **記憶體效率**: 50% 改善

---

## 🔗 相關文檔

- **環境設置**: [environment-setup.md](./environment-setup.md)
- **GPU 配置**: [../technical/gpu-configuration.md](../technical/gpu-configuration.md)
- **故障排除**: [../user-guide/troubleshooting.md](../user-guide/troubleshooting.md)
- **系統架構**: [../technical/architecture.md](../technical/architecture.md)

---

*本文檔由 Claude Code 協作維護*