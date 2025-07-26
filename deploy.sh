#!/bin/bash
# =======================================================
# Care Voice 統一部署腳本
# 功能: 一鍵部署統一 multi-stage 服務
# =======================================================

set -e

echo "🚀 Care Voice 統一架構部署"
echo "=========================="

# 檢查 podman-compose
if ! command -v podman-compose &> /dev/null; then
    echo "❌ 需要安裝 podman-compose"
    echo "💡 安裝命令: pip install podman-compose"
    exit 1
fi

# 停止舊服務 (兩種配置都停止)
echo "🛑 停止舊服務..."
podman-compose -f podman-compose.integrated.yml down 2>/dev/null || true
podman-compose -f podman-compose.simple.yml down 2>/dev/null || true

# 構建並啟動新的統一服務
echo "🔨 構建並啟動統一服務..."
echo "   使用配置: podman-compose.simple.yml"
echo "   Dockerfile: Dockerfile.unified"
podman-compose -f podman-compose.simple.yml up --build -d

# 等待服務就緒
echo "⏳ 等待服務啟動..."
sleep 30

# 測試服務
echo "🧪 測試服務..."
if curl -f http://localhost:8000/health 2>/dev/null; then
    echo ""
    echo "✅ 部署成功！"
    echo "🌐 訪問地址: http://localhost:8000"
    echo "📊 服務狀態:"
    podman-compose -f podman-compose.simple.yml ps
else
    echo "❌ 服務啟動失敗，檢查日誌..."
    podman-compose -f podman-compose.simple.yml logs --tail=50
    exit 1
fi