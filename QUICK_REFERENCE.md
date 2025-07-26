# ⚡ Care Voice 快速參考指南

**最新更新**: 2025-07-26  
**版本**: v1.0 整合架構版  

---

## 🚀 快速開始 (30秒)

```bash
# 一鍵部署 Care Voice
./deploy.sh

# 訪問服務
open http://localhost:8000
```

---

## 📋 核心命令速查

### **部署相關**
```bash
./deploy.sh           # 一鍵部署 (最常用)
./build.sh            # 完整構建 (開發用)
./build.sh dev        # 開發模式
./build.sh build-only # 僅構建
```

### **服務管理**
```bash
./manage.sh start     # 啟動服務
./manage.sh stop      # 停止服務
./manage.sh restart   # 重啟服務
./manage.sh status    # 查看狀態
./manage.sh logs      # 實時日誌
./manage.sh health    # 健康檢查
```

---

## 🌐 服務端點速查

| 端點 | 功能 | 示例 |
|------|------|------|
| `http://localhost:8000` | 前端應用 | 瀏覽器直接訪問 |
| `http://localhost:8000/api/upload` | 音頻上傳 | POST 音頻文件 |
| `http://localhost:8000/health` | 健康檢查 | GET 請求 |

---

## 🐛 故障排除速查

### **服務無法啟動**
```bash
./manage.sh status    # 檢查狀態
./manage.sh logs      # 查看錯誤日誌
./manage.sh health    # 詳細健康檢查
```

### **重新部署**
```bash
./manage.sh stop      # 停止服務
./deploy.sh           # 重新部署
```

### **完全重置**
```bash
podman-compose -f podman-compose.integrated.yml down --volumes
./deploy.sh
```

---

## 📁 文件結構速查

### **核心腳本**
```
deploy.sh             # 一鍵部署
manage.sh             # 服務管理  
build.sh              # 完整構建
```

### **配置文件**
```
nginx-integrated.conf              # nginx 配置
supervisord-integrated.conf        # 進程管理
podman-compose.integrated.yml      # 服務編排
```

### **構建文件**
```
frontend/Dockerfile.build         # 前端編譯
backend/Dockerfile.build          # 後端編譯
Dockerfile.final                  # 最終整合
```

---

## ⚙️ 環境變數速查

### **構建選項**
```bash
VERBOSE=true ./build.sh           # 詳細輸出
SKIP_CLEANUP=true ./build.sh      # 跳過清理
```

### **服務配置**
```bash
RUST_LOG=debug                    # 日誌級別
TZ=Asia/Taipei                    # 時區設定
```

---

## 🔍 日誌位置速查

### **容器內日誌**
```
/var/log/supervisor/              # supervisor 日誌
/var/log/nginx/                   # nginx 日誌
/var/log/care-voice/              # 應用日誌
```

### **查看日誌命令**
```bash
./manage.sh logs                  # 實時日誌
podman logs care-voice-integrated # 容器日誌
```

---

## 📊 狀態檢查速查

### **服務狀態**
```bash
./manage.sh status                # 完整狀態報告
curl http://localhost:8000/health # 健康檢查
podman ps | grep care-voice       # 容器狀態
```

### **資源使用**
```bash
podman stats care-voice-integrated # 資源使用
podman images | grep care-voice    # 鏡像大小
```

---

## 🔧 開發模式速查

### **前端開發**
```bash
./build.sh dev                    # 啟動開發模式
# 前端: http://localhost:3000
# 後端: http://localhost:8001
```

### **調試命令**
```bash
podman exec -it care-voice-integrated /bin/bash  # 進入容器
./manage.sh logs                                 # 實時日誌
VERBOSE=true ./build.sh                          # 詳細構建
```

---

## 📖 文檔速查

### **主要文檔**
- `INTEGRATED_DEPLOYMENT_README.md` - 完整使用指南
- `docs/development/INTEGRATED_ARCHITECTURE_DESIGN.md` - 架構設計
- `docs/development/INTEGRATED_ARCHITECTURE_FINAL_SUMMARY.md` - 實施總結

### **快速幫助**
```bash
./deploy.sh --help    # 部署幫助
./manage.sh --help    # 管理幫助  
./build.sh --help     # 構建幫助
```

---

## ⚡ 常用操作組合

### **日常開發流程**
```bash
./manage.sh stop      # 停止服務
./deploy.sh           # 重新部署
./manage.sh logs      # 查看啟動日誌
```

### **問題調試流程**
```bash
./manage.sh status    # 檢查狀態
./manage.sh health    # 健康檢查
./manage.sh logs      # 查看日誌
./manage.sh restart   # 重啟服務
```

### **完整重建流程**
```bash
podman-compose -f podman-compose.integrated.yml down --volumes
podman system prune -f
./deploy.sh
```

---

## 🚨 緊急操作

### **立即停止所有服務**
```bash
./manage.sh stop
# 或強制停止
podman stop care-voice-integrated
```

### **快速恢復服務**
```bash
./manage.sh start
# 或重新部署
./deploy.sh
```

### **查看系統資源**
```bash
podman stats --no-stream
netstat -tlnp | grep 8000
```

---

## 📞 獲取幫助

### **內建幫助**
```bash
./manage.sh          # 顯示所有可用命令
./deploy.sh --help   # 部署選項說明
./build.sh --help    # 構建選項說明
```

### **文檔位置**
- 完整指南: `INTEGRATED_DEPLOYMENT_README.md`
- 架構設計: `docs/development/`
- 本快速參考: `QUICK_REFERENCE.md`

---

**💡 提示**: 90% 的使用情況只需要 `./deploy.sh` 和 `./manage.sh status`！