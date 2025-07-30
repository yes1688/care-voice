#!/bin/bash
# ===================================
# Care Voice 業界領先統一停止腳本
# ===================================

echo "⏹️ 停止 Care Voice 業界領先語音AI系統..."

# 停止統一前端服務
echo "🛑 停止統一前端服務..."
podman stop care-voice-unified 2>/dev/null || true
podman rm care-voice-unified 2>/dev/null || true

# 停止後端 AI 服務
echo "🛑 停止後端 AI 服務..."
podman stop care-voice-backend 2>/dev/null || true
podman rm care-voice-backend 2>/dev/null || true

# 停止其他遺留容器
echo "🛑 清理其他 Care Voice 容器..."
podman stop care-voice-frontend 2>/dev/null || true
podman rm care-voice-frontend 2>/dev/null || true

podman stop care-voice-production 2>/dev/null || true
podman rm care-voice-production 2>/dev/null || true

# 清理網路（可選）
echo "🌐 清理網路配置..."
podman network rm care-voice-network 2>/dev/null || true

echo ""
echo "✅ Care Voice 統一系統已完全停止！"
echo ""
echo "📋 系統管理:"
echo "  🚀 重新啟動: ./start.sh"
echo "  📊 檢查狀態: podman ps | grep care-voice"
echo "  📋 查看鏡像: podman images | grep care-voice"