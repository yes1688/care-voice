# 🏗️ Care Voice 專案架構完整指南

**文檔版本**: v1.0  
**創建日期**: 2025-07-26  
**更新時間**: 22:45  
**目的**: 詳細說明專案運作方式、前端編譯流程、nginx 容器配置  

---

## 📊 專案整體架構

### **技術棧總覽**

| 層級 | 技術選擇 | 用途 | 備註 |
|------|----------|------|------|
| **前端** | Vite + SolidJS + TypeScript | 音頻錄製與轉錄界面 | 非 React，是 SolidJS |
| **反向代理** | nginx | 靜態文件服務 + API 代理 | 多種配置模式 |
| **後端** | Rust + whisper-rs | 語音轉錄與 AI 摘要 | GPU 加速 |
| **容器化** | Podman / Docker | 服務部署與管理 | 支援 GPU 直通 |

---

## 🎨 前端架構詳解

### **技術棧: Vite + SolidJS + TypeScript**

#### **為什麼是 SolidJS 而非 React？**
```typescript
// SolidJS 的響應式語法 (非 React hooks)
import { createSignal, Show } from 'solid-js';

function App() {
  const [isRecording, setIsRecording] = createSignal(false);
  // 編譯時優化，運行時性能更佳
}
```

#### **專案結構**
```
frontend/
├── src/
│   ├── App.tsx           # 主應用組件 (SolidJS)
│   └── index.tsx         # 應用入口
├── dist/                 # 編譯輸出目錄
├── package.json          # 依賴管理
├── vite.config.ts        # Vite 配置
├── Dockerfile           # 前端容器化配置
├── nginx.conf           # 前端專用 nginx 配置
└── nginx-standalone.conf # 獨立 nginx 配置
```

### **前端編譯流程**

#### **1. 開發模式**
```bash
cd frontend/
npm install              # 安裝依賴
npm run dev             # 啟動開發服務器 (3000端口)
```
- **Vite 特性**: 快速 HMR (熱模組替換)
- **SolidJS 優勢**: 編譯時優化，運行時性能優秀

#### **2. 生產編譯**
```bash
npm run build           # 編譯到 dist/ 目錄
```
**編譯產出**:
```
dist/
├── index.html          # SPA 入口頁面
├── assets/
│   ├── index-[hash].js # 編譯後的 JavaScript
│   └── index-[hash].css # 樣式文件
└── [其他靜態資源]
```

#### **3. 容器化編譯** (Multi-stage Build)
```dockerfile
# Stage 1: Build stage
FROM docker.io/node:20-slim AS builder
WORKDIR /app
COPY package.json ./
RUN npm install
COPY . .
RUN npm run build

# Stage 2: Production stage  
FROM docker.io/nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 3000
```

---

## 🌐 nginx 容器配置詳解

### **配置模式總覽**

| 配置文件 | 用途 | 端口 | 容器名稱 |
|----------|------|------|----------|
| `frontend/nginx.conf` | 前端專用 nginx | 3000 | frontend 容器 |
| `unified-nginx.conf` | 統一前後端 nginx | 8001 | 整合容器 |
| `nginx-standalone.conf` | 獨立部署配置 | 自定義 | 靈活配置 |

### **模式一: 分離式部署** (前端專用 nginx)

#### **容器運行方式**
```bash
# 前端容器 (nginx + 靜態文件)
podman run -d -p 3000:3000 care-voice-frontend

# 後端容器 (Rust 服務)
podman run -d -p 8000:8000 care-voice-backend
```

#### **nginx.conf 關鍵配置**
```nginx
server {
    listen 3000;
    root /usr/share/nginx/html;
    index index.html;
    
    # SPA 路由支援
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    # API 代理到後端
    location /api/ {
        proxy_pass http://backend:8000/;  # Docker 網路內部通訊
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # CORS 處理
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
    }
}
```

### **模式二: 統一式部署** (整合 nginx)

#### **容器運行方式**
```bash
# 統一容器 (nginx + 前端 + 後端)
podman run -d -p 8001:8001 care-voice-unified
```

#### **unified-nginx.conf 關鍵配置**
```nginx
server {
    listen 8001;
    root /usr/share/nginx/html;
    
    # API 代理到本地後端
    location /api/ {
        rewrite ^/api/(.*)$ /$1 break;  # 移除 /api 前綴
        proxy_pass http://localhost:8000;  # 同容器內後端服務
        client_max_body_size 50M;         # 支援大音頻文件
        proxy_connect_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # 健康檢查直接代理
    location /health {
        proxy_pass http://localhost:8000/health;
    }
    
    # 靜態資源快取
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

---

## 🚀 部署架構分析

### **Docker Compose 配置**

#### **1. 分離式部署** (`docker-compose.yml`)
```yaml
version: '3.8'
services:
  backend:
    build: ./backend-sync
    ports: ["8000:8000"]
    
  frontend:
    image: localhost/care-voice_frontend_unified:latest
    ports: ["3000:3000"]
    depends_on:
      backend:
        condition: service_healthy
```

**優勢**:
- ✅ 前後端獨立部署和擴展
- ✅ 開發調試方便
- ✅ 故障隔離

**劣勢**:
- ❌ 需要管理兩個容器
- ❌ 網路配置較複雜

#### **2. 統一式部署** (`docker-compose.unified.yml`)
```yaml
version: '3.8'
services:
  care-voice-unified:
    build:
      context: .
      dockerfile: Dockerfile.unified  # 注意：此文件不存在
    ports: ["8000:8000"]
```

**優勢**:
- ✅ 單一容器管理
- ✅ 部署簡單
- ✅ 資源使用更高效

**劣勢**:
- ❌ 前後端耦合
- ❌ 擴展性較差

### **當前 Podman 實際運行狀況**

#### **運行中容器**
```bash
# 檢查當前容器
podman ps

# 實際運行狀況 (基於分析)
care-voice-ultimate     8001端口   # 原版統一容器
care-voice-opus-test    8002端口   # Opus 測試容器
```

#### **容器實際配置推測**
- **8001端口容器**: 使用 `unified-nginx.conf` 配置
- **nginx 運行方式**: 容器內整合 (前端靜態文件 + API 代理)
- **前端編譯位置**: 編譯後的 `dist/` 內容在容器的 `/usr/share/nginx/html`

---

## 🎵 音頻處理流程詳解

### **完整數據流**

```
用戶瀏覽器 
    ↓ (錄音)
📱 SolidJS 前端應用 (優先 WAV 格式)
    ↓ (HTTP POST /api/upload)
🌐 nginx 反向代理 (移除 /api 前綴)
    ↓ (轉發到 http://localhost:8000/upload)
🦀 Rust 後端服務 (whisper-rs)
    ↓ (GPU 加速轉錄)
🤖 AI 處理 (轉錄 + 摘要)
    ↓ (JSON 回應)
📱 前端顯示結果
```

### **前端音頻錄製邏輯**

#### **格式優先序策略**
```typescript
// 1. 優先使用 WAV 格式 (直接支援)
const wavFormats = ['audio/wav', 'audio/wave', 'audio/x-wav'];

// 2. 降級到 WebM/OGG (需要伺服器轉換)  
const fallbackFormats = ['audio/webm', 'audio/ogg'];

// 3. 智能格式選擇
for (const format of wavFormats) {
  if (MediaRecorder.isTypeSupported(format)) {
    options.mimeType = format;
    console.log(`✅ 使用 WAV 格式: ${format}`);
    break;
  }
}
```

#### **API 請求處理**
```typescript
// 使用相對路徑，通過 nginx 代理
const response = await fetch('/api/upload', {
  method: 'POST',
  body: formData,  // 包含音頻 blob
});
```

### **nginx 代理配置**

#### **API 路徑重寫**
```nginx
# 前端請求: /api/upload  
# nginx 處理: 移除 /api 前綴
location /api/ {
    rewrite ^/api/(.*)$ /$1 break;
    proxy_pass http://localhost:8000;
}
# 實際後端接收: /upload
```

#### **文件上傳優化**
```nginx
# 支援大音頻文件 (最大 50MB)
client_max_body_size 50M;
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;
```

---

## 🔧 開發與部署指南

### **本地開發模式**

#### **前端開發**
```bash
cd frontend/
npm install
npm run dev        # http://localhost:3000
```

#### **後端開發**
```bash
cd backend/
cargo run          # http://localhost:8000
```

#### **跨域問題解決**
- **開發模式**: Vite 開發服務器需要配置 proxy
- **生產模式**: nginx 配置 CORS headers

### **容器構建流程**

#### **前端容器**
```bash
cd frontend/
podman build -t care-voice-frontend .
podman run -d -p 3000:3000 care-voice-frontend
```

#### **統一容器** (如果有 Dockerfile.unified)
```bash
# 需要包含前端編譯步驟
podman build -f Dockerfile.unified -t care-voice-unified .
podman run -d -p 8001:8001 care-voice-unified
```

---

## 🎯 關鍵設計特點

### **前端架構優勢**

#### **SolidJS 選擇原因**
- **性能**: 編譯時優化，運行時開銷極小
- **響應式**: 細粒度更新，僅重新渲染變化部分
- **體積**: 比 React 更小的打包體積
- **TypeScript**: 原生 TypeScript 支援

#### **音頻錄製策略**
- **格式智能選擇**: WAV > WebM > OGG 優先序
- **錯誤處理**: 友善的用戶提示
- **性能優化**: 格式檢測避免不必要轉換

### **nginx 配置特色**

#### **SPA 支援**
```nginx
# 客戶端路由支援
location / {
    try_files $uri $uri/ /index.html;
}
```

#### **性能優化**
```nginx
# 靜態資源長期快取
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# HTML 文件防止快取
location ~* \.html$ {
    add_header Cache-Control "no-cache, no-store, must-revalidate";
}
```

#### **安全配置**
```nginx
# 安全 headers
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header Content-Security-Policy "default-src 'self'..." always;
```

---

## 🔍 故障排除指南

### **常見問題診斷**

#### **前端無法連接後端**
```bash
# 1. 檢查容器狀態
podman ps | grep care-voice

# 2. 檢查端口監聽
netstat -tlnp | grep -E ":(3000|8000|8001)"

# 3. 檢查 nginx 代理配置
podman exec -it <container> cat /etc/nginx/nginx.conf
```

#### **音頻上傳失敗**
```bash
# 檢查文件大小限制
curl -X POST -F "audio=@test.wav" http://localhost:8001/api/upload

# 檢查 nginx 錯誤日誌
podman logs <container> | grep error
```

#### **前端編譯問題**
```bash
# 清理並重新安裝
cd frontend/
rm -rf node_modules/ dist/
npm install
npm run build
```

---

## 📚 相關文檔索引

### **技術文檔**
- [CARE_VOICE_CONTAINER_STATUS_OVERVIEW.md](./CARE_VOICE_CONTAINER_STATUS_OVERVIEW.md) - 容器現狀總覽
- [OPUS_IMPLEMENTATION_STATUS.md](../../OPUS_IMPLEMENTATION_STATUS.md) - Opus 音頻支援狀態
- [ANALYSIS_UPDATE_SUMMARY.md](./ANALYSIS_UPDATE_SUMMARY.md) - 問題診斷總結

### **部署相關**
- [deployment-guide.md](./deployment-guide.md) - 生產環境部署指南
- [environment-setup.md](./environment-setup.md) - 開發環境設置

### **架構設計**
- [AUDIO_PROCESSING_ARCHITECTURE.md](../technical/AUDIO_PROCESSING_ARCHITECTURE.md) - 音頻處理架構
- [architecture.md](../technical/architecture.md) - 系統架構總覽

---

## 💡 總結

**Care Voice 專案特點**:
- **現代前端**: Vite + SolidJS 提供優秀的開發體驗和運行性能
- **靈活部署**: 支援分離式和統一式兩種部署模式
- **智能音頻**: 自動格式檢測和降級策略
- **容器化**: 完整的 Docker/Podman 支援

**當前運行狀況**:
- **8001端口**: 統一式部署，nginx + 前端 + 後端整合
- **前端技術**: SolidJS (非React) + TypeScript
- **nginx 角色**: 靜態文件服務 + API 反向代理

**關鍵理解**:
- nginx 運行在容器內，不是獨立的 nginx 容器
- 前端編譯後整合到統一容器中
- API 請求通過 nginx 代理到同容器內的後端服務