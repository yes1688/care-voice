#!/bin/bash
# ===================================
# Care Voice Enterprise 構建腳本
# 業界領先的 AI 語音轉錄服務構建系統
# ===================================

set -euo pipefail

# 顏色配置
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 配置
IMAGE_NAME="care-voice-enterprise"
VERSION="0.3.0"
REGISTRY="${REGISTRY:-localhost}"
FULL_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:${VERSION}"
LATEST_TAG="${REGISTRY}/${IMAGE_NAME}:latest"

# 函數定義
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# 檢查依賴
check_dependencies() {
    log_step "檢查構建依賴..."
    
    local deps=("podman" "git" "curl")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "缺少必要依賴: ${missing_deps[*]}"
        log_info "請安裝缺少的依賴項"
        exit 1
    fi
    
    log_success "所有依賴檢查通過"
}

# 檢查 GPU 支援
check_gpu_support() {
    log_step "檢查 GPU 支援..."
    
    if command -v nvidia-smi &> /dev/null; then
        if nvidia-smi &> /dev/null; then
            local gpu_info
            gpu_info=$(nvidia-smi --query-gpu=name,driver_version,cuda_version --format=csv,noheader | head -1)
            log_success "檢測到 GPU: $gpu_info"
            return 0
        else
            log_warning "nvidia-smi 可用但無法訪問 GPU"
        fi
    else
        log_warning "nvidia-smi 不可用"
    fi
    
    log_warning "未檢測到可用的 NVIDIA GPU，將構建 CPU 版本"
    return 1
}

# 檢查容器運行時
check_container_runtime() {
    log_step "檢查容器運行時..."
    
    if ! podman info &> /dev/null; then
        log_error "Podman 運行時不可用"
        exit 1
    fi
    
    local podman_version
    podman_version=$(podman --version | cut -d' ' -f3)
    log_success "Podman 版本: $podman_version"
}

# 清理舊映像
cleanup_old_images() {
    log_step "清理舊映像..."
    
    local old_images
    old_images=$(podman images --filter "reference=${REGISTRY}/${IMAGE_NAME}" --format "{{.Repository}}:{{.Tag}}" | grep -v ":${VERSION}$" | grep -v ":latest$" || true)
    
    if [ -n "$old_images" ]; then
        log_info "發現舊映像，正在清理..."
        echo "$old_images" | xargs -r podman rmi -f
        log_success "舊映像清理完成"
    else
        log_info "沒有發現需要清理的舊映像"
    fi
}

# 構建映像
build_image() {
    log_step "開始構建 Care Voice Enterprise 映像..."
    
    local build_start
    build_start=$(date +%s)
    
    # 構建參數
    local build_args=(
        "--file" "Dockerfile.enterprise"
        "--tag" "$FULL_IMAGE_NAME"
        "--tag" "$LATEST_TAG"
        "--label" "build.timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        "--label" "build.version=$VERSION"
        "--label" "build.commit=$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
        "--label" "build.branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
    )
    
    # GPU 支援
    if check_gpu_support; then
        build_args+=("--device" "nvidia.com/gpu=all")
        log_info "啟用 GPU 支援構建"
    fi
    
    # 開始構建
    log_info "構建映像: $FULL_IMAGE_NAME"
    log_info "使用 Dockerfile: Dockerfile.enterprise"
    
    if podman build "${build_args[@]}" .; then
        local build_end
        build_end=$(date +%s)
        local build_duration=$((build_end - build_start))
        
        log_success "映像構建成功！"
        log_info "構建時間: ${build_duration} 秒"
    else
        log_error "映像構建失敗"
        exit 1
    fi
}

# 驗證映像
verify_image() {
    log_step "驗證映像..."
    
    # 檢查映像是否存在
    if ! podman image exists "$FULL_IMAGE_NAME"; then
        log_error "映像不存在: $FULL_IMAGE_NAME"
        exit 1
    fi
    
    # 獲取映像資訊
    local image_size
    image_size=$(podman image ls --format "{{.Size}}" --filter "reference=$FULL_IMAGE_NAME")
    log_info "映像大小: $image_size"
    
    # 檢查映像標籤
    local image_labels
    image_labels=$(podman image inspect "$FULL_IMAGE_NAME" --format "{{json .Labels}}" | jq -r 'to_entries[] | "\(.key)=\(.value)"' | head -5)
    log_info "映像標籤:"
    echo "$image_labels" | sed 's/^/  /'
    
    log_success "映像驗證通過"
}

# 運行健康檢查
health_check() {
    log_step "執行健康檢查..."
    
    local container_name="care-voice-health-check-$$"
    local health_check_timeout=60
    
    log_info "啟動測試容器..."
    
    # GPU 運行參數
    local run_args=(
        "--name" "$container_name"
        "--rm"
        "--detach"
        "--publish" "18000:8000"
    )
    
    if check_gpu_support; then
        run_args+=("--device" "nvidia.com/gpu=all")
    fi
    
    # 啟動容器
    local container_id
    if container_id=$(podman run "${run_args[@]}" "$FULL_IMAGE_NAME"); then
        log_info "測試容器啟動成功: $container_id"
    else
        log_error "測試容器啟動失敗"
        return 1
    fi
    
    # 等待服務啟動
    log_info "等待服務啟動..."
    local attempts=0
    local max_attempts=$((health_check_timeout / 5))
    
    while [ $attempts -lt $max_attempts ]; do
        if curl -sf http://localhost:18000/health &> /dev/null; then
            log_success "健康檢查通過"
            podman stop "$container_name" &> /dev/null || true
            return 0
        fi
        
        sleep 5
        ((attempts++))
        echo -n "."
    done
    
    echo ""
    log_error "健康檢查超時"
    
    # 顯示容器日誌
    log_info "容器日誌:"
    podman logs "$container_name" | tail -20
    
    podman stop "$container_name" &> /dev/null || true
    return 1
}

# 顯示構建摘要
show_summary() {
    log_step "構建摘要"
    
    cat << EOF

🎉 Care Voice Enterprise 構建完成！

📊 映像資訊:
   名稱: $FULL_IMAGE_NAME
   標籤: $LATEST_TAG
   大小: $(podman image ls --format "{{.Size}}" --filter "reference=$FULL_IMAGE_NAME")

🚀 使用方式:
   # 運行容器
   podman run -d --name care-voice-enterprise \\
     --device nvidia.com/gpu=all \\
     -p 8000:8000 \\
     $FULL_IMAGE_NAME

   # 檢查健康狀態
   curl http://localhost:8000/health

   # 查看容器日誌
   podman logs care-voice-enterprise

🎯 功能特色:
   ✅ GPU 加速 (CUDA 12.9)
   ✅ 多模型並行處理
   ✅ 99.9% 瀏覽器支援
   ✅ 企業級可靠性
   ✅ 實時效能監控

📚 文檔: https://github.com/care-voice/care-voice
EOF
}

# 主函數
main() {
    echo -e "${CYAN}"
    cat << "EOF"
   ____                 __     __      _          
  / ___|__ _ _ __ ___    \ \   / /__  (_) ___ ___ 
 | |   / _` | '__/ _ \    \ \ / / _ \ | |/ __/ _ \
 | |__| (_| | | |  __/     \ V / (_) || | (_|  __/
  \____\__,_|_|  \___|      \_/ \___/ |_|\___\___|
                                                  
        Enterprise AI Voice Transcription
             Industry-Leading Performance
EOF
    echo -e "${NC}"
    
    log_info "開始構建 Care Voice Enterprise v$VERSION"
    
    # 執行構建步驟
    check_dependencies
    check_container_runtime
    cleanup_old_images
    build_image
    verify_image
    
    # 可選的健康檢查
    if [ "${SKIP_HEALTH_CHECK:-false}" != "true" ]; then
        health_check
    else
        log_warning "跳過健康檢查 (SKIP_HEALTH_CHECK=true)"
    fi
    
    show_summary
    
    log_success "🎉 Care Voice Enterprise 構建成功完成！"
}

# 錯誤處理
trap 'log_error "構建過程中發生錯誤"; exit 1' ERR

# 執行主函數
main "$@"