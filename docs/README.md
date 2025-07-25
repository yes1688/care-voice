# 📚 Care Voice 文檔中心

**Care Voice** - 業界領先的 Rust whisper-rs + GPU 加速錄音轉文字系統

## 🎯 快速導航

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
| **[系統架構](./technical/architecture.md)** | 技術設計和實施計劃 | 技術人員 |
| **[GPU 配置](./technical/gpu-configuration.md)** | CUDA 設置和優化 | GPU 專家 |
| **[效能指南](./technical/performance-guide.md)** | 效能測試和調優 | 性能工程師 |

### 💻 開發文檔
開發者和維護者參考資料

| 文檔 | 描述 | 適用對象 |
|------|------|----------|
| **[API 參考](./development/api-reference.md)** | 接口文檔和範例 | 開發者 |
| **[容器指南](./development/container-guide.md)** | Docker/Podman 使用 | DevOps |
| **[whisper-rs 配置](./development/whisper-rs-setup.md)** | Rust 後端配置 | Rust 開發者 |

## 🚀 核心特色

- ⚡ **GPU 加速**：whisper-rs CUDA 支援，50% 記憶體節省
- 🎤 **即時錄音**：瀏覽器原生 WebM/MP4 錄音
- 📝 **精準轉錄**：whisper-rs 0.14.3 原生 Rust 性能
- 🎯 **智慧摘要**：AI 重點提取
- 🐳 **容器化部署**：Podman GPU 容器，一鍵啟動

## 🎯 依角色導航

### 新用戶
`README.md` → [快速開始](./user-guide/quick-start.md) → [安裝指南](./user-guide/installation.md)

### 開發者
[快速開始](./user-guide/quick-start.md) → [系統架構](./technical/architecture.md) → [API 參考](./development/api-reference.md)

### 運維人員
[快速開始](./user-guide/quick-start.md) → [故障排除](./user-guide/troubleshooting.md) → [容器指南](./development/container-guide.md)

### 性能專家
[GPU 配置](./technical/gpu-configuration.md) → [效能指南](./technical/performance-guide.md) → [系統架構](./technical/architecture.md)

## 📊 效能對比

| 指標 | whisper-rs GPU | 標準版 | 改善 |
|------|----------------|---------|------|
| **VRAM 使用** | ~3GB | ~6GB | -50% |
| **啟動時間** | <30s | ~60s | -50% |
| **轉錄速度** | Rust 原生 | Python 依賴 | 更快 |
| **記憶體效率** | 優化 | 標準 | 顯著改善 |

## 🔗 外部資源

- **專案首頁**：[README.md](../README.md)
- **系統配置**：[claude.md](../claude.md)
- **GitHub Repository**：[Care Voice](https://github.com/your-repo)

---

## 📄 文檔維護

**最後更新**：2025-07-25  
**維護狀態**：✅ 當前  
**文檔版本**：v2.0

如需更新文檔，請參考各文檔頭部的更新說明。