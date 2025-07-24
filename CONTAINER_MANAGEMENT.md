# Care Voice 容器版本管理指南

## 🎯 多容器環境概覽

Care Voice 項目目前運行多個容器，提供不同的功能和兼容性支援。

### 當前容器狀態 ✅

| 容器名稱 | 端口 | 狀態 | 主要功能 | GPU 支援 |
|----------|------|------|----------|----------|
| **care-voice-rtx50** | **8001** | **🟢 運行中** | **RTX 50 系列 GPU 加速 Whisper** | **RTX 5070 Ti (CDI)** |
| care-voice-gpu | 8000 | 🟡 舊版本 | 舊版 GPU 加速 | 傳統 GPU 存取 |
| care-voice-cpu | 3001 | 🟡 舊版本 | CPU 回退版本 | 不支援 |
| care-voice-unified | 3002 | 🔴 停用 | 實驗性統一服務 | 不支援 |

## 🚀 推薦使用

### RTX 50 系列主要容器 (推薦) ⭐⭐⭐⭐⭐

```bash
# 啟動 RTX 50 系列 GPU 加速容器 (推薦)
podman run -d --name care-voice-rtx50 \
    --device nvidia.com/gpu=all \
    -p 8001:8001 \
    -e NVIDIA_VISIBLE_DEVICES=all \
    care-voice-rtx50:latest

# 訪問服務
curl http://localhost:8001/health
firefox http://localhost:8001
```

**優勢**:
- ✅ RTX 5070 Ti GPU 加速 (31,250 GFLOPS)
- ✅ 混合精度 FP16 推理優化
- ✅ 多世代 GPU 兼容 (RTX 50/40/30/20 + GTX 10)
- ✅ CDI GPU 存取技術
- ✅ 最新 CUDA 12.8 + PyTorch nightly cu128

## 📋 容器管理操作

### 檢查所有容器狀態

```bash
# 查看所有 Care Voice 容器
podman ps -a | grep care-voice

# 檢查 RTX 50 系列容器詳細狀態
podman inspect care-voice-rtx50 | grep -E "State|Status|ExitCode"

# 查看容器資源使用
podman stats care-voice-rtx50
```

### 容器日誌管理

```bash
# RTX 50 系列容器日誌
podman logs care-voice-rtx50

# GPU 診斷日誌
podman exec care-voice-rtx50 cat /app/logs/gpu_diagnostics_report.json

# Whisper 服務日誌
podman exec care-voice-rtx50 cat /app/logs/rtx50_whisper_service.log

# 系統服務狀態
podman exec care-voice-rtx50 supervisorctl status
```

### 容器健康檢查

```bash
# RTX 50 系列健康檢查
curl http://localhost:8001/health

# GPU 狀態檢查
podman exec care-voice-rtx50 nvidia-smi

# 完整 GPU 診斷
podman exec care-voice-rtx50 python3 /app/gpu_diagnostics.py
```

## 🔄 版本升級策略

### 從舊版本升級到 RTX 50 系列

```bash
# 1. 備份舊容器數據 (可選)
podman cp care-voice-gpu:/app/logs ./backup-logs-$(date +%Y%m%d) 2>/dev/null || true

# 2. 優雅停止舊容器
podman stop care-voice-gpu care-voice-cpu care-voice-unified 2>/dev/null || true

# 3. 部署 RTX 50 系列新容器
podman run -d --name care-voice-rtx50 \
    --device nvidia.com/gpu=all \
    -p 8001:8001 \
    care-voice-rtx50:latest

# 4. 驗證升級成功
curl http://localhost:8001/health
podman exec care-voice-rtx50 python3 /app/gpu_diagnostics.py

# 5. 清理舊容器 (確認新容器正常後執行)
podman rm care-voice-gpu care-voice-cpu care-voice-unified 2>/dev/null || true
```

### 容器版本回退 (緊急情況)

```bash
# 如果 RTX 50 系列出現問題，快速回退到 GPU 版本
podman stop care-voice-rtx50
podman run -d --name care-voice-gpu-fallback \
    --gpus all -p 8000:8000 \
    care-voice-legacy:latest

# 訪問回退服務
curl http://localhost:8000/health
```

## 🛠️ 故障排除

### RTX 50 系列容器問題

```bash
# 檢查容器是否正常啟動
podman ps | grep care-voice-rtx50

# 檢查 GPU 可見性
podman exec care-voice-rtx50 nvidia-smi

# 檢查 CDI GPU 設備
nvidia-ctk cdi list | grep nvidia.com/gpu

# 重啟容器服務
podman restart care-voice-rtx50
```

### 端口衝突解決

```bash
# 檢查端口佔用
netstat -tlnp | grep -E "800[01]|300[12]"

# 停止衝突的容器
podman stop $(podman ps -q --filter "publish=8001")

# 使用不同端口啟動
podman run -d --name care-voice-rtx50-alt \
    --device nvidia.com/gpu=all \
    -p 8002:8001 \
    care-voice-rtx50:latest
```

### 資源清理

```bash
# 清理所有停止的 Care Voice 容器
podman container prune --filter "label=project=care-voice"

# 清理未使用的映像
podman image prune -f

# 查看磁碟使用情況
podman system df
```

## 📊 容器性能監控

### 實時監控

```bash
# RTX 50 系列容器資源使用
watch -n 1 podman stats care-voice-rtx50

# GPU 使用率監控
podman exec care-voice-rtx50 watch -n 1 nvidia-smi

# 系統整體狀況
htop
```

### 性能指標收集

```bash
# 容器 CPU/記憶體使用
podman exec care-voice-rtx50 ps aux | head -10

# GPU 記憶體使用
podman exec care-voice-rtx50 python3 -c "import torch; print(f'VRAM: {torch.cuda.memory_allocated()/1024**3:.2f}GB')"

# 磁碟 I/O 狀況
podman exec care-voice-rtx50 iostat -x 1 3
```

## 📁 容器文件管理

### 重要配置文件位置

```bash
# RTX 50 系列容器內部結構
podman exec care-voice-rtx50 find /app -type f -name "*.py" | head -10
podman exec care-voice-rtx50 ls -la /app/logs/

# 配置文件備份
podman cp care-voice-rtx50:/etc/supervisor/supervisord.conf ./supervisor-backup.conf
podman cp care-voice-rtx50:/etc/nginx/nginx.conf ./nginx-backup.conf
```

### 動態配置更新

```bash
# 更新 Supervisor 配置
podman cp new-supervisord.conf care-voice-rtx50:/etc/supervisor/supervisord.conf
podman exec care-voice-rtx50 supervisorctl reread
podman exec care-voice-rtx50 supervisorctl reload

# 重載 Nginx 配置
podman cp new-nginx.conf care-voice-rtx50:/etc/nginx/nginx.conf
podman exec care-voice-rtx50 nginx -s reload
```

## 🔒 安全和維護

### 定期維護任務

```bash
# 每週容器健康檢查
curl -f http://localhost:8001/health || echo "健康檢查失敗"

# 每月日誌輪轉
podman exec care-voice-rtx50 logrotate /etc/logrotate.conf

# 季度性能基準測試
podman exec care-voice-rtx50 python3 /app/gpu_diagnostics.py
```

### 備份策略

```bash
# 完整容器映像備份
podman commit care-voice-rtx50 care-voice-rtx50-backup:$(date +%Y%m%d)

# 配置和日誌備份
podman cp care-voice-rtx50:/app/logs ./backup-logs-$(date +%Y%m%d)
podman cp care-voice-rtx50:/etc/supervisor ./backup-config-$(date +%Y%m%d)
```

---

**狀態**: 🟢 RTX 50 系列容器管理系統就緒  
**最後更新**: 2025-07-24  
**主要容器**: care-voice-rtx50 (端口 8001, RTX 5070 Ti GPU 加速)  
**管理重點**: CDI GPU 存取 + 多世代兼容 + 混合精度優化