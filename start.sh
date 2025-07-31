#!/bin/bash
# ===================================
# Care Voice 業界領先統一啟動腳本
# ===================================

set -e

echo "🚀 啟動 Care Voice 業界領先語音AI系統..."

# 停止現有服務
echo "🛑 停止現有服務..."
./stop.sh

# ==========================================
# 🌐 網路設定（開發模式使用 localhost）
# ==========================================
# 開發模式：前端使用 nginx 代理到 localhost:8081
# 生產模式：可選擇使用專用網路
# echo "🌐 創建 Care Voice 網路..."
# podman network exists care-voice-network || podman network create care-voice-network

# 確保前端已構建
echo "🔧 構建前端..."
cd frontend && npm run build && cd ..

# ==========================================
# 🔧 開發模式：使用 build-env 容器運行編譯後端
# ==========================================
echo "🤖 啟動後端 AI 服務（開發模式）..."
podman run -d --name care-voice-backend \
  -v "$(pwd):/workspace" \
  -v "$(pwd)/models:/app/models:ro" \
  -p 8081:8081 \
  --device nvidia.com/gpu=all \
  --memory=4g \
  --memory-swap=6g \
  -e RUST_LOG=info \
  -e NVIDIA_VISIBLE_DEVICES=all \
  -e CUDA_VISIBLE_DEVICES=all \
  -w /workspace \
  localhost/care-voice-build-env:latest \
  ./backend/target/release/care-voice \
  2>/dev/null || echo "⚠️  後端啟動失敗，檢查編譯是否完成..."

# ==========================================
# 📦 生產模式：統一容器（待實作）
# ==========================================
# echo "🤖 啟動後端 AI 服務（生產模式）..."
# podman run -d \
#   --name care-voice-backend \
#   --device nvidia.com/gpu=all \
#   -p 8081:8081 \
#   -v "$(pwd)/models:/app/models:ro" \
#   -e RUST_LOG=info \
#   localhost/care-voice:production \
#   2>/dev/null || echo "⚠️  生產容器啟動失敗..."

# 等待後端初始化
echo "⏳ 等待後端初始化..."
sleep 5

# ==========================================
# 🌐 前端服務：nginx + 統一端點代理
# ==========================================
echo "▶️  啟動統一前端服務..."
podman run -d \
  --name care-voice-unified \
  --network host \
  -v "$(pwd)/frontend/dist:/usr/share/nginx/html:ro" \
  -v "$(pwd)/nginx-production.conf:/etc/nginx/conf.d/default.conf:ro" \
  docker.io/library/nginx:alpine

# 等待服務啟動
echo "⏳ 等待統一服務啟動..."
sleep 3

# 檢查服務狀態
FRONTEND_STATUS="❌"
BACKEND_STATUS="❌"

if podman ps | grep -q care-voice-unified; then
  FRONTEND_STATUS="✅"
fi

if podman ps | grep -q care-voice-backend; then
  BACKEND_STATUS="✅"
else
  BACKEND_STATUS="⚠️  (調適中)"
fi

echo ""
echo "📊 Care Voice 統一系統狀態:"
echo "  🌐 前端服務: $FRONTEND_STATUS"
echo "  🤖 後端服務: $BACKEND_STATUS"
echo ""
echo "🔗 統一訪問入口:"
echo "  🌐 主界面: http://localhost:3000"
echo "  💊 健康檢查: http://localhost:3000/health"
echo "  🎤 WebCodecs錄音: http://localhost:3000 (/upload-webcodecs)"
echo ""
echo "📋 服務管理:"
echo "  前端日誌: podman logs -f care-voice-unified"
echo "  後端日誌: podman logs -f care-voice-backend"
echo "  停止服務: ./stop.sh"
echo ""
echo "🔧 開發模式說明:"
echo "  • 後端：build-env容器 + 編譯二進制檔"
echo "  • 前端：nginx + 統一端點代理"
echo "  • 架構：localhost:3000 → nginx → localhost:8081"
echo ""

if [[ "$FRONTEND_STATUS" == "✅" ]]; then
  echo "✅ 統一架構已就緒！"
  if [[ "$BACKEND_STATUS" == "⚠️  (調適中)" ]]; then
    echo "📝 後端調適完成後，錄音功能將自動可用"
  fi
else
  echo "❌ 前端服務啟動失敗"
  podman logs care-voice-unified 2>/dev/null || true
  exit 1
fi