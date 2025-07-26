#!/bin/bash
# =======================================================
# Care Voice 整合架構一鍵構建腳本
# 功能: 自動執行三階段構建和部署
# =======================================================

set -e  # 遇到錯誤立即退出

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 配置變數
COMPOSE_FILE="podman-compose.simple.yml"
PROJECT_NAME="care-voice"
BUILD_MODE=${1:-"production"}  # production, dev, build-only
SKIP_CLEANUP=${SKIP_CLEANUP:-false}
VERBOSE=${VERBOSE:-false}

# 函數定義
print_banner() {
    echo -e "${CYAN}"
    echo "======================================================="
    echo "   🏗️  Care Voice 整合架構構建腳本 v1.0"
    echo "======================================================="
    echo -e "${NC}"
}

print_step() {
    echo -e "${BLUE}[步驟] $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${PURPLE}ℹ️  $1${NC}"
}

# 檢查依賴
check_dependencies() {
    print_step "檢查系統依賴..."
    
    # 檢查 podman-compose
    if ! command -v podman-compose &> /dev/null; then
        print_error "podman-compose 未安裝"
        print_info "請安裝: pip install podman-compose"
        exit 1
    fi
    
    # 檢查 podman
    if ! command -v podman &> /dev/null; then
        print_error "podman 未安裝"
        exit 1
    fi
    
    # 檢查配置文件
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        print_error "找不到 $COMPOSE_FILE"
        exit 1
    fi
    
    print_success "依賴檢查通過"
}

# 顯示系統資訊
show_system_info() {
    print_step "系統資訊"
    echo "  - Podman 版本: $(podman --version)"
    echo "  - 構建模式: $BUILD_MODE"
    echo "  - 項目名稱: $PROJECT_NAME"
    echo "  - 配置文件: $COMPOSE_FILE"
    echo "  - 當前目錄: $(pwd)"
    echo "  - 當前分支: $(git branch --show-current 2>/dev/null || echo 'N/A')"
    echo "  - Git 狀態: $(git status --porcelain 2>/dev/null | wc -l) 個修改"
}

# 清理舊容器和鏡像
cleanup_old_resources() {
    if [[ "$SKIP_CLEANUP" == "true" ]]; then
        print_warning "跳過清理步驟"
        return
    fi
    
    print_step "清理舊資源..."
    
    # 停止並移除舊容器
    podman-compose -f "$COMPOSE_FILE" down --remove-orphans 2>/dev/null || true
    
    # 移除舊鏡像 (可選)
    read -p "是否移除舊的鏡像? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        podman rmi care-voice-frontend:latest 2>/dev/null || true
        podman rmi care-voice-backend:latest 2>/dev/null || true
        podman rmi care-voice-integrated:latest 2>/dev/null || true
        print_success "舊鏡像已移除"
    else
        print_info "保留舊鏡像"
    fi
}

# 階段1: 前端編譯
build_frontend() {
    print_step "階段1: 前端編譯 (SolidJS + Vite)"
    
    echo "📦 開始前端編譯..."
    if [[ "$VERBOSE" == "true" ]]; then
        podman-compose -f "$COMPOSE_FILE" build frontend-builder
    else
        podman-compose -f "$COMPOSE_FILE" build frontend-builder --quiet
    fi
    
    # 驗證編譯結果
    echo "🔍 驗證前端編譯結果..."
    podman run --rm care-voice-frontend:latest ls -la /app/dist/
    
    print_success "前端編譯完成"
}

# 階段2: 後端編譯
build_backend() {
    print_step "階段2: 後端編譯 (Rust + Opus)"
    
    echo "🦀 開始後端編譯..."
    if [[ "$VERBOSE" == "true" ]]; then
        podman-compose -f "$COMPOSE_FILE" build backend-builder
    else
        podman-compose -f "$COMPOSE_FILE" build backend-builder --quiet
    fi
    
    # 驗證編譯結果
    echo "🔍 驗證後端編譯結果..."
    podman run --rm care-voice-backend:latest ls -la /app/target/release/care-voice
    
    print_success "後端編譯完成"
}

# 階段3: 最終整合
build_integrated() {
    print_step "階段3: 最終整合 (nginx + 前端 + 後端)"
    
    echo "🔗 開始整合構建..."
    if [[ "$VERBOSE" == "true" ]]; then
        podman-compose -f "$COMPOSE_FILE" build care-voice-integrated
    else
        podman-compose -f "$COMPOSE_FILE" build care-voice-integrated --quiet
    fi
    
    print_success "整合構建完成"
}

# 啟動服務
start_services() {
    print_step "啟動整合服務"
    
    case "$BUILD_MODE" in
        "production")
            echo "🚀 啟動生產環境..."
            podman-compose -f "$COMPOSE_FILE" up -d care-voice-integrated
            ;;
        "dev")
            echo "🔧 啟動開發環境..."
            podman-compose -f "$COMPOSE_FILE" --profile dev up -d
            ;;
        "build-only")
            echo "📦 僅構建，不啟動服務"
            return
            ;;
        *)
            print_error "未知的構建模式: $BUILD_MODE"
            exit 1
            ;;
    esac
    
    if [[ "$BUILD_MODE" != "build-only" ]]; then
        print_success "服務啟動完成"
    fi
}

# 等待服務就緒
wait_for_services() {
    if [[ "$BUILD_MODE" == "build-only" ]]; then
        return
    fi
    
    print_step "等待服務就緒..."
    
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f http://localhost:8000/health &>/dev/null; then
            print_success "服務就緒 (嘗試 $attempt/$max_attempts)"
            return
        fi
        
        echo "等待服務啟動... ($attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done
    
    print_error "服務啟動超時"
    show_service_logs
    exit 1
}

# 顯示服務狀態
show_service_status() {
    print_step "服務狀態"
    
    echo "📊 容器狀態:"
    podman-compose -f "$COMPOSE_FILE" ps
    
    echo ""
    echo "🌐 服務端點:"
    echo "  - 主要服務: http://localhost:8000"
    echo "  - 健康檢查: http://localhost:8000/health"
    echo "  - API 端點: http://localhost:8000/api/"
    
    if [[ "$BUILD_MODE" == "dev" ]]; then
        echo "  - 前端開發: http://localhost:3000"
        echo "  - 後端開發: http://localhost:8001"
    fi
}

# 顯示服務日誌
show_service_logs() {
    print_step "最近日誌"
    podman-compose -f "$COMPOSE_FILE" logs --tail=20
}

# 測試基本功能
test_basic_functionality() {
    if [[ "$BUILD_MODE" == "build-only" ]]; then
        return
    fi
    
    print_step "測試基本功能"
    
    # 測試健康檢查
    echo "🔍 測試健康檢查..."
    if curl -f http://localhost:8000/health; then
        print_success "健康檢查通過"
    else
        print_error "健康檢查失敗"
        return 1
    fi
    
    # 測試前端頁面
    echo "🔍 測試前端頁面..."
    if curl -f http://localhost:8000/ > /dev/null; then
        print_success "前端頁面可訪問"
    else
        print_error "前端頁面無法訪問"
        return 1
    fi
    
    print_success "基本功能測試通過"
}

# 顯示鏡像資訊
show_image_info() {
    print_step "鏡像資訊"
    
    echo "📦 構建的鏡像:"
    podman images | grep -E "(care-voice|nginx|rust|node)" | head -10
    
    echo ""
    echo "💾 鏡像大小:"
    for image in "care-voice-frontend:latest" "care-voice-backend:latest" "care-voice-integrated:latest"; do
        if podman image exists "$image" 2>/dev/null; then
            size=$(podman images --format "{{.Size}}" "$image" 2>/dev/null)
            echo "  - $image: $size"
        fi
    done
}

# 顯示使用說明
show_usage() {
    echo "使用方法: $0 [模式] [選項]"
    echo ""
    echo "模式:"
    echo "  production    - 生產環境構建和部署 (預設)"
    echo "  dev           - 開發環境構建和部署"
    echo "  build-only    - 僅構建，不啟動"
    echo ""
    echo "環境變數:"
    echo "  SKIP_CLEANUP=true    - 跳過清理步驟"
    echo "  VERBOSE=true         - 顯示詳細輸出"
    echo ""
    echo "範例:"
    echo "  $0                   # 生產環境構建"
    echo "  $0 dev               # 開發環境構建"
    echo "  $0 build-only        # 僅構建"
    echo "  VERBOSE=true $0      # 詳細輸出"
}

# 主函數
main() {
    # 檢查參數
    if [[ "$1" == "--help" || "$1" == "-h" ]]; then
        show_usage
        exit 0
    fi
    
    print_banner
    
    # 記錄開始時間
    local start_time=$(date +%s)
    
    # 執行構建流程
    check_dependencies
    show_system_info
    cleanup_old_resources
    
    # 並行構建前後端 (如果支援)
    if command -v parallel &> /dev/null; then
        print_step "並行構建前後端"
        parallel -j2 ::: build_frontend build_backend
    else
        build_frontend
        build_backend
    fi
    
    build_integrated
    start_services
    wait_for_services
    show_service_status
    test_basic_functionality
    show_image_info
    
    # 計算耗時
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_success "整合構建完成！耗時: ${duration}秒"
    
    # 顯示下一步操作
    echo ""
    print_info "下一步操作:"
    echo "  - 訪問應用: http://localhost:8000"
    echo "  - 查看日誌: podman-compose -f $COMPOSE_FILE logs -f"
    echo "  - 停止服務: podman-compose -f $COMPOSE_FILE down"
    echo "  - 重啟服務: podman-compose -f $COMPOSE_FILE restart"
}

# 錯誤處理
trap 'print_error "構建過程中發生錯誤"; exit 1' ERR

# 執行主函數
main "$@"