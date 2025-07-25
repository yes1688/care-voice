# 🚨 Care Voice 故障排除指南

## 🎯 快速診斷

### 第一步：基本檢查
```bash
# 檢查服務狀態
podman ps | grep care-voice

# 檢查服務健康
curl http://localhost:8001/health

# 檢查容器日誌
podman logs care-voice | tail -20
```

## 🔥 常見問題解決

### 1. 容器無法啟動

#### 症狀
- `podman run` 命令執行失敗
- 容器狀態顯示 "Exited"

#### 解決方案
```bash
# 檢查詳細錯誤
podman logs care-voice

# 檢查埠口衝突
lsof -i :8001

# 停止衝突服務
podman stop $(podman ps -q --filter "ancestor=care-voice:whisper-rs-gpu")

# 清理並重新啟動
podman rm care-voice
podman run -d --name care-voice --gpus all -p 8001:8001 care-voice:whisper-rs-gpu
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
podman exec -it care-voice nvidia-smi

# 檢查 CUDA 版本
podman exec -it care-voice nvcc --version
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

### 6. 音頻上傳問題

#### 症狀
- 錄音功能無法使用
- 音頻文件上傳失敗

#### 瀏覽器相關
```javascript
// 檢查瀏覽器控制台錯誤
// 確保使用 HTTPS 或 localhost
// 檢查麥克風權限
```

#### 後端診斷
```bash
# 檢查上傳端點
curl -X POST -F "audio=@test.wav" http://localhost:8001/transcribe

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

### 社群支援
- **GitHub Issues**: [專案 Issues 頁面]
- **文檔**: [完整技術文檔](../technical/)
- **配置參考**: [系統配置](../../claude.md)

---

**提示**: 大多數問題可以通過重新啟動容器解決。如果問題持續存在，請收集日誌並參考 [GPU 配置指南](../technical/gpu-configuration.md)。