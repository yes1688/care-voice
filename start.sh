#!/bin/bash
# ===================================
# Care Voice 業界領先統一啟動腳本
# ===================================

set -e

echo "🚀 啟動 Care Voice 業界領先語音AI系統..."

# 停止現有服務
echo "🛑 停止現有服務..."
./stop.sh

# 創建網路（如果不存在）
echo "🌐 創建 Care Voice 網路..."
podman network exists care-voice-network || podman network create care-voice-network

# 確保前端已構建
echo "🔧 構建前端..."
cd frontend && npm run build && cd ..

# 啟動後端服務（調適中）
echo "🤖 啟動後端 AI 服務..."
podman run -d \
  --name care-voice-backend \
  --network care-voice-network \
  --device nvidia.com/gpu=all \
  -p 8081:8001 \
  -v "$(pwd)/models:/app/models:ro" \
  -e RUST_LOG=info \
  -e NVIDIA_VISIBLE_DEVICES=all \
  -e CUDA_VISIBLE_DEVICES=all \
  localhost/care-voice:unified \
  2>/dev/null || echo "⚠️  後端啟動失敗（調適中），繼續啟動前端..."

# 等待後端初始化
echo "⏳ 等待後端初始化..."
sleep 5

# 啟動統一前端服务
echo "▶️  啟動統一前端服務..."
podman run -d \
  --name care-voice-unified \
  --network care-voice-network \
  -p 3000:8000 \
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
echo "  🎤 錄音功能: http://localhost:3000 (需後端完成)"
echo ""
echo "📋 服務管理:"
echo "  前端日誌: podman logs -f care-voice-unified"
echo "  後端日誌: podman logs -f care-voice-backend"
echo "  停止服務: ./stop.sh"
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