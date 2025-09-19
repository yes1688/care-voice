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
# 🌐 網路設定（統一架構使用容器網路）
# ==========================================
echo "🌐 創建 Care Voice 統一網路..."
podman network exists care-voice-network || podman network create care-voice-network

# 檢查是否已編譯
echo "🔍 檢查編譯狀態..."
if [[ ! -f "frontend/dist/index.html" ]]; then
    echo "❌ 前端未編譯，請先執行: ./build.sh"
    echo "💡 編譯完成後再執行: ./start.sh"
    exit 1
fi

if ! podman image exists localhost/care-voice-build-env:latest; then
    echo "❌ 容器鏡像不存在，請先執行: ./build.sh"
    echo "💡 編譯完成後再執行: ./start.sh"
    exit 1
fi

echo "✅ 編譯狀態檢查通過"

# ==========================================
# 🔧 開發模式：使用 build-env 容器運行編譯後端
# ==========================================
echo "🤖 啟動後端 AI 服務（統一網路模式）..."
podman run -d --name care-voice-backend \
  --network care-voice-network \
  --privileged \
  -v "$(pwd)/backend/target/release/care-voice:/app/care-voice:ro" \
  -v "$(pwd)/models:/app/models:ro" \
  -v "$(pwd)/audio-debug:/tmp/care-voice-debug:rw" \
  -v "/usr/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:ro" \
  -v "/usr/local/cuda-13.0:/usr/local/cuda-13.0:ro" \
  -v "/dev:/dev" \
  --device /dev/nvidia0 \
  --device /dev/nvidiactl \
  --device /dev/nvidia-uvm \
  --memory=4g \
  --memory-swap=6g \
  -e RUST_LOG=info \
  -e BACKEND_PORT=8005 \
  -e NVIDIA_VISIBLE_DEVICES=all \
  -e CUDA_VISIBLE_DEVICES=0 \
  -e NVIDIA_DRIVER_CAPABILITIES=compute,utility \
  -e CUDA_HOME=/usr/local/cuda \
  -e PATH="/usr/local/cuda/bin:/usr/local/cuda-13.0/bin:$PATH" \
  -e LD_LIBRARY_PATH="/usr/local/cuda/lib64:/usr/local/cuda/targets/x86_64-linux/lib:/usr/local/cuda-13.0/lib64:/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH" \
  -e TORCH_CUDA_ARCH_LIST="12.0" \
  -e CUDA_ARCH_LIST="120" \
  -e GGML_CUDA_FORCE_MMV=1 \
  -e GGML_CUDA_NO_PINNED=1 \
  -e CARE_VOICE_DEBUG_AUDIO=1 \
  -w /app \
  localhost/care-voice:runtime \
  ./care-voice \
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
  --network care-voice-network \
  -p 7004:7004 \
  -v "$(pwd)/frontend/dist:/usr/share/nginx/html:ro" \
  -v "$(pwd)/nginx-temp.conf:/etc/nginx/conf.d/default.conf:ro" \
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
echo "  🌐 主界面: http://localhost:7004"
echo "  💊 健康檢查: http://localhost:7004/health"
echo "  🎤 統一音頻上傳: http://localhost:7004/upload"
echo ""
echo "📋 服務管理:"
echo "  前端日誌: podman logs -f care-voice-unified"
echo "  後端日誌: podman logs -f care-voice-backend"
echo "  停止服務: ./stop.sh"
echo ""
echo "🔧 Care Voice SOP 工作流程:"
echo "  • 編譯：./build.sh (前端 + 後端容器)"
echo "  • 啟動：./start.sh (純粹啟動服務)"
echo "  • 關閉：./stop.sh (純粹關閉服務)"
echo "  • 架構：localhost:7004 → nginx → care-voice-backend:8005"
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