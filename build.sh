#!/bin/bash
# ===================================
# Care Voice 統一編譯腳本
# 職責：編譯前端 + 構建容器
# ===================================

set -e

echo "🏗️ Care Voice 統一編譯開始..."
echo "========================================"

# 編譯前端
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
echo "📊 前端編譯產出:"
du -sh dist/
ls -la dist/

cd ..

# 重新編譯後端（RTX 50 系列支援）
echo ""
echo "🦀 重新編譯後端 (RTX 5070 Ti 優化)..."
echo "🔧 強制重新編譯 whisper-rs 支援 compute capability 12.0..."

# 確保目錄存在
mkdir -p backend/target/release/

# 停止並移除現有容器（如果存在）
echo "🛑 停止現有服務..."
./stop.sh 2>/dev/null || true

# 重新編譯支援 RTX 50 系列
echo "🚀 重新編譯完整項目..."
podman run --rm \
  -v "$(pwd)/backend:/workspace" \
  -w /workspace \
  localhost/care-voice-build-env:latest \
  /usr/local/bin/compile.sh || {
    echo "❌ RTX 50 系列重新編譯失敗"
    echo "💡 檢查編譯日誌中的錯誤信息"
    exit 1
}

# 驗證複製結果
if [[ ! -f backend/target/release/care-voice ]]; then
    echo "❌ 後端文件複製失敗：找不到 care-voice 二進制文件"
    exit 1
fi

echo "✅ 後端二進制文件就緒"
echo "📊 後端文件資訊:"
ls -la backend/target/release/care-voice
du -sh backend/target/release/care-voice

echo ""
echo "✅ 統一編譯完成！"
echo "========================================"
echo "📋 編譯產出:"
echo "  🌐 前端: frontend/dist/"
echo "  🦀 後端: backend/target/release/care-voice"
echo ""
echo "🚀 下一步: ./start.sh 啟動服務"
echo ""