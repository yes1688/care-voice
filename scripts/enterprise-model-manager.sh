#!/bin/bash
# ===================================
# Care Voice 業界領先企業級模型管理系統
# Enterprise-Grade Model Management System
# ===================================

set -euo pipefail

# 配置參數
MODEL_DIR="${MODEL_DIR:-/app/models}"
DOWNLOAD_DIR="${DOWNLOAD_DIR:-./models}"
BACKUP_DIR="${BACKUP_DIR:-./models-backup}"

# 業界標準模型配置
declare -A ENTERPRISE_MODELS=(
    ["base"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
    ["small"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
    ["medium"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
    ["large-v3"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
)

declare -A MODEL_SIZES=(
    ["base"]="142MB"
    ["small"]="244MB"
    ["medium"]="769MB"
    ["large-v3"]="1550MB"
)

declare -A MODEL_QUALITY=(
    ["base"]="Good - 快速推理，適合即時應用"
    ["small"]="Better - 平衡速度與準確度"
    ["medium"]="Best - 高準確度，企業級品質"
    ["large-v3"]="Ultimate - 業界領先準確度"
)

# 顏色和格式
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# 日誌函數
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

log_enterprise() {
    echo -e "${PURPLE}[ENTERPRISE]${NC} $1"
}

# 檢查系統需求
check_requirements() {
    log_info "檢查企業級系統需求..."
    
    # 檢查必要工具
    for tool in curl wget sha256sum; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "缺少必要工具: $tool"
            exit 1
        fi
    done
    
    # 檢查磁盤空間 (需要至少 5GB)
    available_space=$(df "$(dirname "$DOWNLOAD_DIR")" | awk 'NR==2 {print $4}')
    required_space=$((5 * 1024 * 1024)) # 5GB in KB
    
    if [ "$available_space" -lt "$required_space" ]; then
        log_error "磁盤空間不足。需要至少 5GB，可用: $((available_space/1024/1024))GB"
        exit 1
    fi
    
    log_success "系統需求檢查通過"
}

# 創建目錄結構
setup_directories() {
    log_info "設置企業級目錄結構..."
    
    mkdir -p "$DOWNLOAD_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "${DOWNLOAD_DIR}/checksums"
    mkdir -p "${DOWNLOAD_DIR}/metadata"
    
    log_success "目錄結構創建完成"
}

# 顯示模型信息
show_model_info() {
    echo -e "${BOLD}${CYAN}===============================================${NC}"
    echo -e "${BOLD}${CYAN}   Care Voice 企業級 Whisper 模型選擇${NC}"
    echo -e "${BOLD}${CYAN}===============================================${NC}"
    echo ""
    
    for model in "${!ENTERPRISE_MODELS[@]}"; do
        echo -e "${BOLD}🤖 模型: ${GREEN}$model${NC}"
        echo -e "   📊 大小: ${YELLOW}${MODEL_SIZES[$model]}${NC}"
        echo -e "   🎯 品質: ${MODEL_QUALITY[$model]}"
        echo -e "   🔗 URL: ${ENTERPRISE_MODELS[$model]}"
        echo ""
    done
}

# 下載模型（帶重試和校驗）
download_model_with_verification() {
    local model_name="$1"
    local url="${ENTERPRISE_MODELS[$model_name]}"
    local filename="ggml-${model_name}.bin"
    local filepath="${DOWNLOAD_DIR}/${filename}"
    local temp_filepath="${filepath}.tmp"
    
    log_enterprise "開始下載企業級模型: $model_name"
    log_info "目標路徑: $filepath"
    log_info "模型大小: ${MODEL_SIZES[$model_name]}"
    
    # 檢查是否已存在
    if [[ -f "$filepath" ]]; then
        log_warning "模型文件已存在，驗證完整性..."
        if verify_model_integrity "$filepath"; then
            log_success "現有模型驗證通過，跳過下載"
            return 0
        else
            log_warning "現有模型損壞，重新下載..."
        fi
    fi
    
    # 下載（帶重試機制）
    local max_retries=3
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        log_info "下載嘗試 $((retry_count + 1))/$max_retries..."
        
        if curl -L --fail \
                --progress-bar \
                --retry 3 \
                --retry-delay 5 \
                --max-time 3600 \
                --continue-at - \
                -o "$temp_filepath" \
                "$url"; then
            log_success "下載完成"
            break
        else
            ((retry_count++))
            log_warning "下載失敗，重試中... ($retry_count/$max_retries)"
            sleep 10
        fi
    done
    
    if [ $retry_count -eq $max_retries ]; then
        log_error "下載失敗: $model_name"
        rm -f "$temp_filepath"
        return 1
    fi
    
    # 移動到最終位置
    mv "$temp_filepath" "$filepath"
    
    # 驗證下載的文件
    if verify_model_integrity "$filepath"; then
        log_success "模型 $model_name 下載並驗證成功"
        
        # 創建元數據
        create_model_metadata "$model_name" "$filepath"
        
        return 0
    else
        log_error "模型 $model_name 驗證失敗"
        rm -f "$filepath"
        return 1
    fi
}

# 驗證模型完整性
verify_model_integrity() {
    local filepath="$1"
    
    # 基本檢查：文件存在且非空
    if [[ ! -f "$filepath" ]] || [[ ! -s "$filepath" ]]; then
        log_error "文件不存在或為空: $filepath"
        return 1
    fi
    
    # 檢查文件格式（Whisper 模型特徵）
    local file_header
    file_header=$(hexdump -C "$filepath" | head -1)
    
    # 檢查 ggml 格式的兩種字節序：6c6d6767 (lmgg) 或 6767 6d6c (ggml)
    if [[ "$file_header" == *"6c 6d 67 67"* ]] || [[ "$file_header" == *"6767 6d6c"* ]]; then
        log_success "模型格式驗證通過 (ggml format detected)"
        return 0
    else
        log_error "模型格式驗證失敗"
        log_error "文件頭部: $file_header"
        return 1
    fi
}

# 創建模型元數據
create_model_metadata() {
    local model_name="$1"
    local filepath="$2"
    local metadata_file="${DOWNLOAD_DIR}/metadata/${model_name}.json"
    
    local file_size
    file_size=$(stat -c%s "$filepath")
    
    local checksum
    checksum=$(sha256sum "$filepath" | cut -d' ' -f1)
    
    cat > "$metadata_file" << EOF
{
    "model_name": "$model_name",
    "file_path": "$filepath",
    "file_size": $file_size,
    "sha256": "$checksum",
    "download_date": "$(date -Iseconds)",
    "version": "enterprise-v1.0",
    "quality": "${MODEL_QUALITY[$model_name]}",
    "url": "${ENTERPRISE_MODELS[$model_name]}"
}
EOF
    
    log_success "元數據創建完成: $metadata_file"
}

# 主下載函數
download_enterprise_models() {
    local models_to_download=("$@")
    
    if [ ${#models_to_download[@]} -eq 0 ]; then
        # 默認下載推薦模型
        models_to_download=("base" "small")
        log_info "未指定模型，下載企業推薦組合: ${models_to_download[*]}"
    fi
    
    log_enterprise "開始企業級模型部署流程"
    
    for model in "${models_to_download[@]}"; do
        if [[ -z "${ENTERPRISE_MODELS[$model]:-}" ]]; then
            log_error "未知模型: $model"
            continue
        fi
        
        if download_model_with_verification "$model"; then
            log_success "✅ 模型 $model 部署成功"
        else
            log_error "❌ 模型 $model 部署失敗"
        fi
    done
}

# 部署到容器
deploy_to_container() {
    local container_name="$1"
    
    log_enterprise "部署模型到容器: $container_name"
    
    # 檢查容器是否存在
    if ! podman ps -a --format "{{.Names}}" | grep -q "^${container_name}$"; then
        log_error "容器不存在: $container_name"
        return 1
    fi
    
    # 複製模型文件
    for model_file in "$DOWNLOAD_DIR"/*.bin; do
        if [[ -f "$model_file" ]]; then
            local filename
            filename=$(basename "$model_file")
            
            log_info "部署模型: $filename"
            
            if podman cp "$model_file" "${container_name}:/app/models/"; then
                log_success "✅ $filename 部署成功"
            else
                log_error "❌ $filename 部署失敗"
            fi
        fi
    done
    
    # 驗證部署
    log_info "驗證容器內模型..."
    podman exec "$container_name" ls -la /app/models/
}

# 清理函數
cleanup_models() {
    log_info "清理舊模型文件..."
    
    # 備份現有模型
    if [[ -d "$DOWNLOAD_DIR" ]] && [[ -n "$(ls -A "$DOWNLOAD_DIR"/*.bin 2>/dev/null)" ]]; then
        local backup_timestamp
        backup_timestamp=$(date +"%Y%m%d_%H%M%S")
        local backup_path="${BACKUP_DIR}/backup_${backup_timestamp}"
        
        mkdir -p "$backup_path"
        cp "$DOWNLOAD_DIR"/*.bin "$backup_path" 2>/dev/null || true
        log_success "模型已備份到: $backup_path"
    fi
    
    # 清理下載目錄
    rm -f "$DOWNLOAD_DIR"/*.bin
    rm -f "$DOWNLOAD_DIR"/*.tmp
    
    log_success "清理完成"
}

# 主函數
main() {
    echo -e "${BOLD}${PURPLE}===============================================${NC}"
    echo -e "${BOLD}${PURPLE}  Care Voice 企業級模型管理系統 v2.0${NC}"
    echo -e "${BOLD}${PURPLE}===============================================${NC}"
    echo ""
    
    local command="${1:-download}"
    shift || true
    
    case "$command" in
        "download")
            check_requirements
            setup_directories
            show_model_info
            download_enterprise_models "$@"
            ;;
        "deploy")
            local container_name="${1:-care-voice-backend}"
            deploy_to_container "$container_name"
            ;;
        "clean")
            cleanup_models
            ;;
        "info")
            show_model_info
            ;;
        "verify")
            for model_file in "$DOWNLOAD_DIR"/*.bin; do
                if [[ -f "$model_file" ]]; then
                    verify_model_integrity "$model_file"
                fi
            done
            ;;
        *)
            echo "使用方法: $0 [download|deploy|clean|info|verify] [模型名稱...]"
            echo ""
            echo "命令："
            echo "  download [models] - 下載指定模型 (默認: base small)"
            echo "  deploy [container] - 部署到容器 (默認: care-voice-backend)"
            echo "  clean            - 清理模型文件"
            echo "  info             - 顯示模型信息"
            echo "  verify           - 驗證模型完整性"
            echo ""
            echo "可用模型: ${!ENTERPRISE_MODELS[*]}"
            exit 1
            ;;
    esac
}

# 執行主函數
main "$@"