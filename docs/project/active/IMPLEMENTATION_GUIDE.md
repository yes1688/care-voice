# 🛠️ Care Voice 業界領先實施指南

**指南版本**: Implementation v2.0  
**適用對象**: 系統管理員、DevOps 工程師  
**完成時間**: 預估 2-4 小時  

## 🎯 實施目標

徹底修復 Care Voice 系統的 Whisper 模型缺失問題，實現業界領先的企業級 AI 語音轉錄服務，確保 99.9% 可用性和最佳用戶體驗。

## 📋 前置需求

### 系統需求
```bash
# 硬體需求
CPU: 4+ cores
RAM: 8GB+
Storage: 10GB+ available
GPU: NVIDIA GTX 10xx+ (可選，用於加速)

# 軟體需求
OS: Ubuntu 20.04+ / RHEL 8+
Container: Podman/Docker
Network: Internet access for model download
```

### 權限需求
```bash
# 必要權限
sudo access (for container operations)
Network access (for model download)
GPU access (if available)
```

## 🚀 步驟 1: 企業級模型部署

### 1.1 下載 Whisper 模型
```bash
# 使用企業級模型管理系統
cd /mnt/datadrive/MyProjects/care-voice

# 下載基礎模型 (推薦用於快速啟動)
./scripts/enterprise-model-manager.sh download base

# 或下載高品質模型組合 (生產環境推薦)
./scripts/enterprise-model-manager.sh download base small medium
```

**預期輸出**:
```
[ENTERPRISE] 開始企業級模型部署流程
[INFO] 下載嘗試 1/3...
[SUCCESS] ✅ 模型 base 部署成功
[SUCCESS] 元數據創建完成: ./models/metadata/base.json
```

### 1.2 驗證模型完整性
```bash
# 驗證下載的模型
./scripts/enterprise-model-manager.sh verify

# 檢查模型文件
ls -la models/
# 應該看到: ggml-base.bin (約 142MB)
```

### 1.3 部署模型到容器
```bash
# 部署到後端容器
./scripts/enterprise-model-manager.sh deploy care-voice-backend

# 驗證容器內模型
podman exec care-voice-backend ls -la /app/models/
```

**成功指標**:
- ✅ 模型文件存在於容器內
- ✅ 文件大小正確 (base: ~142MB)
- ✅ 權限設置正確

## 🔄 步驟 2: 服務重啟與驗證

### 2.1 重啟統一服務
```bash
# 完整重啟服務
./stop.sh
./start.sh
```

**預期輸出**:
```
📊 Care Voice 統一系統狀態:
  🌐 前端服務: ✅
  🤖 後端服務: ✅  # 應該顯示成功

🔗 統一訪問入口:
  🌐 主界面: http://localhost:3000
  💊 健康檢查: http://localhost:3000/health
```

### 2.2 驗證後端服務啟動
```bash
# 檢查容器狀態
podman ps | grep care-voice

# 檢查後端日誌 (應該無錯誤)
podman logs care-voice-backend | tail -20

# 應該看到類似:
# [INFO] Whisper model loaded successfully
# [INFO] Server listening on 0.0.0.0:8001
```

### 2.3 測試 API 端點
```bash
# 測試健康檢查
curl -s http://localhost:3000/health | jq .

# 預期響應:
{
  "status": "healthy",
  "version": "0.3.0",
  "gpu": {
    "available": true
  },
  "models": [
    {
      "name": "base",
      "loaded": true
    }
  ]
}
```

## 🎵 步驟 3: 功能完整性測試

### 3.1 前端界面測試
```bash
# 訪問主界面
open http://localhost:3000
# 或使用 curl 測試
curl -I http://localhost:3000
```

**檢查項目**:
- ✅ 頁面正常載入
- ✅ 瀏覽器檢測顯示
- ✅ 服務狀態顯示 "健康"
- ✅ 無 JavaScript 錯誤

### 3.2 錄音功能測試
1. **點擊 "開始高品質錄音"**
   - 瀏覽器請求麥克風權限
   - 錄音介面正常顯示

2. **進行錄音測試**
   - 說話 5-10 秒
   - 點擊 "停止錄音"
   - 檢查音頻資訊顯示

3. **AI 轉錄測試**
   - 點擊 "AI 轉錄處理"
   - 等待處理完成 (約 10-30 秒)
   - 檢查轉錄結果

**成功指標**:
- ✅ 錄音功能正常
- ✅ 文件上傳成功
- ✅ AI 轉錄生成結果
- ✅ 摘要功能正常

## 🔧 故障排除指南

### 問題 1: 模型下載失敗
```bash
# 症狀: 下載中斷或速度慢
# 解決方案:
1. 檢查網路連接
2. 使用代理 (如需要)
3. 重試下載 (腳本支援斷點續傳)

# 手動重試
./scripts/enterprise-model-manager.sh download base
```

### 問題 2: 容器啟動失敗
```bash
# 檢查容器日誌
podman logs care-voice-backend

# 常見錯誤:
- "模型文件不存在" → 重新部署模型
- "權限拒絕" → 檢查文件權限
- "端口佔用" → 檢查端口衝突

# 解決方案:
./stop.sh  # 完全停止
podman system prune -f  # 清理
./start.sh  # 重新啟動
```

### 問題 3: 502 Bad Gateway 持續
```bash
# 診斷步驟:
1. 檢查後端服務狀態
podman exec care-voice-backend ss -tlnp | grep 8001

2. 檢查網路連接
podman exec care-voice-unified curl care-voice-backend:8001/health

3. 查看詳細錯誤
podman logs care-voice-backend | grep ERROR
```

### 問題 4: GPU 加速不工作
```bash
# 檢查 GPU 可用性
nvidia-smi

# 檢查容器 GPU 訪問
podman exec care-voice-backend nvidia-smi

# 如果失敗，重啟時添加 GPU 支援
./stop.sh
./start.sh  # 腳本已包含 --device nvidia.com/gpu=all
```

## 📊 效能最佳化

### CPU 最佳化
```bash
# 檢查 CPU 使用
podman stats care-voice-backend

# 調整 Rust 服務並行度 (在容器內)
export RUST_LOG=info
export RAYON_NUM_THREADS=4  # 根據 CPU 核心數調整
```

### 記憶體最佳化
```bash
# 監控記憶體使用
podman exec care-voice-backend free -h

# 如果記憶體不足，考慮:
1. 使用較小的模型 (base 而不是 large)
2. 增加 swap 空間
3. 升級系統記憶體
```

### GPU 最佳化
```bash
# 檢查 GPU 記憶體使用
podman exec care-voice-backend nvidia-smi

# 最佳化設置:
export CUDA_VISIBLE_DEVICES=0
export CUDA_CACHE_DISABLE=0
```

## 🎯 驗收標準

### 功能驗收
- [ ] 前端頁面正常載入 (http://localhost:3000)
- [ ] 健康檢查返回 200 OK
- [ ] 瀏覽器檢測正常工作
- [ ] 錄音功能完整可用
- [ ] AI 轉錄生成正確結果
- [ ] 摘要功能正常工作

### 效能驗收
- [ ] 頁面載入時間 < 2 秒
- [ ] API 響應時間 < 500ms
- [ ] 音頻處理時間 < 30 秒
- [ ] CPU 使用率 < 80%
- [ ] 記憶體使用率 < 70%

### 可靠性驗收
- [ ] 服務連續運行 24 小時無錯誤
- [ ] 重啟後自動恢復
- [ ] 錯誤日誌無 CRITICAL 級別錯誤
- [ ] 健康檢查持續通過

## 📚 參考資源

### 文檔
- [企業架構修復計畫](./ENTERPRISE_ARCHITECTURE_RECOVERY_PLAN.md)
- [系統診斷報告](./DIAGNOSIS_REPORT.md)
- [API 文檔](../README.md)

### 工具腳本
- [企業級模型管理器](../scripts/enterprise-model-manager.sh)
- [統一啟動腳本](../start.sh)
- [統一停止腳本](../stop.sh)

### 監控命令
```bash
# 實時監控
watch -n 1 'podman ps && echo && curl -s http://localhost:3000/health'

# 日誌監控
podman logs -f care-voice-backend

# 效能監控
podman stats care-voice-backend care-voice-unified
```

---

**實施負責人**: DevOps Team  
**技術支援**: Claude Code  
**更新頻率**: 每次部署後更新  
**文檔狀態**: PRODUCTION READY