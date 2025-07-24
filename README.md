# 錄音轉文字系統 (Care Voice)

基於 Rust + Solid.js + Gemini API + Podman 的簡單錄音轉文字應用程式。

## 功能特色

- 🎤 **瀏覽器錄音**：支援 WebM/MP4 格式
- 📝 **完整逐字稿**：使用 Gemini 2.5 Flash 轉錄音頻
- 🎯 **關懷重點摘要**：AI 自動提取重要資訊
- 🐳 **Podman 容器化**：一鍵部署和運行
- 🌐 **現代 Web 技術**：Rust 後端 + Solid.js 前端

## 快速開始

### 前置需求

- Podman 4.0+
- podman-compose
- Gemini API 金鑰

### 安裝 podman-compose (如果未安裝)

```bash
# 使用 pip 安裝
pip3 install podman-compose

# 或使用套件管理器
dnf install podman-compose    # Fedora/RHEL
apt install podman-compose    # Ubuntu (較新版本)
```

### 設置 API 金鑰

1. 編輯 `.env` 檔案
2. 將 `your_gemini_api_key_here` 替換為你的 Gemini API 金鑰

```bash
GEMINI_API_KEY=你的_gemini_api_金鑰
VITE_API_URL=http://localhost:8000
```

### 啟動應用程式

```bash
# 建構並啟動所有服務
podman-compose up --build

# 或在背景運行
podman-compose up --build -d
```

### 訪問應用程式

- **前端界面**: http://localhost:3000
- **後端 API**: http://localhost:8000
- **健康檢查**: http://localhost:8000/health

## 使用方式

1. 點擊「🎤 開始錄音」按鈕
2. 說話並等待錄音完成
3. 點擊「⏹️ 停止錄音」
4. 點擊「📤 轉換為文字」上傳音頻
5. 等待 AI 處理並查看結果：
   - 📝 完整逐字稿
   - 🎯 關懷重點摘要

## 技術架構

### 後端 (Rust + Axum)
- **檔案**: `backend/src/main.rs`
- **功能**: multipart 檔案上傳、Gemini 2.5 Flash API 整合
- **端口**: 8000

### 前端 (Solid.js)
- **檔案**: `frontend/src/App.tsx`
- **功能**: MediaRecorder 錄音、檔案上傳、結果顯示
- **端口**: 3000

### 容器化 (Podman)
- **多階段建構**：優化容器大小
- **健康檢查**：確保服務可用性
- **無 daemon 運行**：Podman 特色

## 開發指令

```bash
# 查看服務狀態
podman-compose ps

# 查看日誌
podman-compose logs

# 重新建構服務
podman-compose build

# 停止服務
podman-compose down

# 清理容器和數據
podman-compose down -v
podman system prune -f
```

## API 端點

- `GET /health` - 健康檢查
- `POST /upload` - 音頻檔案上傳 (multipart/form-data)

## 疑難排解

### 錄音權限問題
- 確保瀏覽器已授予麥克風權限
- 使用 HTTPS 或 localhost 訪問

### API 連接失敗
- 檢查 Gemini API 金鑰是否正確設置
- 確認後端容器正常運行

### 容器建構失敗
- 檢查 Podman 版本是否支援
- 嘗試清理並重新建構

## 專案結構

```
care-voice/
├── backend/           # Rust 後端
│   ├── src/main.rs   # 主程式
│   ├── Cargo.toml    # 依賴配置
│   └── Dockerfile    # 後端容器
├── frontend/          # Solid.js 前端
│   ├── src/App.tsx   # 主組件
│   ├── package.json  # 前端依賴
│   ├── nginx.conf    # Nginx 配置
│   └── Dockerfile    # 前端容器
├── docker-compose.yml # 服務編排
├── .env              # 環境變數
└── README.md         # 說明文檔
```

## 授權

MIT License