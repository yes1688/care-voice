# Care Voice 專案清理計劃

## ✅ **保留檔案 (新優化系統)**

### **核心執行檔案**
```bash
✅ KEEP: Dockerfile.optimized       # 新的三階段優化編譯
✅ KEEP: build-optimized.sh         # 新的優化編譯腳本
✅ KEEP: start.sh                   # 啟動腳本 (已更新端口)
✅ KEEP: stop.sh                    # 停止腳本
✅ KEEP: nginx-temp.conf            # 統一代理配置 (port 7004)
✅ KEEP: OPTIMIZATION_RECORD.md     # 方案記錄
✅ KEEP: CLEANUP_PLAN.md            # 清理計劃
```

### **專案核心**
```bash
✅ KEEP: frontend/                  # 前端完整目錄
✅ KEEP: backend/                   # 後端完整目錄
✅ KEEP: models/                    # 模型檔案
✅ KEEP: audio-debug/               # 除錯音頻 (如果存在)
```

## 🗑️ **可清理檔案 (舊系統)**

### **舊編譯系統**
```bash
❌ DELETE: build.sh                 # 舊編譯腳本
❌ DELETE: Dockerfile.build-env     # 舊編譯環境
❌ DELETE: Dockerfile.unified       # 舊統一容器
❌ DELETE: Dockerfile.simple        # 舊簡化版本
❌ DELETE: Dockerfile.runtime       # 已被優化版取代
❌ DELETE: docker-compose.unified.yml # 舊容器編排
```

### **舊配置檔案**
```bash
❌ DELETE: nginx-production.conf    # 舊 nginx 配置
```

### **舊容器鏡像** (稍後清理)
```bash
❌ DELETE: care-voice-build-env:latest      # 12.1GB
❌ DELETE: care-voice-final:latest          # 12.1GB
❌ DELETE: care-voice-websocket*:latest     # 舊版本
❌ DELETE: care-voice-opus-fixed:latest     # 舊版本
```

## 🔒 **安全清理順序**

### **階段1: 檔案系統清理**
1. 確認新系統運行正常
2. 備份清理檔案 (建立 deprecated/ 目錄)
3. 移動舊檔案到 deprecated/
4. 測試新系統功能

### **階段2: 容器清理**
1. 停止所有舊容器
2. 移除舊容器鏡像
3. 清理未使用的 volumes

### **階段3: 最終驗證**
1. 測試完整編譯流程
2. 測試服務啟動
3. 測試功能正常

---
**清理原則**: 先移動到 deprecated/ 目錄，確認無問題後再永久刪除