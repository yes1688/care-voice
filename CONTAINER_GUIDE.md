# Care Voice 容器架構指南

## 🏗️ 當前容器架構 (2025-08)

### 主要容器配置

#### 1. **Dockerfile.build-env** ✅ 編譯環境
- **用途**: RTX 50 系列優化的編譯環境
- **大小**: ~10.4GB (包含完整編譯工具鏈)
- **更新**: 2025-08-10 (RTX 5070 Ti 支援)
- **使用**: `podman build -f Dockerfile.build-env -t care-voice-build-env:latest .`

#### 2. **Dockerfile.runtime** ✅ 運行時容器
- **用途**: 輕量級 CUDA runtime 容器
- **大小**: ~3.98GB (僅 runtime 依賴)
- **基礎**: `nvidia/cuda:12.9.1-runtime-ubuntu24.04`
- **使用**: 生產環境運行

#### 3. **Dockerfile.unified** ✅ 統一容器 (主要)
- **用途**: 完整的統一部署容器 (前端+後端+nginx)
- **基礎**: `nvidia/cuda:12.9.1-runtime-ubuntu24.04`
- **更新**: 2025-08-09 (最新統一架構)
- **使用**: 單容器部署，包含完整功能

### 測試和輔助容器

#### 4. **Dockerfile.simple** 🧪 純前端測試
- **用途**: 純前端 nginx 容器，用於測試
- **大小**: ~55.6MB (nginx alpine)
- **使用**: 前端功能測試

### 過時容器 (建議清理)

#### 5. **Dockerfile.unified-modern** ❌ 過時
- **狀態**: 舊版統一容器
- **日期**: 2025-07-29 (已過時)
- **問題**: 使用 devel 鏡像，體積過大
- **建議**: 刪除，已被 Dockerfile.unified 取代

## 🚀 標準工作流程

### 開發階段
```bash
# 1. 編譯 (使用輕量編譯環境)
./build.sh

# 2. 啟動 (使用 runtime 容器)
./start.sh
```

### 生產部署
```bash
# 使用統一容器
podman build -f Dockerfile.unified -t care-voice:unified .
podman run --gpus all -p 3000:3000 care-voice:unified
```

## 📊 容器大小對比

| 容器 | 大小 | 用途 | 狀態 |
|------|------|------|------|
| build-env | ~10.4GB | 編譯 | ✅ 活躍 |
| runtime | ~3.98GB | 運行 | ✅ 活躍 |
| unified | ~4.5GB | 統一部署 | ✅ 活躍 |
| simple | ~55.6MB | 前端測試 | 🧪 測試 |
| unified-modern | ~6GB | 舊版統一 | ❌ 過時 |

## 💡 最佳實踐

1. **開發**: 使用 build-env + runtime 分離架構
2. **測試**: 使用 simple 容器測試前端
3. **生產**: 使用 unified 容器統一部署
4. **清理**: 定期清理過時的容器配置

---

*最後更新: 2025-08-10*