#!/bin/bash
# Care Voice Opus 完整支援構建腳本

set -e

echo "🐳 開始構建 Care Voice Opus 完整支援容器..."

# 確保在正確的目錄
cd /mnt/datadrive/MyProjects/care-voice/backend

# 顯示當前修改的檔案
echo "📝 檢查修改過的檔案:"
ls -la src/opus_decoder.rs src/audio_decoder.rs Cargo.toml

# 構建完整版鏡像
echo ""
echo "📦 構建鏡像: care-voice:opus-complete-v1"
podman build \
  -f Dockerfile.opus-complete \
  -t care-voice:opus-complete-v1 \
  --no-cache \
  .

echo "✅ 構建完成！"

# 顯示鏡像資訊
echo ""
echo "📊 鏡像資訊:"
podman images | grep "care-voice.*opus"

echo ""
echo "🔄 停止舊的測試容器 (如果存在):"
podman stop care-voice-opus-test 2>/dev/null || echo "舊容器不存在或已停止"
podman rm care-voice-opus-test 2>/dev/null || echo "舊容器已清理"

echo ""
echo "🚀 啟動完整 Opus 支援容器:"
podman run -d --name care-voice-opus-complete \
  --device nvidia.com/gpu=all \
  -p 8003:8000 \
  care-voice:opus-complete-v1

echo ""
echo "⏳ 等待服務啟動..."
sleep 10

echo ""
echo "🔍 測試健康檢查:"
curl http://localhost:8003/health

echo ""
echo "✅ 完整 Opus 支援部署完成！"
echo "📍 服務地址: http://localhost:8003"
echo "🎵 現在支援完整的 WebM-Opus 和 OGG-Opus 解碼"