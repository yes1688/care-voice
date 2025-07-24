#!/usr/bin/env python3
"""
GPU vs CPU 性能對比測試
"""

import time
import requests
import json

def test_endpoint(url, description, iterations=5):
    """測試指定端點的性能"""
    print(f"\n🔍 測試 {description}")
    print("=" * 50)
    
    # 健康檢查
    try:
        health_response = requests.get(f"{url}/health", timeout=10)
        if health_response.status_code == 200:
            health_data = health_response.json()
            print(f"✅ 服務狀態: {health_data['status']}")
            print(f"   設備: {health_data.get('device', 'unknown')}")
            print(f"   GPU 可用: {health_data.get('gpu_available', False)}")
            print(f"   CUDA 設備數: {health_data.get('cuda_device_count', 0)}")
        else:
            print(f"❌ 健康檢查失敗: {health_response.status_code}")
            return None
    except Exception as e:
        print(f"❌ 連接失敗: {e}")
        return None
    
    # 性能測試
    times = []
    print(f"\n📊 進行 {iterations} 次性能測試...")
    
    for i in range(iterations):
        start_time = time.time()
        try:
            # 模擬音頻上傳 (使用空數據)
            files = {'audio': ('test.webm', b'test audio data', 'audio/webm')}
            response = requests.post(f"{url}/api/upload", files=files, timeout=30)
            
            end_time = time.time()
            duration = end_time - start_time
            times.append(duration)
            
            if response.status_code == 200:
                print(f"   測試 {i+1}: {duration:.3f}s ✅")
            else:
                print(f"   測試 {i+1}: {duration:.3f}s ❌ (HTTP {response.status_code})")
        
        except Exception as e:
            end_time = time.time()
            duration = end_time - start_time
            print(f"   測試 {i+1}: {duration:.3f}s ❌ (錯誤: {e})")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        
        print(f"\n📈 性能統計:")
        print(f"   平均時間: {avg_time:.3f}s")
        print(f"   最快時間: {min_time:.3f}s")
        print(f"   最慢時間: {max_time:.3f}s")
        
        return {
            'avg': avg_time,
            'min': min_time,
            'max': max_time,
            'times': times,
            'device': health_data.get('device', 'unknown'),
            'gpu_available': health_data.get('gpu_available', False)
        }
    
    return None

def main():
    print("🚀 Care Voice GPU vs CPU 性能對比測試")
    print("=" * 60)
    
    # 測試配置
    endpoints = [
        ("http://localhost:8005", "GPU 加速版本"),
        ("http://localhost:8006", "CPU 版本")
    ]
    
    results = {}
    
    # 執行測試
    for url, desc in endpoints:
        result = test_endpoint(url, desc, iterations=3)
        if result:
            results[desc] = result
    
    # 對比結果
    if len(results) >= 2:
        print("\n🎯 性能對比總結")
        print("=" * 60)
        
        gpu_result = results.get("GPU 加速版本")
        cpu_result = results.get("CPU 版本")
        
        if gpu_result and cpu_result:
            gpu_avg = gpu_result['avg']
            cpu_avg = cpu_result['avg']
            speedup = cpu_avg / gpu_avg if gpu_avg > 0 else 0
            
            print(f"GPU 版本平均時間: {gpu_avg:.3f}s")
            print(f"CPU 版本平均時間: {cpu_avg:.3f}s")
            print(f"GPU 加速倍數: {speedup:.2f}x")
            
            if speedup > 1:
                print(f"🏆 GPU 版本快 {speedup:.2f} 倍！")
            elif speedup < 1:
                print(f"⚠️  CPU 版本快 {1/speedup:.2f} 倍")
            else:
                print("🤔 性能相當")
        
        # 詳細數據
        print(f"\n📋 詳細結果:")
        for name, result in results.items():
            print(f"\n{name}:")
            print(f"  設備: {result['device']}")
            print(f"  GPU: {'是' if result['gpu_available'] else '否'}")
            print(f"  時間: {result['times']}")
    
    print(f"\n✅ 測試完成！")

if __name__ == "__main__":
    main()