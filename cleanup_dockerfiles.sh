#!/bin/bash

echo "🧹 Care Voice - 清理過時 Dockerfile 版本"
echo "=================================================="

# 建議移除的過時 Dockerfile
DEPRECATED_FILES=(
    "Dockerfile.gpu_simple"
    "Dockerfile.gpu_simple2"
    "Dockerfile.test_static"
    "Dockerfile.simple_static"
    "Dockerfile.cuda_simple"
    "Dockerfile.gpu_python"
    "Dockerfile.minimal_test"
    "Dockerfile.whisper_fix"
    "Dockerfile.test"
    "Dockerfile.gpu_working"
)

# 建議保留的核心配置
KEEP_FILES=(
    "Dockerfile.rtx50-series"      # RTX 50 系列最新優化
    "Dockerfile.blackdx_gpu"       # 穩定 GPU 解決方案
    "Dockerfile.blackdx_cpu"       # 穩定 CPU 解決方案
    "Dockerfile.unified"           # 統一部署方案
    "Dockerfile.verified_static"   # 驗證過的靜態版本
    "Dockerfile.gpu_optimized"     # GPU 優化版本
    "Dockerfile.gpu_test"          # GPU 診斷工具
)

echo "📋 分析當前 Dockerfile 版本..."
total_files=$(ls Dockerfile.* 2>/dev/null | wc -l)
echo "總計找到 $total_files 個 Dockerfile"

echo ""
echo "🗑️  建議移除的過時版本:"
for file in "${DEPRECATED_FILES[@]}"; do
    if [ -f "$file" ]; then
        size=$(du -h "$file" | cut -f1)
        echo "  ❌ $file ($size)"
    fi
done

echo ""
echo "✅ 建議保留的核心版本:"
for file in "${KEEP_FILES[@]}"; do
    if [ -f "$file" ]; then
        size=$(du -h "$file" | cut -f1)
        echo "  ✅ $file ($size)"
    fi
done

echo ""
echo "🔍 未分類的 Dockerfile:"
for file in Dockerfile.*; do
    if [ -f "$file" ]; then
        # 檢查是否在保留或移除列表中
        basename=$(basename "$file")
        in_deprecated=false
        in_keep=false
        
        for dep in "${DEPRECATED_FILES[@]}"; do
            if [ "$basename" = "$dep" ]; then
                in_deprecated=true
                break
            fi
        done
        
        for keep in "${KEEP_FILES[@]}"; do
            if [ "$basename" = "$keep" ]; then
                in_keep=true
                break
            fi
        done
        
        if [ "$in_deprecated" = false ] && [ "$in_keep" = false ]; then
            size=$(du -h "$file" | cut -f1)
            echo "  ❓ $basename ($size) - 需要手動檢查"
        fi
    fi
done

echo ""
echo "📊 清理統計:"
deprecated_count=0
for file in "${DEPRECATED_FILES[@]}"; do
    if [ -f "$file" ]; then
        ((deprecated_count++))
    fi
done

keep_count=0
for file in "${KEEP_FILES[@]}"; do
    if [ -f "$file" ]; then
        ((keep_count++))
    fi
done

echo "  可移除: $deprecated_count 個文件"
echo "  建議保留: $keep_count 個文件"
echo "  清理後將從 $total_files 個文件減少到約 $keep_count 個核心文件"

echo ""
echo "⚠️  執行清理命令 (請手動執行):"
echo "mkdir -p deprecated_dockerfiles"
for file in "${DEPRECATED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "mv $file deprecated_dockerfiles/"
    fi
done

echo ""
echo "🔧 清理完成後的建議配置："
echo "  🎯 GPU 環境: 使用 Dockerfile.rtx50-series"
echo "  🔄 穩定環境: 使用 Dockerfile.blackdx_gpu 或 Dockerfile.blackdx_cpu"
echo "  📦 統一部署: 使用 Dockerfile.unified"
echo "  🔍 問題診斷: 使用 Dockerfile.gpu_test"

echo ""
echo "✅ 分析完成！請檢查建議並手動執行清理命令。"