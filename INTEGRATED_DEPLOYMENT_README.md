# 🏗️ Care Voice 整合部署指南

## 🎯 快速開始

### **最簡單的方式 (推薦)**
```bash
# 一鍵部署
./deploy-simple.sh
```

### **完整構建方式**
```bash
# 詳細構建和部署
./build-integrated.sh
```

### **使用 podman-compose**
```bash
# 構建並啟動
podman-compose -f podman-compose.integrated.yml up --build -d

# 僅構建不啟動
podman-compose -f podman-compose.integrated.yml build
```

---

## 📋 架構說明

### **三階段構建流程**
```
1. 前端編譯 → frontend/Dockerfile.build → care-voice-frontend:latest
2. 後端編譯 → backend/Dockerfile.build → care-voice-backend:latest  
3. 最終整合 → Dockerfile.final → care-voice-integrated:latest
```

### **最終服務架構**
```
用戶 :8000
    ↓
nginx (統一入口)
    ├── / → 前端 SolidJS 應用
    ├── /api → 後端 Rust 服務 :8001
    └── /health → 健康檢查
```

---

## 🔧 文件說明

| 文件 | 用途 |
|------|------|
| `frontend/Dockerfile.build` | 前端編譯階段 |
| `backend/Dockerfile.build` | 後端編譯階段 |
| `Dockerfile.final` | 最終整合階段 |
| `nginx-integrated.conf` | 統一 nginx 配置 |
| `supervisord-integrated.conf` | 多進程管理 |
| `podman-compose.integrated.yml` | 整合編排配置 |
| `build-integrated.sh` | 完整構建腳本 |
| `deploy-simple.sh` | 簡化部署腳本 |

---

## 🚀 使用方式

### **1. 生產環境部署**
```bash
# 方式一: 一鍵部署 (最簡單)
./deploy-simple.sh

# 方式二: 完整構建
./build-integrated.sh production

# 方式三: compose 手動
podman-compose -f podman-compose.integrated.yml up -d --build
```

### **2. 開發環境**
```bash
# 啟動開發模式 (前端熱重載)
./build-integrated.sh dev

# 或使用 compose
podman-compose -f podman-compose.integrated.yml --profile dev up -d
```

### **3. 僅構建不啟動**
```bash
# 僅構建所有鏡像
./build-integrated.sh build-only

# 或分別構建
podman-compose -f podman-compose.integrated.yml build frontend-builder
podman-compose -f podman-compose.integrated.yml build backend-builder  
podman-compose -f podman-compose.integrated.yml build care-voice-integrated
```

---

## 🔍 服務管理

### **查看狀態**
```bash
# 容器狀態
podman-compose -f podman-compose.integrated.yml ps

# 服務日誌
podman-compose -f podman-compose.integrated.yml logs -f

# 健康檢查
curl http://localhost:8000/health
```

### **停止服務**
```bash
# 停止所有服務
podman-compose -f podman-compose.integrated.yml down

# 停止並清理
podman-compose -f podman-compose.integrated.yml down --volumes --remove-orphans
```

### **重啟服務**
```bash
# 重啟整合服務
podman-compose -f podman-compose.integrated.yml restart care-voice-integrated

# 重新構建並重啟
podman-compose -f podman-compose.integrated.yml up -d --build --force-recreate
```

---

## 🌐 服務端點

| 端點 | 功能 | 示例 |
|------|------|------|
| `http://localhost:8000` | 前端應用 | 打開瀏覽器訪問 |
| `http://localhost:8000/api/upload` | 音頻上傳 API | POST 音頻文件 |
| `http://localhost:8000/health` | 健康檢查 | GET 請求 |

### **開發模式額外端點** (如果啟用)
| 端點 | 功能 |
|------|------|
| `http://localhost:3000` | 前端開發服務器 |
| `http://localhost:8001` | 後端開發服務器 |

---

## 🐛 故障排除

### **常見問題**

#### **1. 構建失敗**
```bash
# 檢查依賴
podman --version
podman-compose --version

# 清理重試
podman system prune -f
./build-integrated.sh build-only
```

#### **2. 服務啟動失敗**
```bash
# 查看日誌
podman-compose -f podman-compose.integrated.yml logs

# 檢查容器狀態
podman ps -a | grep care-voice

# 檢查健康狀態
podman inspect care-voice-integrated | grep Health
```

#### **3. 前端無法訪問**
```bash
# 檢查 nginx 配置
podman exec care-voice-integrated nginx -t

# 檢查靜態文件
podman exec care-voice-integrated ls -la /usr/share/nginx/html/
```

#### **4. 後端 API 失敗**
```bash
# 檢查後端進程
podman exec care-voice-integrated ps aux | grep care-voice

# 檢查後端日誌
podman exec care-voice-integrated tail -f /var/log/supervisor/backend_stdout.log
```

### **調試模式**
```bash
# 詳細輸出
VERBOSE=true ./build-integrated.sh

# 跳過清理 (保留舊鏡像)
SKIP_CLEANUP=true ./build-integrated.sh

# 進入容器調試
podman exec -it care-voice-integrated /bin/bash
```

---

## 📊 性能與監控

### **資源使用**
```bash
# 查看資源使用
podman stats care-voice-integrated

# 查看鏡像大小
podman images | grep care-voice
```

### **日誌管理**
```bash
# 日誌位置
/var/log/supervisor/    # supervisor 日誌
/var/log/nginx/         # nginx 日誌
/var/log/care-voice/    # 應用日誌

# 實時日誌
podman exec care-voice-integrated tail -f /var/log/supervisor/*.log
```

---

## 🔧 自定義配置

### **修改端口**
編輯 `podman-compose.integrated.yml`:
```yaml
ports:
  - "9000:8000"  # 改為 9000 端口
```

### **修改 nginx 配置**
編輯 `nginx-integrated.conf` 後重新構建:
```bash
podman-compose -f podman-compose.integrated.yml build care-voice-integrated
podman-compose -f podman-compose.integrated.yml up -d --force-recreate
```

### **環境變數**
編輯 `podman-compose.integrated.yml`:
```yaml
environment:
  - RUST_LOG=debug  # 修改日誌級別
  - TZ=UTC          # 修改時區
```

---

## 🎯 優勢特點

✅ **一鍵部署**: 單一命令完成所有構建和部署  
✅ **分階段構建**: 前後端並行編譯，充分利用快取  
✅ **統一入口**: 單一端口 (8000) 提供完整服務  
✅ **自動監控**: 內建健康檢查和日誌管理  
✅ **開發友善**: 支援開發模式和熱重載  
✅ **容器化**: 完整隔離，無環境污染  

---

## 📞 需要幫助？

- 查看詳細架構: `docs/development/INTEGRATED_ARCHITECTURE_DESIGN.md`
- 檢查容器狀態: `docs/development/CARE_VOICE_CONTAINER_STATUS_OVERVIEW.md`  
- 專案架構指南: `docs/development/PROJECT_ARCHITECTURE_GUIDE.md`