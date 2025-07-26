#!/bin/bash
# =======================================================
# Care Voice 簡化部署腳本
# 功能: 一鍵部署整合服務
# =======================================================

set -e

echo "🚀 Care Voice 一鍵部署"
echo "======================="

# 檢查 podman-compose
if ! command -v podman-compose &> /dev/null; then
    echo "❌ 需要安裝 podman-compose"
    echo "💡 安裝命令: pip install podman-compose"
    exit 1
fi

# 停止舊服務
echo "🛑 停止舊服務..."
podman-compose -f podman-compose.integrated.yml down 2>/dev/null || true

# 構建並啟動
echo "🔨 構建並啟動服務..."
podman-compose -f podman-compose.integrated.yml up --build -d

# 等待服務就緒
echo "⏳ 等待服務啟動..."
sleep 30

# 測試服務
echo "🧪 測試服務..."
if curl -f http://localhost:8000/health; then
    echo "✅ 部署成功！"
    echo "🌐 訪問地址: http://localhost:8000"
else
    echo "❌ 服務啟動失敗"
    podman-compose -f podman-compose.integrated.yml logs
    exit 1
fi