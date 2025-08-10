# Care Voice AI 快速開始

## 一鍵啟動

```bash
# 啟動服務
./start.sh

# 檢查狀態
curl http://localhost:3000/health
```

## 使用方式

1. 🌐 **前端界面**: http://localhost:3000
2. 🤖 **API 服務**: http://localhost:8081
3. 💊 **健康檢查**: http://localhost:3000/health

## 錄音轉文字流程

1. 點擊錄音按鈕
2. 開始語音錄製
3. 停止錄音
4. 自動轉錄顯示結果

## 停止服務

```bash
./stop.sh
```

## 故障排除

- **GPU 不可用**: 檢查 `nvidia-smi`
- **埠口衝突**: 使用 `lsof -i :3000`
- **記憶體不足**: 確保至少 4GB VRAM

需要詳細說明請參考主要 README.md 文檔。