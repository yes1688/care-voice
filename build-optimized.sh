#!/bin/bash
# ===================================
# Care Voice 優化編譯腳本
# 利用 Docker layer caching 加速編譯
# ===================================

set -e

echo "🚀 Care Voice 優化編譯開始..."
echo "========================================"

# 記錄開始時間
START_TIME=$(date +%s)

# 編譯前端 (保持不變)
echo "📦 編譯前端 (SolidJS + Vite)..."
cd frontend
if [[ ! -f package.json ]]; then
    echo "❌ frontend/package.json 不存在"
    exit 1
fi

echo "🔧 安裝前端依賴..."
npm ci --include=dev

echo "🚀 開始前端編譯..."
npm run build

if [[ ! -f dist/index.html ]]; then
    echo "❌ 前端編譯失敗"
    exit 1
fi

echo "✅ 前端編譯完成"
cd ..

# 停止現有服務
echo "🛑 停止現有服務..."
./stop.sh 2>/dev/null || true

# 🎯 使用優化的 Dockerfile 編譯後端
echo ""
echo "🦀 開始優化後端編譯..."
echo "🎯 使用分階段編譯：依賴套件 vs 原始碼"

# 編譯優化後端容器
echo "🚀 建置優化後端鏡像..."
podman build -f Dockerfile.optimized -t care-voice:optimized . || {
    echo "❌ 優化編譯失敗"
    echo "💡 檢查編譯日誌中的錯誤信息"
    exit 1
}

# 測試從優化容器中複製二進制檔案
echo "📦 從容器中複製編譯結果..."
podman create --name temp-container care-voice:optimized
podman cp temp-container:/app/care-voice backend/target/release/care-voice
podman rm temp-container

# 驗證複製結果
if [[ ! -f backend/target/release/care-voice ]]; then
    echo "❌ 後端文件複製失敗：找不到 care-voice 二進制文件"
    exit 1
fi

echo "✅ 後端二進制文件就緒"
echo "📊 後端文件資訊:"
ls -la backend/target/release/care-voice
du -sh backend/target/release/care-voice

# 計算編譯時間
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

echo ""
echo "✅ 優化編譯完成！"
echo "========================================"
echo "📋 編譯產出:"
echo "  🌐 前端: frontend/dist/"
echo "  🦀 後端: backend/target/release/care-voice"
echo "⏱️  總編譯時間: ${MINUTES}分${SECONDS}秒"
echo ""
echo "🚀 下一步: ./start.sh 啟動服務"
echo ""
echo "💡 優化說明:"
echo "  - 依賴套件編譯會被 Docker 快取"
echo "  - 下次只需重新編譯您的程式碼"
echo "  - 預期下次編譯時間: 2-5分鐘"
echo ""