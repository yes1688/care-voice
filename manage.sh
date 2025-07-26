#!/bin/bash
# =======================================================
# Care Voice 統一服務管理腳本
# 用途: 管理 Care Voice 服務的啟動、停止、重啟等操作
# =======================================================

set -e

# 配置
COMPOSE_FILE="podman-compose.simple.yml"
SERVICE_NAME="care-voice"

# 顏色定義
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# 函數定義
print_usage() {
    echo "Care Voice 服務管理"
    echo "=================="
    echo "用法: $0 <命令>"
    echo ""
    echo "命令:"
    echo "  start    - 啟動服務"
    echo "  stop     - 停止服務"
    echo "  restart  - 重啟服務"
    echo "  status   - 查看服務狀態"
    echo "  logs     - 查看實時日誌"
    echo "  health   - 檢查服務健康狀態"
    echo ""
    echo "範例:"
    echo "  $0 start     # 啟動服務"
    echo "  $0 logs      # 查看日誌"
    echo "  $0 status    # 查看狀態"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
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

# 檢查服務是否存在
check_service_exists() {
    if ! podman-compose -f "$COMPOSE_FILE" ps | grep -q "$SERVICE_NAME"; then
        print_warning "服務容器不存在，請先運行 ./deploy.sh 部署服務"
        return 1
    fi
    return 0
}

# 啟動服務
start_service() {
    print_info "啟動 Care Voice 服務..."
    
    podman-compose -f "$COMPOSE_FILE" start "$SERVICE_NAME"
    
    print_info "等待服務啟動..."
    sleep 5
    
    if curl -f http://localhost:8000/health &>/dev/null; then
        print_success "服務啟動成功"
        print_info "訪問地址: http://localhost:8000"
    else
        print_error "服務啟動失敗，請檢查日誌"
        return 1
    fi
}

# 停止服務
stop_service() {
    print_info "停止 Care Voice 服務..."
    
    podman-compose -f "$COMPOSE_FILE" stop "$SERVICE_NAME"
    
    print_success "服務已停止"
}

# 重啟服務
restart_service() {
    print_info "重啟 Care Voice 服務..."
    
    stop_service
    sleep 2
    start_service
}

# 查看服務狀態
show_status() {
    print_info "Care Voice 服務狀態"
    echo "==================="
    
    # 容器狀態
    echo "📦 容器狀態:"
    podman-compose -f "$COMPOSE_FILE" ps
    
    echo ""
    echo "🌐 端點檢查:"
    
    # 健康檢查
    if curl -f http://localhost:8000/health &>/dev/null; then
        print_success "健康檢查: 正常"
    else
        print_error "健康檢查: 失敗"
    fi
    
    # 前端檢查
    if curl -f http://localhost:8000/ &>/dev/null; then
        print_success "前端服務: 正常"
    else
        print_error "前端服務: 失敗"
    fi
    
    echo ""
    echo "📊 資源使用:"
    podman stats --no-stream "$SERVICE_NAME" 2>/dev/null || echo "無法獲取資源使用情況"
}

# 查看日誌
show_logs() {
    print_info "Care Voice 服務日誌 (按 Ctrl+C 退出)"
    echo "================================="
    
    podman-compose -f "$COMPOSE_FILE" logs -f "$SERVICE_NAME"
}

# 健康檢查
health_check() {
    print_info "Care Voice 健康檢查"
    echo "==================="
    
    # 檢查容器是否運行
    if ! podman-compose -f "$COMPOSE_FILE" ps | grep -q "$SERVICE_NAME.*Up"; then
        print_error "容器未運行"
        return 1
    fi
    
    print_success "容器運行中"
    
    # 檢查端口
    if ! netstat -tlnp 2>/dev/null | grep -q ":8000 "; then
        print_error "端口 8000 未監聽"
        return 1
    fi
    
    print_success "端口 8000 正常監聽"
    
    # 檢查健康端點
    if curl -f http://localhost:8000/health &>/dev/null; then
        print_success "健康端點響應正常"
    else
        print_error "健康端點響應失敗"
        return 1
    fi
    
    # 檢查前端
    if curl -f http://localhost:8000/ &>/dev/null; then
        print_success "前端頁面可訪問"
    else
        print_error "前端頁面無法訪問"
        return 1
    fi
    
    print_success "所有檢查通過，服務健康運行"
}

# 主邏輯
main() {
    # 檢查參數
    if [[ $# -eq 0 ]]; then
        print_usage
        exit 1
    fi
    
    # 檢查 podman-compose
    if ! command -v podman-compose &> /dev/null; then
        print_error "需要安裝 podman-compose"
        exit 1
    fi
    
    # 檢查配置文件
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        print_error "找不到 $COMPOSE_FILE"
        exit 1
    fi
    
    case "$1" in
        "start")
            start_service
            ;;
        "stop")
            stop_service
            ;;
        "restart")
            restart_service
            ;;
        "status")
            show_status
            ;;
        "logs")
            show_logs
            ;;
        "health")
            health_check
            ;;
        "--help"|"-h"|"help")
            print_usage
            ;;
        *)
            print_error "未知命令: $1"
            echo ""
            print_usage
            exit 1
            ;;
    esac
}

# 執行主函數
main "$@"