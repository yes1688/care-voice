# 🏗️ Care Voice 統一架構部署指南

**版本**: v2.0 統一架構版  
**更新日期**: 2025-07-26  
**架構**: 統一 multi-stage Dockerfile  

## 🎯 快速開始

### **一鍵部署 (推薦)**
```bash
# 部署 Care Voice 統一服務
./deploy.sh
```

**✅ 新架構特點**: 使用標準 multi-stage 構建，大幅簡化部署流程

### **服務管理**
```bash
# 啟動服務
./manage.sh start

# 停止服務  
./manage.sh stop

# 查看狀態
./manage.sh status

# 查看日誌
./manage.sh logs
```

### **開發模式**
```bash
# 完整構建選項
./build.sh           # 生產構建
./build.sh dev       # 開發模式
./build.sh build-only # 僅構建
```

---

## 📋 架構說明

### **統一 Multi-Stage 構建**
```
Dockerfile.unified:
1. 前端構建階段 → Node.js 20 + SolidJS + Vite
2. 後端構建階段 → Rust 1.85 + whisper-rs + Opus
3. 最終整合階段 → nginx + supervisor 統一管理
```

### **最終服務架構**
```
用戶 :8000
    ↓
nginx (統一入口)
    ├── / → 前端 SolidJS 應用
    ├── /api → 後端 Rust 服務 :8001
    └── /health → 健康檢查

統一容器 care-voice-integrated
    ├── nginx (前端靜態文件)
    ├── supervisor (進程管理)
    └── care-voice (後端服務)
```

---

## 🔧 文件說明

### **核心腳本 (僅3個)**
| 腳本 | 用途 | 使用頻率 |
|------|------|----------|
| `deploy.sh` | 一鍵部署 | ⭐⭐⭐ 主要使用 |
| `manage.sh` | 服務管理 | ⭐⭐ 日常管理 |
| `build.sh` | 完整構建 | ⭐ 開發調試 |

### **配置文件**
| 文件 | 用途 |
|------|------|
| `Dockerfile.unified` | 統一 multi-stage 構建 |
| `podman-compose.simple.yml` | 簡化服務編排 |
| `nginx-integrated.conf` | 統一 nginx 配置 |
| `supervisord-integrated.conf` | 多進程管理 |

---

## 🚀 使用方式

### **1. 首次部署**
```bash
# 一鍵部署 (最簡單)
./deploy.sh
```

### **2. 日常管理**
```bash
# 啟動服務
./manage.sh start

# 停止服務
./manage.sh stop

# 重啟服務
./manage.sh restart

# 查看狀態
./manage.sh status

# 實時日誌
./manage.sh logs

# 健康檢查
./manage.sh health
```

### **3. 開發模式**
```bash
# 生產構建
./build.sh

# 開發模式 (前端熱重載)
./build.sh dev

# 僅構建不啟動
./build.sh build-only
```

---

## 🔍 服務管理

### **快速命令**
```bash
# 所有操作都通過 manage.sh
./manage.sh status    # 查看狀態
./manage.sh logs      # 查看日誌  
./manage.sh health    # 健康檢查
./manage.sh restart   # 重啟服務
```

### **手動 compose 操作** (進階)
```bash
# 直接使用 compose (不推薦日常使用)
podman-compose -f podman-compose.simple.yml ps
podman-compose -f podman-compose.simple.yml logs -f
podman-compose -f podman-compose.simple.yml down
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
podman-compose -f podman-compose.simple.yml logs

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
podman-compose -f podman-compose.simple.yml build care-voice-integrated
podman-compose -f podman-compose.simple.yml up -d --force-recreate
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