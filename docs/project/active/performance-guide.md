# 📊 Care Voice 效能指南

## 🎯 效能概覽

### 基準測試結果

| 配置 | VRAM 使用 | 轉錄速度 | 啟動時間 | 記憶體效率 |
|------|-----------|----------|----------|------------|
| **whisper-rs GPU** | ~3GB | 實時 | <30s | 優化 |
| **標準 PyTorch** | ~6GB | 0.5x 實時 | ~60s | 標準 |
| **CPU 版本** | 0GB | 0.1x 實時 | <20s | 基準 |

### 效能改善指標
- **記憶體節省**: 50% VRAM 使用減少
- **啟動加速**: 50% 啟動時間縮短
- **轉錄效率**: Rust 原生性能優勢
- **資源利用**: 更好的 GPU 利用率

## 🚀 效能優化配置

### GPU 記憶體優化

#### 基本優化
```bash
# 啟用記憶體池禁用
podman run -d --name care-voice --gpus all \
  -e CUDA_MEMORY_POOL_DISABLED=1 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

#### 進階記憶體管理
```bash
# 設置 GPU 記憶體增長策略
podman run -d --name care-voice --gpus all \
  -e TF_FORCE_GPU_ALLOW_GROWTH=true \
  -e CUDA_VISIBLE_DEVICES=0 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

### 模型選擇策略

#### 效能 vs 準確度對比
| 模型 | 大小 | VRAM | 轉錄速度 | 準確度 | 適用場景 |
|------|------|------|----------|--------|----------|
| **base** | 150MB | 1-2GB | 2x 實時 | 良好 | 實時應用 |
| **medium** | 1.5GB | 2-4GB | 1x 實時 | 優秀 | 平衡選擇 |
| **large-v3** | 3GB | 4-8GB | 0.8x 實時 | 最佳 | 高品質需求 |

#### 動態模型切換
```bash
# 使用基礎模型 (快速)
podman run -d --name care-voice-fast --gpus all \
  -v $(pwd)/models/ggml-base.bin:/app/models/model.bin:ro \
  -p 8001:8001 care-voice:whisper-rs-gpu

# 使用大型模型 (高精度)
podman run -d --name care-voice-quality --gpus all \
  -v $(pwd)/models/ggml-large-v3.bin:/app/models/model.bin:ro \
  -p 8002:8001 care-voice:whisper-rs-gpu
```

## 📈 效能監控

### 實時監控設置

#### GPU 使用率監控
```bash
# 基本 GPU 監控
watch -n 1 'nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv,noheader,nounits'

# 詳細 GPU 指標
nvidia-smi dmon -s pucvmet -d 1
```

#### 容器資源監控
```bash
# 容器效能統計
podman stats care-voice --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"

# 持續監控
watch -n 2 'podman stats care-voice --no-stream'
```

### 效能基準測試

#### 轉錄速度測試
```bash
# 建立測試腳本
cat > benchmark_test.sh << 'EOF'
#!/bin/bash
echo "開始轉錄效能測試..."
start_time=$(date +%s.%N)

curl -X POST -F "audio=@test_audio.wav" \
  http://localhost:8001/transcribe > /dev/null 2>&1

end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)
echo "轉錄完成時間: ${duration} 秒"
EOF

chmod +x benchmark_test.sh
./benchmark_test.sh
```

#### 並發測試
```bash
# 並發請求測試
for i in {1..5}; do
  curl -X POST -F "audio=@test_audio.wav" \
    http://localhost:8001/transcribe &
done
wait

# 使用 Apache Bench 測試
ab -n 100 -c 10 http://localhost:8001/health
```

## ⚡ 高效能配置

### 硬體配置建議

#### 最佳效能配置
```yaml
硬體規格:
  CPU: Intel i7/AMD Ryzen 7 或更高
  記憶體: 16GB+ DDR4
  GPU: RTX 3070/4060 或更高
  存儲: NVMe SSD
  網路: 千兆乙太網
```

#### 容器資源限制
```bash
# 平衡效能配置
podman run -d --name care-voice --gpus all \
  --cpus="4" \
  --memory="8g" \
  --memory-swap="8g" \
  --shm-size="2g" \
  -p 8001:8001 care-voice:whisper-rs-gpu

# 高效能配置
podman run -d --name care-voice --gpus all \
  --cpus="8" \
  --memory="16g" \
  --memory-swap="16g" \
  --shm-size="4g" \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

### 系統調優

#### Linux 核心參數
```bash
# 優化網路效能
echo 'net.core.rmem_max = 16777216' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 65536 16777216' >> /etc/sysctl.conf

# 套用設定
sudo sysctl -p
```

#### GPU 調優
```bash
# 設置 GPU 效能模式
sudo nvidia-smi -pm 1

# 設置最大效能狀態
sudo nvidia-smi -ac 5001,1177  # 依 GPU 型號調整
```

## 🔧 進階優化

### 混合精度推理

#### 啟用 FP16 優化
```bash
# 啟用半精度推理
podman run -d --name care-voice --gpus all \
  -e ENABLE_FP16=1 \
  -e CUDA_ALLOW_HALF=1 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

#### Tensor Core 優化
```bash
# 啟用 Tensor Core 加速
podman run -d --name care-voice --gpus all \
  -e CUBLAS_WORKSPACE_CONFIG=:4096:8 \
  -e NVIDIA_TF32_OVERRIDE=1 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

### 多 GPU 配置

#### 多 GPU 平行處理
```bash
# 使用多個 GPU
podman run -d --name care-voice --gpus all \
  -e CUDA_VISIBLE_DEVICES=0,1 \
  -p 8001:8001 care-voice:whisper-rs-gpu

# GPU 負載平衡
for gpu in 0 1; do
  podman run -d --name care-voice-gpu${gpu} \
    --gpus "device=${gpu}" \
    -e CUDA_VISIBLE_DEVICES=${gpu} \
    -p 800${gpu}:8001 care-voice:whisper-rs-gpu
done
```

### 快取優化

#### 模型快取策略
```bash
# 預載入模型快取
podman run -d --name care-voice --gpus all \
  -e PRELOAD_MODEL=1 \
  -e MODEL_CACHE_SIZE=1024 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

#### 音頻處理快取
```bash
# 啟用音頻處理快取
podman run -d --name care-voice --gpus all \
  -v /tmp/audio_cache:/tmp/cache \
  -e AUDIO_CACHE_ENABLED=1 \
  -p 8001:8001 care-voice:whisper-rs-gpu
```

## 📊 效能分析工具

### GPU 分析

#### NVIDIA Nsight Systems
```bash
# 安裝 Nsight Systems
sudo apt install nsight-systems

# 分析 GPU 效能
nsys profile --trace=cuda,nvtx podman exec care-voice /app/care-voice
```

#### GPU 記憶體分析
```bash
# 使用 nvidia-ml-py 監控
pip install nvidia-ml-py3

python3 << 'EOF'
import pynvml
pynvml.nvmlInit()
handle = pynvml.nvmlDeviceGetHandleByIndex(0)
info = pynvml.nvmlDeviceGetMemoryInfo(handle)
print(f"GPU Memory: {info.used/1024**3:.1f}GB / {info.total/1024**3:.1f}GB")
EOF
```

### 應用效能分析

#### 轉錄延遲分析
```bash
# 詳細時間測量
curl -w "@curl-format.txt" -X POST -F "audio=@test.wav" \
  http://localhost:8001/transcribe

# curl-format.txt 內容:
cat > curl-format.txt << 'EOF'
     time_namelookup:  %{time_namelookup}\n
        time_connect:  %{time_connect}\n
     time_appconnect:  %{time_appconnect}\n
    time_pretransfer:  %{time_pretransfer}\n
       time_redirect:  %{time_redirect}\n
  time_starttransfer:  %{time_starttransfer}\n
                     ----------\n
          time_total:  %{time_total}\n
EOF
```

## 🎯 效能調優檢查清單

### 基本優化 ✅
- [ ] 選擇合適的模型大小
- [ ] 啟用 GPU 加速
- [ ] 設置適當的資源限制
- [ ] 優化網路配置

### 進階優化 ✅
- [ ] 啟用混合精度推理
- [ ] 配置 GPU 記憶體管理
- [ ] 設置模型快取
- [ ] 調優系統參數

### 監控設置 ✅
- [ ] 部署 GPU 監控
- [ ] 設置效能警報
- [ ] 建立基準測試
- [ ] 定期效能評估

## 📈 效能最佳實踐

### 1. 漸進式優化
- 從基本配置開始
- 逐步套用優化選項
- 每次變更後測量效能
- 記錄最佳配置

### 2. 監控導向調優
- 持續監控關鍵指標
- 識別效能瓶頸
- 針對性優化
- 驗證改善效果

### 3. 負載平衡策略
- 多模型部署
- 請求分流
- 動態擴展
- 故障轉移

---

**效能調優提示**: 效能優化是一個持續過程，建議定期評估和調整配置以適應不同的工作負載需求。