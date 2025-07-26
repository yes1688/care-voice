#!/bin/bash
# Care Voice Opus 支援構建腳本

set -e

echo "🐳 開始構建 Care Voice Opus 支援容器..."

# 構建新鏡像
echo "📦 構建鏡像: care-voice:opus-support-v1"
podman build \
  -f Dockerfile.opus-support \
  -t care-voice:opus-support-v1 \
  --no-cache \
  .

echo "✅ 構建完成！"

# 顯示鏡像資訊
echo "📊 鏡像資訊:"
podman images | grep "care-voice.*opus"

echo ""
echo "🚀 可以使用以下命令運行:"
echo "podman run -d --name care-voice-opus-test --device nvidia.com/gpu=all -p 8002:8000 care-voice:opus-support-v1"

echo ""
echo "🔍 測試健康檢查:"
echo "curl http://localhost:8002/health"