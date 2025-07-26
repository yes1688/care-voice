#!/bin/bash
# ===================================
# Care Voice Enterprise - Kubernetes 部署腳本
# 企業級自動化部署和監控
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
NAMESPACE="care-voice"
IMAGE_NAME="care-voice-enterprise"
VERSION="0.3.0"
KUBE_CONFIG="${KUBECONFIG:-$HOME/.kube/config}"

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
    log_step "檢查部署依賴..."
    
    local deps=("kubectl" "helm")
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

# 檢查 Kubernetes 連接
check_kubernetes() {
    log_step "檢查 Kubernetes 連接..."
    
    if ! kubectl cluster-info &> /dev/null; then
        log_error "無法連接到 Kubernetes 集群"
        log_info "請檢查 kubeconfig: $KUBE_CONFIG"
        exit 1
    fi
    
    local k8s_version
    k8s_version=$(kubectl version --short --client 2>/dev/null | grep Client | cut -d' ' -f3)
    log_success "Kubernetes 連接正常，kubectl 版本: $k8s_version"
    
    # 檢查 GPU 節點
    local gpu_nodes
    gpu_nodes=$(kubectl get nodes -l accelerator=nvidia-gpu --no-headers 2>/dev/null | wc -l || echo "0")
    
    if [ "$gpu_nodes" -eq 0 ]; then
        log_warning "未檢測到 GPU 節點，Care Voice 將以 CPU 模式運行"
        log_info "如需 GPU 加速，請確保集群中有標籤為 'accelerator=nvidia-gpu' 的節點"
    else
        log_success "檢測到 $gpu_nodes 個 GPU 節點"
    fi
}

# 創建命名空間和 RBAC
setup_namespace() {
    log_step "設置命名空間和安全配置..."
    
    if kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_info "命名空間 '$NAMESPACE' 已存在"
    else
        log_info "創建命名空間 '$NAMESPACE'"
        kubectl apply -f k8s/namespace.yaml
        log_success "命名空間創建完成"
    fi
    
    # 等待命名空間就緒
    kubectl wait --for=condition=Ready namespace/"$NAMESPACE" --timeout=60s
}

# 檢查映像是否存在
check_image() {
    log_step "檢查容器映像..."
    
    # 這裡假設映像已經構建並推送到可訪問的註冊表
    # 在實際環境中，您可能需要推送到 DockerHub、ECR、GCR 等
    
    local image_check_pod="image-check-$$"
    
    # 嘗試在集群中拉取映像
    if kubectl run "$image_check_pod" \
        --image="localhost/${IMAGE_NAME}:${VERSION}" \
        --restart=Never \
        --rm \
        -i \
        --timeout=60s \
        --namespace="$NAMESPACE" \
        --command -- echo "Image check" &> /dev/null; then
        log_success "容器映像可用: localhost/${IMAGE_NAME}:${VERSION}"
    else
        log_warning "無法拉取映像，請確保映像已構建並可訪問"
        log_info "如需構建映像，請運行: ./build-enterprise.sh"
        
        # 詢問是否繼續
        read -p "是否繼續部署？(y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "部署已取消"
            exit 1
        fi
    fi
}

# 檢查存儲類
check_storage_class() {
    log_step "檢查存儲配置..."
    
    # 檢查是否有 fast-ssd 存儲類
    if kubectl get storageclass fast-ssd &> /dev/null; then
        log_success "找到 fast-ssd 存儲類"
    else
        log_warning "未找到 fast-ssd 存儲類"
        
        # 列出可用的存儲類
        local default_sc
        default_sc=$(kubectl get storageclass -o jsonpath='{.items[?(@.metadata.annotations.storageclass\.kubernetes\.io/is-default-class=="true")].metadata.name}')
        
        if [ -n "$default_sc" ]; then
            log_info "將使用預設存儲類: $default_sc"
            # 更新部署配置
            sed -i.bak "s/storageClassName: fast-ssd/storageClassName: $default_sc/" k8s/deployment.yaml
        else
            log_warning "未找到預設存儲類，將使用集群預設"
            sed -i.bak '/storageClassName:/d' k8s/deployment.yaml
        fi
    fi
}

# 部署應用
deploy_application() {
    log_step "部署 Care Voice Enterprise..."
    
    # 應用配置
    kubectl apply -f k8s/deployment.yaml
    
    log_info "等待 Deployment 就緒..."
    kubectl rollout status deployment/care-voice-enterprise -n "$NAMESPACE" --timeout=600s
    
    log_success "Care Voice Enterprise 部署完成"
}

# 等待服務就緒
wait_for_service() {
    log_step "等待服務就緒..."
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        local ready_pods
        ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app=care-voice --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l)
        
        if [ "$ready_pods" -gt 0 ]; then
            log_success "服務已就緒，運行中的 Pod 數量: $ready_pods"
            break
        fi
        
        log_info "等待服務啟動... (嘗試 $attempt/$max_attempts)"
        sleep 10
        ((attempt++))
    done
    
    if [ $attempt -gt $max_attempts ]; then
        log_error "服務啟動超時"
        return 1
    fi
}

# 健康檢查
health_check() {
    log_step "執行健康檢查..."
    
    # 獲取服務 IP
    local service_ip
    service_ip=$(kubectl get service care-voice-enterprise-service -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}')
    
    if [ -z "$service_ip" ]; then
        log_error "無法獲取服務 IP"
        return 1
    fi
    
    log_info "服務 IP: $service_ip"
    
    # 在集群內執行健康檢查
    local health_check_pod="health-check-$$"
    
    if kubectl run "$health_check_pod" \
        --image=curlimages/curl:8.6.0 \
        --restart=Never \
        --rm \
        -i \
        --timeout=60s \
        --namespace="$NAMESPACE" \
        --command -- curl -s "http://$service_ip/health" | grep -q "healthy"; then
        log_success "健康檢查通過"
        return 0
    else
        log_error "健康檢查失敗"
        return 1
    fi
}

# 顯示部署資訊
show_deployment_info() {
    log_step "部署資訊摘要"
    
    echo
    echo "🎉 Care Voice Enterprise 部署完成！"
    echo
    
    # Pod 資訊
    echo "📊 Pod 狀態:"
    kubectl get pods -n "$NAMESPACE" -l app=care-voice -o wide
    echo
    
    # 服務資訊
    echo "🌐 服務資訊:"
    kubectl get services -n "$NAMESPACE"
    echo
    
    # HPA 資訊
    echo "📈 自動擴展狀態:"
    kubectl get hpa -n "$NAMESPACE" || echo "HPA 未配置"
    echo
    
    # 獲取訪問方式
    local service_type
    service_type=$(kubectl get service care-voice-enterprise-service -n "$NAMESPACE" -o jsonpath='{.spec.type}')
    
    case $service_type in
        "LoadBalancer")
            local external_ip
            external_ip=$(kubectl get service care-voice-enterprise-service -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
            if [ -n "$external_ip" ]; then
                echo "🔗 外部訪問地址: http://$external_ip"
            else
                echo "⏳ 等待 LoadBalancer IP 分配..."
            fi
            ;;
        "NodePort")
            local node_port
            node_port=$(kubectl get service care-voice-enterprise-service -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].nodePort}')
            local node_ip
            node_ip=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="ExternalIP")].address}')
            echo "🔗 NodePort 訪問: http://$node_ip:$node_port"
            ;;
        "ClusterIP")
            echo "🔗 集群內訪問: kubectl port-forward service/care-voice-enterprise-service -n $NAMESPACE 8080:80"
            ;;
    esac
    
    echo
    echo "📋 有用的命令:"
    echo "  查看 Pod 日誌: kubectl logs -f deployment/care-voice-enterprise -n $NAMESPACE"
    echo "  檢查健康狀態: kubectl get pods -n $NAMESPACE"
    echo "  端口轉發: kubectl port-forward service/care-voice-enterprise-service -n $NAMESPACE 8080:80"
    echo "  刪除部署: kubectl delete namespace $NAMESPACE"
    echo
}

# 清理舊部署 (可選)
cleanup_old_deployment() {
    if [ "${CLEANUP_OLD:-false}" = "true" ]; then
        log_step "清理舊部署..."
        
        if kubectl get namespace "$NAMESPACE" &> /dev/null; then
            log_warning "刪除現有命名空間 '$NAMESPACE'"
            kubectl delete namespace "$NAMESPACE" --timeout=300s
            
            # 等待命名空間完全刪除
            while kubectl get namespace "$NAMESPACE" &> /dev/null; do
                log_info "等待命名空間刪除..."
                sleep 5
            done
        fi
        
        log_success "舊部署清理完成"
    fi
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
                                                  
        Enterprise Kubernetes Deployment
             Industry-Leading Performance
EOF
    echo -e "${NC}"
    
    log_info "開始部署 Care Voice Enterprise 到 Kubernetes"
    
    # 執行部署步驟
    check_dependencies
    check_kubernetes
    cleanup_old_deployment
    setup_namespace
    check_image
    check_storage_class
    deploy_application
    wait_for_service
    
    # 可選的健康檢查
    if [ "${SKIP_HEALTH_CHECK:-false}" != "true" ]; then
        if ! health_check; then
            log_warning "健康檢查失敗，但部署已完成"
        fi
    else
        log_warning "跳過健康檢查 (SKIP_HEALTH_CHECK=true)"
    fi
    
    show_deployment_info
    
    log_success "🎉 Care Voice Enterprise Kubernetes 部署成功完成！"
}

# 錯誤處理
trap 'log_error "部署過程中發生錯誤"; exit 1' ERR

# 執行主函數
main "$@"