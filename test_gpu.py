#!/usr/bin/env python3
"""
GPU 驗證腳本 - 檢測 NVIDIA GPU 和 CUDA 可用性
"""

import subprocess
import sys
import os

def run_command(cmd):
    """運行命令並返回結果"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.returncode == 0, result.stdout.strip(), result.stderr.strip()
    except Exception as e:
        return False, "", str(e)

def test_nvidia_smi():
    """測試 nvidia-smi 命令"""
    print("🔍 測試 nvidia-smi...")
    success, stdout, stderr = run_command("nvidia-smi")
    if success:
        print("✅ nvidia-smi 可用")
        lines = stdout.split('\n')
        for line in lines:
            if 'GeForce' in line or 'RTX' in line or 'GTX' in line:
                print(f"   GPU: {line.strip()}")
        return True
    else:
        print("❌ nvidia-smi 失敗")
        print(f"   錯誤: {stderr}")
        return False

def test_nvcc():
    """測試 CUDA 編譯器"""
    print("\n🔍 測試 nvcc...")
    success, stdout, stderr = run_command("nvcc --version")
    if success:
        print("✅ CUDA 編譯器可用")
        for line in stdout.split('\n'):
            if 'release' in line.lower():
                print(f"   版本: {line.strip()}")
        return True
    else:
        print("❌ nvcc 不可用")
        return False

def test_cuda_libs():
    """檢查 CUDA 庫"""
    print("\n🔍 檢查 CUDA 庫...")
    cuda_paths = [
        "/usr/local/cuda/lib64",
        "/usr/lib/x86_64-linux-gnu",
        "/opt/cuda/lib64"
    ]
    
    found_libs = []
    for path in cuda_paths:
        if os.path.exists(path):
            for lib in ['libcudart.so', 'libcublas.so', 'libcufft.so']:
                lib_path = os.path.join(path, lib)
                if os.path.exists(lib_path) or any(
                    os.path.exists(os.path.join(path, f)) 
                    for f in os.listdir(path) 
                    if f.startswith(lib.split('.')[0])
                ):
                    found_libs.append(f"{lib} in {path}")
    
    if found_libs:
        print("✅ 找到 CUDA 庫:")
        for lib in found_libs:
            print(f"   {lib}")
        return True
    else:
        print("❌ 未找到 CUDA 庫")
        return False

def test_container_gpu_access():
    """測試容器 GPU 訪問"""
    print("\n🔍 測試容器 GPU 訪問...")
    cmd = "podman run --rm --device nvidia.com/gpu=all docker.io/nvidia/cuda:12.2-base-ubuntu20.04 nvidia-smi"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("✅ 容器可以訪問 GPU")
        return True
    else:
        print("❌ 容器無法訪問 GPU")
        print(f"   錯誤: {stderr}")
        return False

def main():
    print("🚀 Care Voice GPU 環境驗證")
    print("=" * 50)
    
    results = []
    results.append(("NVIDIA SMI", test_nvidia_smi()))
    results.append(("CUDA 編譯器", test_nvcc()))
    results.append(("CUDA 庫", test_cuda_libs()))
    results.append(("容器 GPU 訪問", test_container_gpu_access()))
    
    print("\n📊 驗證結果總結:")
    print("=" * 50)
    all_passed = True
    for test_name, passed in results:
        status = "✅ 通過" if passed else "❌ 失敗"
        print(f"{test_name:<20} {status}")
        if not passed:
            all_passed = False
    
    print("\n🎯 總體結果:")
    if all_passed:
        print("✅ GPU 環境完全可用，可以進行 GPU 加速構建")
        return 0
    else:
        print("⚠️  部分 GPU 功能不可用，建議檢查以上失敗項目")
        return 1

if __name__ == "__main__":
    sys.exit(main())