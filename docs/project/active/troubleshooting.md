# 🚨 Care Voice 故障排除指南

## 🎯 快速診斷

### 第一步：基本檢查
```bash
# 檢查服務狀態
podman ps | grep care-voice

# 檢查服務健康
curl http://localhost:8001/health

# 檢查容器日誌
podman logs care-voice-ultimate | tail -20
```

## ⚡ 快速修復指令

### 最常見問題的一鍵修復
```bash
# 1. 服务完全重啟 (解决 90% 问题)
podman restart care-voice-ultimate

# 2. GPU 訪問恢復
podman exec care-voice-ultimate python3 /app/gpu_diagnostics.py

# 3. 健康檢查確認
curl http://localhost:8001/health | jq

# 4. 容器重新部署 (如果以上都失效)
podman stop care-voice-ultimate && podman rm care-voice-ultimate
podman run -d --name care-voice-ultimate --gpus all -p 8001:8001 \
  -v ./backend/models:/app/models:ro care-voice:whisper-rs-gpu-v2
```

## 🔥 常見問題解決

### 1. 容器無法啟動

#### 症狀
- `podman run` 命令執行失敗
- 容器狀態顯示 "Exited"

#### 解決方案
```bash
# 檢查詳細錯誤
podman logs care-voice-ultimate

# 檢查埠口衝突
lsof -i :8001

# 停止衝突服務
podman stop $(podman ps -q --filter "ancestor=care-voice:whisper-rs-gpu-v2")

# 清理並重新啟動
podman rm care-voice-ultimate
podman run -d --name care-voice-ultimate --gpus all -p 8001:8001 \
  -v ./backend/models:/app/models:ro care-voice:whisper-rs-gpu-v2
```

### 2. GPU 不可用

#### 症狀
- 健康檢查顯示 `"gpu_available": false`
- 轉錄速度異常慢

#### 診斷命令
```bash
# 檢查主機 GPU
nvidia-smi

# 檢查容器內 GPU
podman exec -it care-voice-ultimate nvidia-smi

# 檢查 CUDA 版本 (應為 12.9.1)
podman exec -it care-voice-ultimate nvcc --version

# 使用專用 GPU 診斷工具
podman exec care-voice-ultimate python3 /app/gpu_diagnostics.py
```

#### 解決方案
```bash
# 重新安裝 NVIDIA Container Toolkit
sudo apt remove nvidia-container-toolkit
sudo apt update
sudo apt install nvidia-container-toolkit

# 重啟容器服務
sudo systemctl restart docker  # 如使用 Docker

# 檢查 --gpus 參數
podman run -d --name care-voice --gpus all -p 8001:8001 care-voice:whisper-rs-gpu
```

### 3. 記憶體不足

#### 症狀
- 容器突然停止
- 日誌顯示 "Out of memory" 錯誤
- GPU 記憶體耗盡

#### 診斷命令
```bash
# 檢查系統記憶體
free -h

# 檢查 GPU 記憶體
nvidia-smi --query-gpu=memory.used,memory.total --format=csv

# 檢查容器資源使用
podman stats care-voice
```

#### 解決方案
```bash
# 使用較小模型
# 將 models/ggml-medium.bin 替換為 models/ggml-base.bin

# 限制容器記憶體
podman run -d --name care-voice --gpus all \
  --memory=6g --memory-swap=6g \
  -p 8001:8001 care-voice:whisper-rs-gpu

# 設置環境變數
podman run -d --name care-voice --gpus all \
  -e CUDA_MEMORY_POOL_DISABLED=1 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

### 4. 模型載入失敗

#### 症狀
- 健康檢查顯示 `"model_loaded": false`
- 轉錄請求返回錯誤

#### 診斷命令
```bash
# 檢查模型文件
ls -la models/
file models/ggml-*.bin

# 檢查模型掛載
podman exec -it care-voice ls -la /app/models/
```

#### 解決方案
```bash
# 重新下載模型
rm models/ggml-*.bin
curl -L -o models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# 檢查文件權限
chmod 644 models/ggml-*.bin

# 重新掛載模型
podman stop care-voice
podman rm care-voice
podman run -d --name care-voice --gpus all -p 8001:8001 \
  -v $(pwd)/models:/app/models:ro care-voice:whisper-rs-gpu
```

### 5. 網路連接問題

#### 症狀
- 無法訪問 http://localhost:8001
- 前端無法連接到後端

#### 診斷命令
```bash
# 檢查埠口綁定
netstat -tulpn | grep 8001

# 檢查防火牆
sudo ufw status  # Ubuntu
sudo firewall-cmd --list-ports  # Fedora/RHEL

# 檢查容器網路
podman port care-voice
```

#### 解決方案
```bash
# 開放防火牆埠口
sudo ufw allow 8001  # Ubuntu
sudo firewall-cmd --permanent --add-port=8001/tcp && sudo firewall-cmd --reload  # Fedora/RHEL

# 綁定到所有介面
podman run -d --name care-voice --gpus all -p 0.0.0.0:8001:8001 care-voice:whisper-rs-gpu

# 檢查 SELinux (如適用)
setsebool -P container_connect_any 1
```

### 6. 音頻格式轉換問題 ⭐ (已解決方案)

#### 症狀
- Chrome/Edge 瀏覽器錄音後出現 `422 Unprocessable Entity` 錯誤
- Firefox 瀏覽器也出現同樣錯誤 (2025年更新)
- 錯誤信息: "Audio format conversion failed"
- Safari 需要 HTTPS 才能錄音

#### 根本原因 (已確認)
**2025年重大發現**: 所有現代瀏覽器都已遷移到 Opus 編碼器:
- **Chrome/Edge**: `audio/webm;codecs=opus`
- **Firefox**: `audio/ogg;codecs=opus` (從 Vorbis 遷移)
- **Safari**: `audio/mp4` (AAC 編碼，需要 HTTPS)

**技術限制**: 後端 symphonia 0.5.4 不支援 Opus 解碼器

📊 **深度分析**: [瀏覽器音頻錄製完整分析](../technical/BROWSER_AUDIO_RECORDING_ANALYSIS.md)

#### 快速診斷
```bash
# 檢查當前錯誤狀況
podman exec care-voice-ultimate grep -E "(Audio conversion failed|不支援)" /var/log/supervisor/whisper-rs.log | tail -5

# 檢查瀏覽器使用的格式
podman logs care-voice-ultimate | grep -E "(webm|ogg|mp4)" | tail -3

# 檢查服務狀態
curl -s http://localhost:8001/health | jq
```

#### 🚀 推薦解決方案: Opus 後端處理
**基於業界標準 (Discord/Zoom/Google 同款技術)**:

##### 完整解決方案文檔
- 🎯 **[Opus 後端處理方案](../technical/OPUS_BACKEND_SOLUTION.md)** - 業界標準技術方案
- 🛠️ **[實施指南](../development/OPUS_IMPLEMENTATION_GUIDE.md)** - 詳細實施步驟
- 📊 **[多方案對比](../technical/WEBM_SOLUTION_PLAN.md)** - 技術方案比較

##### 核心修復步驟
```bash
# 1. 實施 Opus 後端支援 (推薦方案)
# 更新 backend/Cargo.toml
[dependencies]
opus = "0.3.0"              # 原生 Opus 解碼器
ogg = "0.9.0"               # Firefox OGG 容器支援
webm-parser = "0.1.0"       # Chrome WebM 容器支援

# 2. 重建容器 (包含 Opus 支援)
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:opus-support .

# 3. 部署更新版本
podman stop care-voice-ultimate && podman rm care-voice-ultimate
podman run -d --name care-voice-ultimate --device /dev/nvidia0 \
  --device /dev/nvidiactl --device /dev/nvidia-uvm \
  -p 8001:8001 \
  -v ./backend/models:/app/models:ro \
  care-voice:opus-support

# 4. 驗證修復效果
curl -s http://localhost:8001/health
# 測試 Chrome/Firefox 錄音功能
```

#### 預期修復效果
```
修復前:
Chrome:  ❌ WebM Opus → 422 錯誤
Firefox: ❌ OGG Opus → 422 錯誤
Safari:  ❓ 需要 HTTPS 測試

修復後:
Chrome:  ✅ WebM Opus → 成功轉錄
Firefox: ✅ OGG Opus → 成功轉錄  
Safari:  ✅ MP4 AAC → 成功轉錄 (HTTPS 環境)
```

#### 臨時應急方案
如果暫時無法實施 Opus 支援，建議:

1. **引導用戶使用 Safari** (需要 HTTPS)
2. **提供用戶友善的錯誤信息**:
   ```
   "⚠️ 瀏覽器音頻格式暫時不支援
   建議: 1. 使用 Safari 瀏覽器
        2. 或等待系統更新 (Opus 支援)"
   ```

### 7. 其他音頻上傳問題

#### 症狀
- 錄音功能無法使用
- 音頻文件上傳失敗 (非格式問題)

#### 瀏覽器相關
```javascript
// 檢查瀏覽器控制台錯誤
// 確保使用 HTTPS 或 localhost
// 檢查麥克風權限
```

#### 後端診斷
```bash
# 檢查上傳端點
curl -X POST -F "audio=@test.wav" http://localhost:8001/api/upload

# 檢查文件大小限制
podman exec -it care-voice cat /etc/nginx/nginx.conf | grep client_max_body_size
```

#### 解決方案
```bash
# 增加文件大小限制 (如需要)
# 編輯 unified-nginx.conf
client_max_body_size 100M;

# 重新建構容器
podman build --no-cache -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu .
```

## 🛠️ 進階故障排除

### 容器建構問題

#### CUDA 映像下載失敗
```bash
# 使用特定版本
podman build --build-arg CUDA_VERSION=12.1.1 -f Dockerfile.whisper-rs-gpu .

# 清理並重試
podman system prune -a
podman build --no-cache -f Dockerfile.whisper-rs-gpu .
```

#### CMake 版本錯誤
```dockerfile
# 在 Dockerfile 中確保 CMake 版本
RUN cmake --version  # 應該 >= 3.18
```

#### Rust 編譯錯誤
```bash
# 檢查編譯環境
podman run -it --rm nvidia/cuda:12.1.1-devel-ubuntu20.04 bash
apt update && apt install build-essential cmake clang libclang-dev
```

### 效能問題

#### 轉錄速度慢
```bash
# 確認 GPU 加速
podman exec -it care-voice nvidia-smi

# 檢查模型大小
ls -lh models/

# 使用更小模型
# base < medium < large (速度由快到慢)
```

#### 高 CPU 使用率
```bash
# 限制 CPU 使用
podman run -d --name care-voice --gpus all --cpus=2 -p 8001:8001 care-voice:whisper-rs-gpu

# 檢查資源分配
podman stats care-voice
```

## 📊 監控與日誌

### 實時監控
```bash
# GPU 使用監控
watch -n 1 'podman exec care-voice nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total --format=csv'

# 容器資源監控
watch -n 1 'podman stats care-voice'

# 服務狀態監控
watch -n 5 'curl -s http://localhost:8001/health | jq'
```

### 日誌分析
```bash
# 查看詳細日誌
podman logs -f care-voice

# 過濾錯誤訊息
podman logs care-voice 2>&1 | grep -i error

# 日誌輪換設置
podman run -d --name care-voice --gpus all \
  --log-driver=journald --log-opt max-size=10m \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

## 🔄 重置與清理

### 完全重置
```bash
# 停止並移除容器
podman stop care-voice
podman rm care-voice

# 移除映像
podman rmi care-voice:whisper-rs-gpu

# 清理系統
podman system prune -a

# 重新建構和部署
podman build -f Dockerfile.whisper-rs-gpu -t care-voice:whisper-rs-gpu .
podman run -d --name care-voice --gpus all -p 8001:8001 care-voice:whisper-rs-gpu
```

### 保留資料重置
```bash
# 只重啟服務
podman restart care-voice

# 重新部署容器 (保留模型)
podman stop care-voice && podman rm care-voice
podman run -d --name care-voice --gpus all -p 8001:8001 \
  -v $(pwd)/models:/app/models:ro care-voice:whisper-rs-gpu
```

## 📞 獲取幫助

### 日誌收集
```bash
# 收集系統資訊
{
  echo "=== System Info ==="
  uname -a
  echo "=== GPU Info ==="
  nvidia-smi
  echo "=== Container Info ==="
  podman version
  echo "=== Service Logs ==="
  podman logs care-voice | tail -50
} > care-voice-debug.log
```

### 社群支援與相關文檔
- **GitHub Issues**: [專案 Issues 頁面]
- **完整文檔系統**:
  - [GPU 配置與診斷](../technical/gpu-configuration.md) - 詳細的 GPU 設置和故障排除
  - [部署指南](../development/deployment-guide.md) - 容器部署和監控
  - [環境配置](../development/environment-setup.md) - CUDA 12.9.1 環境設置
  - [系統狀態](../technical/system-status.md) - 專案當前狀態和成就
- **角色定義**: [開發規範](../../claude.md)

### 效能監控命令
```bash
# GPU 使用率實時監控 (從 claude.md)
watch -n 1 'podman exec care-voice-ultimate nvidia-smi'

# 容器資源監控
podman stats care-voice-ultimate

# 服務健康狀態監控
watch -n 5 'curl -s http://localhost:8001/health | jq'
```

---

**提示**: 90% 的問題可以通過 `podman restart care-voice-ultimate` 解決。對於 GPU 相關問題，請使用 `python3 /app/gpu_diagnostics.py` 診斷工具並參考 [GPU 配置指南](../technical/gpu-configuration.md)。