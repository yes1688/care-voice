# 📚 Care Voice 文檔中心

**Care Voice** - 業界領先的 Rust whisper-rs + GPU 加速錄音轉文字系統  
**最新架構**: 整合架構 v1.0 (2025-07-26)

## 🎯 快速導航

### ⚡ **整合架構 (最新)**
最新的統一部署架構文檔

| 文檔 | 描述 | 適用對象 |
|------|------|----------|
| **[快速參考指南](../QUICK_REFERENCE.md)** | 30秒快速上手 | ⭐⭐⭐ 所有用戶 |
| **[整合部署指南](../INTEGRATED_DEPLOYMENT_README.md)** | 完整部署說明 | ⭐⭐ 運維人員 |
| **[架構設計文檔](./development/INTEGRATED_ARCHITECTURE_DESIGN.md)** | 詳細技術設計 | 開發人員 |
| **[實施總結報告](./development/INTEGRATED_ARCHITECTURE_FINAL_SUMMARY.md)** | 完整成果報告 | 管理人員 |

### 👥 用戶指南
適合所有用戶的操作指南和參考資料

| 文檔 | 描述 | 適用對象 |
|------|------|----------|
| **[快速開始](./user-guide/quick-start.md)** | 一鍵部署和基本使用 | 所有用戶 |
| **[安裝指南](./user-guide/installation.md)** | 詳細安裝步驟 | 新用戶 |
| **[故障排除](./user-guide/troubleshooting.md)** | 常見問題解決方案 | 運維人員 |

### 🔧 技術文檔
系統架構和技術實現細節

| 文檔 | 描述 | 適用對象 |
|------|------|----------|
| **[系統狀態](./technical/system-status.md)** | 專案當前狀態和成就 | 專案經理 |
| **[系統架構](./technical/architecture.md)** | 技術設計和實施計劃 | 技術人員 |
| **[GPU 配置](./technical/gpu-configuration.md)** | CUDA 設置和優化 | GPU 專家 |
| **[效能指南](./technical/performance-guide.md)** | 效能測試和調優 | 性能工程師 |
| **[瀏覽器音頻分析](./technical/BROWSER_AUDIO_RECORDING_ANALYSIS.md)** | 瀏覽器音頻格式深度調查 | 音頻專家 |
| **[Opus 後端方案](./technical/OPUS_BACKEND_SOLUTION.md)** | 業界標準音頻處理方案 | 架構師 |
| **[WebM 解決方案](./technical/WEBM_SOLUTION_PLAN.md)** | 音頻格式問題多方案對比 | 技術人員 |

### 💻 開發文檔
開發者和維護者參考資料

| 文檔 | 描述 | 適用對象 |
|------|------|----------|
| **[部署指南](./development/deployment-guide.md)** | 容器部署和監控命令 | DevOps |
| **[環境配置](./development/environment-setup.md)** | CUDA 12.9.1 環境設置 | 開發者 |
| **[API 參考](./development/api-reference.md)** | 接口文檔和範例 | 開發者 |
| **[容器指南](./development/container-guide.md)** | Docker/Podman 使用 | DevOps |
| **[whisper-rs 配置](./development/whisper-rs-setup.md)** | Rust 後端配置 | Rust 開發者 |
| **[實施時間線](./development/IMPLEMENTATION_TIMELINE.md)** | 整合架構實施記錄 | 項目團隊 |
| **[容器現況總覽](./development/CARE_VOICE_CONTAINER_STATUS_OVERVIEW.md)** | 容器狀態分析 | 運維人員 |

## 🚀 核心特色

- ⚡ **GPU 加速**：whisper-rs CUDA 支援，50% 記憶體節省
- 🎤 **即時錄音**：瀏覽器原生 WebM/MP4 錄音
- 📝 **精準轉錄**：whisper-rs 0.14.3 原生 Rust 性能
- 🎯 **智慧摘要**：AI 重點提取
- 🐳 **容器化部署**：Podman GPU 容器，一鍵啟動

## 🎯 依角色導航

### 🆕 新用戶 (整合架構)
[快速參考指南](../QUICK_REFERENCE.md) → [整合部署指南](../INTEGRATED_DEPLOYMENT_README.md) → 運行 `./deploy.sh`

### 👨‍💻 開發者 (整合架構)
[架構設計文檔](./development/INTEGRATED_ARCHITECTURE_DESIGN.md) → [項目架構指南](./development/PROJECT_ARCHITECTURE_GUIDE.md) → [實施總結](./development/INTEGRATED_ARCHITECTURE_FINAL_SUMMARY.md)

### 🔧 運維人員 (整合架構)  
[快速參考指南](../QUICK_REFERENCE.md) → [容器現況總覽](./development/CARE_VOICE_CONTAINER_STATUS_OVERVIEW.md) → [故障排除](./user-guide/troubleshooting.md)

### 📋 管理人員 (整合架構)
[實施總結報告](./development/INTEGRATED_ARCHITECTURE_FINAL_SUMMARY.md) → [實施時間線](./development/IMPLEMENTATION_TIMELINE.md) → [架構設計](./development/INTEGRATED_ARCHITECTURE_DESIGN.md)

### 🔧 傳統架構參考
- **開發者**: [環境配置](./development/environment-setup.md) → [系統架構](./technical/architecture.md)
- **運維人員**: [部署指南](./development/deployment-guide.md) → [容器指南](./development/container-guide.md)
- **性能專家**: [GPU 配置](./technical/gpu-configuration.md) → [效能指南](./technical/performance-guide.md)

## 📊 效能對比

| 指標 | whisper-rs GPU | 標準版 | 改善 |
|------|----------------|---------|------|
| **VRAM 使用** | ~3GB | ~6GB | -50% |
| **啟動時間** | <30s | ~60s | -50% |
| **轉錄速度** | Rust 原生 | Python 依賴 | 更快 |
| **記憶體效率** | 優化 | 標準 | 顯著改善 |

## 🔗 外部資源

- **專案首頁**：[README.md](../README.md)
- **開發規範**：[claude.md](../claude.md) - 角色定義和開發規定
- **GitHub Repository**：[Care Voice](https://github.com/your-repo)

## 📂 新增文檔

**最新更新 (2025-07-25)**：文檔結構重新組織，新增以下專業文檔：

- 🚀 **[部署指南](./development/deployment-guide.md)** - 完整的容器部署和監控指南
- 🛠️ **[環境配置](./development/environment-setup.md)** - CUDA 12.9.1 升級和開發環境設置  
- 📊 **[系統狀態](./technical/system-status.md)** - 專案里程碑和技術成就報告

---

## 📄 文檔維護

**最後更新**：2025-07-25  
**維護狀態**：✅ 當前  
**文檔版本**：v2.1 - 重新組織完成

### 文檔組織變更
- ✅ **claude.md 簡化**：專注於角色定義和開發規定
- ✅ **技術內容分離**：分散到專業的技術文檔中
- ✅ **交叉引用完善**：建立清晰的文檔導航關係
- ✅ **用戶導向優化**：按角色組織文檔內容

如需更新文檔，請參考各文檔頭部的更新說明。