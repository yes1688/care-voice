#!/bin/bash
# ===================================
# Care Voice 業界領先統一停止腳本
# ===================================

echo "⏹️ 停止 Care Voice 業界領先語音AI系統..."

# ==========================================
# 🌐 停止前端服務（nginx + 統一端點代理）
# ==========================================
echo "🛑 停止統一前端服務..."
podman stop care-voice-unified 2>/dev/null || true
podman rm care-voice-unified 2>/dev/null || true

# ==========================================
# 🤖 停止後端服務（開發/生產模式通用）
# ==========================================
echo "🛑 停止後端 AI 服務..."
# 開發模式：build-env 容器運行的後端  
podman stop care-voice-backend 2>/dev/null || true
podman rm care-voice-backend 2>/dev/null || true
# 清理可能的舊版後端容器
podman ps -a --format "{{.Names}}" | grep "care-voice-backend" | xargs -r podman rm -f 2>/dev/null || true

# ==========================================
# 🧹 清理遺留容器和資源
# ==========================================
echo "🛑 清理其他 Care Voice 容器..."
# 舊版容器清理
podman stop care-voice-frontend 2>/dev/null || true
podman rm care-voice-frontend 2>/dev/null || true

podman stop care-voice-production 2>/dev/null || true
podman rm care-voice-production 2>/dev/null || true

# 網路清理（生產模式可能需要）
echo "🌐 清理網路配置..."
podman network rm care-voice-network 2>/dev/null || true

echo ""
echo "✅ Care Voice 統一系統已完全停止！"
echo ""
echo "📋 系統管理:"
echo "  🚀 重新啟動: ./start.sh"
echo "  📊 檢查狀態: podman ps | grep care-voice"
echo "  📋 查看鏡像: podman images | grep care-voice"
echo ""
echo "🔧 開發環境說明:"
echo "  • build-env 容器保持運行用於編譯"
echo "  • 生產容器將在未來實作"
echo "  • 統一端點架構已就緒"