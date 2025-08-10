# Care Voice AI 部署指南

## 系統需求

### 硬體需求
- **GPU**: NVIDIA GTX 10xx+ 或 RTX 系列（可選）
- **記憶體**: 8GB+ 系統記憶體
- **VRAM**: 4GB+（啟用 GPU 時）

### 軟體需求
- Docker 或 Podman
- Docker Compose
- NVIDIA Container Toolkit（GPU 支援）

## 部署步驟

### 1. 環境準備
```bash
# 檢查 GPU 支援
nvidia-smi

# 檢查容器運行時
docker --version
docker-compose --version
```

### 2. 啟動服務
```bash
# 統一部署（推薦）
./start.sh

# 或使用 Docker Compose
docker-compose -f docker-compose.unified.yml up -d
```

### 3. 驗證部署
```bash
# 檢查服務狀態
curl http://localhost:3000/health

# 檢查容器狀態
docker ps
```

### 4. 停止服務
```bash
./stop.sh
```

## 故障排除

### GPU 相關問題
- 確認 NVIDIA 驅動程式已安裝
- 檢查 Docker GPU 支援：`docker run --gpus all nvidia/cuda:11.0-base nvidia-smi`

### 埠口衝突
- 檢查埠口佔用：`lsof -i :3000`、`lsof -i :8081`
- 修改 docker-compose 中的埠口映射

### 記憶體不足
- 監控系統資源：`htop`、`nvidia-smi`
- 調整容器記憶體限制

## 生產環境配置

### 安全設定
- 設定防火牆規則
- 使用 HTTPS（配置 SSL 憑證）
- 設定適當的檔案權限

### 效能優化
- 調整容器資源限制
- 配置記憶體快取
- 設定 GPU 記憶體管理