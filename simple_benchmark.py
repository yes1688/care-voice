#!/usr/bin/env python3
"""
簡化版 Whisper 效能測試 - 不依賴 PyTorch
"""

import time
import json
import subprocess
import requests
import os
import psutil

def get_basic_system_info():
    """獲取基本系統資訊"""
    return {
        "cpu_count": psutil.cpu_count(),
        "memory_gb": round(psutil.virtual_memory().total / (1024**3), 2),
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S")
    }

def test_rust_backend():
    """測試 Rust whisper-rs 後端"""
    print("🦀 測試 Rust whisper-rs 後端...")
    
    rust_process = None
    try:
        # 檢查是否已編譯
        if not os.path.exists("backend/target/release/care-voice"):
            print("  正在編譯 Rust 後端...")
            compile_start = time.time()
            compile_result = subprocess.run(
                ["cargo", "build", "--release"], 
                cwd="backend",
                capture_output=True,
                text=True,
                timeout=300  # 5分鐘編譯時間限制
            )
            compile_time = time.time() - compile_start
            
            if compile_result.returncode != 0:
                return {
                    "status": "compile_failed",
                    "error": compile_result.stderr[:500],  # 限制錯誤訊息長度
                    "compile_time": compile_time
                }
            print(f"  編譯完成，耗時: {compile_time:.1f}秒")
        else:
            print("  使用現有編譯版本")
        
        # 啟動服務
        print("  啟動 Rust 服務...")
        start_time = time.time()
        
        rust_process = subprocess.Popen(
            ["./target/release/care-voice"],
            cwd="backend",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # 等待服務啟動
        time.sleep(3)
        startup_time = time.time() - start_time
        
        # 測試健康檢查
        try:
            response = requests.get("http://localhost:8080/health", timeout=10)
            if response.status_code == 200:
                health_data = response.json()
                
                # 測量記憶體使用
                memory_usage = 0
                for proc in psutil.process_iter(['pid', 'name', 'memory_info']):
                    if 'care-voice' in proc.info['name']:
                        memory_usage = proc.info['memory_info'].rss / (1024**2)  # MB
                        break
                
                return {
                    "status": "success",
                    "startup_time_seconds": round(startup_time, 2),
                    "memory_usage_mb": round(memory_usage, 2),
                    "health_response": health_data,
                    "port": 8080
                }
            else:
                return {
                    "status": "health_check_failed",
                    "status_code": response.status_code
                }
                
        except requests.exceptions.RequestException as e:
            return {
                "status": "connection_failed",
                "error": str(e)
            }
            
    except subprocess.TimeoutExpired:
        return {
            "status": "compile_timeout",
            "error": "編譯超時（超過5分鐘）"
        }
    except Exception as e:
        return {
            "status": "error",
            "error": str(e)
        }
    finally:
        if rust_process:
            rust_process.terminate()
            try:
                rust_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                rust_process.kill()

def check_whisper_versions():
    """檢查當前的 Whisper 相關版本"""
    print("📦 檢查 Whisper 相關版本...")
    
    versions = {}
    
    # 檢查 Cargo.toml 中的 whisper-rs 版本
    try:
        with open("backend/Cargo.toml", 'r') as f:
            content = f.read()
            for line in content.split('\n'):
                if 'whisper-rs' in line and 'version' in line:
                    versions['whisper-rs'] = line.strip()
                    break
    except Exception as e:
        versions['whisper-rs'] = f"讀取失敗: {e}"
    
    # 檢查模型文件
    model_files = []
    for root, dirs, files in os.walk("."):
        for file in files:
            if file.endswith('.bin') and ('ggml' in file or 'whisper' in file):
                file_path = os.path.join(root, file)
                file_size = os.path.getsize(file_path) / (1024**2)  # MB
                model_files.append({
                    "path": file_path,
                    "size_mb": round(file_size, 1)
                })
    
    versions['model_files'] = model_files
    
    return versions

def main():
    print("🚀 Care Voice 簡化效能測試")
    print("=" * 50)
    
    results = {
        "system_info": get_basic_system_info(),
        "versions": check_whisper_versions(),
        "rust_test": test_rust_backend()
    }
    
    # 保存結果
    with open("simple_benchmark_results.json", 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    
    # 顯示結果
    print("\n📊 測試結果摘要")
    print("=" * 30)
    
    print(f"系統資訊:")
    print(f"  CPU 核心數: {results['system_info']['cpu_count']}")
    print(f"  記憶體: {results['system_info']['memory_gb']} GB")
    
    print(f"\nWhisper 版本:")
    print(f"  {results['versions']['whisper-rs']}")
    print(f"  模型文件: {len(results['versions']['model_files'])} 個")
    for model in results['versions']['model_files']:
        print(f"    - {model['path']} ({model['size_mb']} MB)")
    
    print(f"\nRust 後端測試:")
    rust_result = results['rust_test']
    if rust_result['status'] == 'success':
        print(f"  ✅ 啟動成功")
        print(f"  啟動時間: {rust_result['startup_time_seconds']} 秒")
        print(f"  記憶體使用: {rust_result['memory_usage_mb']} MB")
        if 'health_response' in rust_result:
            health = rust_result['health_response']
            print(f"  服務狀態: {health.get('status', 'unknown')}")
            print(f"  版本: {health.get('version', 'unknown')}")
    else:
        print(f"  ❌ 測試失敗: {rust_result['status']}")
        if 'error' in rust_result:
            print(f"  錯誤: {rust_result['error']}")
    
    print(f"\n📄 詳細結果已保存到: simple_benchmark_results.json")

if __name__ == "__main__":
    main()