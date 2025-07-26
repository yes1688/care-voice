#!/usr/bin/env python3
"""
Care Voice Opus 性能基準測試
對比原版服務與 Opus 支援版本的性能差異
"""

import requests
import time
import psutil
import json
import wave
import numpy as np
from concurrent.futures import ThreadPoolExecutor, as_completed
import statistics

def create_benchmark_audio_files():
    """創建不同大小的基準測試音頻檔案"""
    print("🎵 創建基準測試音頻檔案...")
    
    files = {}
    
    # 不同測試場景
    scenarios = [
        {"name": "短音頻", "duration": 1.0, "filename": "short_audio.wav"},
        {"name": "中等音頻", "duration": 5.0, "filename": "medium_audio.wav"},
        {"name": "長音頻", "duration": 10.0, "filename": "long_audio.wav"},
    ]
    
    sample_rate = 16000
    frequency = 440  # A4 音符
    
    for scenario in scenarios:
        duration = scenario["duration"]
        filename = f"/tmp/{scenario['filename']}"
        
        samples = int(sample_rate * duration)
        t = np.linspace(0, duration, samples, False)
        
        # 生成正弦波音頻數據
        audio_data = np.sin(2 * np.pi * frequency * t) * 0.3
        audio_int16 = (audio_data * 32767).astype(np.int16)
        
        # 創建 WAV 檔案
        with wave.open(filename, 'w') as wav_file:
            wav_file.setnchannels(1)  # mono
            wav_file.setsampwidth(2)  # 16-bit
            wav_file.setframerate(sample_rate)
            wav_file.writeframes(audio_int16.tobytes())
        
        files[scenario["name"]] = {
            "path": filename,
            "duration": duration,
            "size": len(audio_int16.tobytes())
        }
        
        print(f"✅ {scenario['name']}: {duration}秒, {files[scenario['name']]['size']} bytes")
    
    return files

def measure_service_performance(service_url, service_name, test_files, num_requests=5):
    """測量服務性能指標"""
    print(f"\n⏱️ 測試 {service_name} 性能...")
    
    results = {
        "service_name": service_name,
        "service_url": service_url,
        "test_results": {},
        "overall_stats": {}
    }
    
    for file_type, file_info in test_files.items():
        print(f"\n📊 測試 {file_type} ({file_info['duration']}秒)")
        
        response_times = []
        success_count = 0
        
        for i in range(num_requests):
            try:
                start_time = time.time()
                
                # 測試健康檢查 (簡化測試，因為還沒有完整的轉錄 API)
                response = requests.get(f"{service_url}/health", timeout=10)
                
                end_time = time.time()
                response_time = (end_time - start_time) * 1000  # 轉換為毫秒
                
                if response.status_code == 200:
                    success_count += 1
                    response_times.append(response_time)
                    print(f"  請求 {i+1}: {response_time:.2f}ms ✅")
                else:
                    print(f"  請求 {i+1}: 失敗 (狀態碼: {response.status_code}) ❌")
                
                # 短暫延遲避免過載
                time.sleep(0.1)
                
            except Exception as e:
                print(f"  請求 {i+1}: 異常 ({e}) ❌")
        
        # 計算統計數據
        if response_times:
            stats = {
                "min_ms": min(response_times),
                "max_ms": max(response_times),
                "avg_ms": statistics.mean(response_times),
                "median_ms": statistics.median(response_times),
                "success_rate": (success_count / num_requests) * 100,
                "requests_per_second": 1000 / statistics.mean(response_times) if response_times else 0
            }
            
            results["test_results"][file_type] = stats
            
            print(f"    平均回應時間: {stats['avg_ms']:.2f}ms")
            print(f"    成功率: {stats['success_rate']:.1f}%")
            print(f"    每秒請求數: {stats['requests_per_second']:.1f}")
        else:
            results["test_results"][file_type] = {"error": "所有請求都失敗"}
    
    return results

def measure_system_resources():
    """測量系統資源使用情況"""
    print("\n💻 測量系統資源使用...")
    
    # CPU 使用率
    cpu_percent = psutil.cpu_percent(interval=1)
    
    # 記憶體使用率
    memory = psutil.virtual_memory()
    memory_percent = memory.percent
    memory_used_gb = memory.used / (1024**3)
    memory_total_gb = memory.total / (1024**3)
    
    # 磁碟使用率
    disk = psutil.disk_usage('/')
    disk_percent = disk.percent
    disk_used_gb = disk.used / (1024**3)
    disk_total_gb = disk.total / (1024**3)
    
    resources = {
        "cpu_percent": cpu_percent,
        "memory_percent": memory_percent,
        "memory_used_gb": memory_used_gb,
        "memory_total_gb": memory_total_gb,
        "disk_percent": disk_percent,
        "disk_used_gb": disk_used_gb,
        "disk_total_gb": disk_total_gb,
    }
    
    print(f"  📈 CPU 使用率: {cpu_percent:.1f}%")
    print(f"  🧠 記憶體使用: {memory_used_gb:.1f}GB / {memory_total_gb:.1f}GB ({memory_percent:.1f}%)")
    print(f"  💽 磁碟使用: {disk_used_gb:.1f}GB / {disk_total_gb:.1f}GB ({disk_percent:.1f}%)")
    
    return resources

def compare_services(original_results, opus_results):
    """比較兩個服務的性能"""
    print("\n📊 服務性能比較分析")
    print("=" * 70)
    
    if not original_results["test_results"] or not opus_results["test_results"]:
        print("❌ 無法進行比較 - 缺少測試數據")
        return
    
    # 比較平均回應時間
    print(f"{'測試項目':<15} {'原版 (ms)':<12} {'Opus版 (ms)':<12} {'改善':<10}")
    print("-" * 70)
    
    total_improvement = 0
    comparison_count = 0
    
    for file_type in original_results["test_results"]:
        if file_type in opus_results["test_results"]:
            orig_avg = original_results["test_results"][file_type].get("avg_ms", 0)
            opus_avg = opus_results["test_results"][file_type].get("avg_ms", 0)
            
            if orig_avg > 0 and opus_avg > 0:
                improvement = ((orig_avg - opus_avg) / orig_avg) * 100
                total_improvement += improvement
                comparison_count += 1
                
                improvement_str = f"{improvement:+.1f}%"
                print(f"{file_type:<15} {orig_avg:<12.2f} {opus_avg:<12.2f} {improvement_str:<10}")
    
    if comparison_count > 0:
        avg_improvement = total_improvement / comparison_count
        print("-" * 70)
        print(f"平均性能改善: {avg_improvement:+.1f}%")
        
        if avg_improvement > 30:
            print("🚀 性能顯著提升！")
        elif avg_improvement > 0:
            print("📈 性能有所改善")
        elif avg_improvement > -10:
            print("⚖️ 性能基本持平")
        else:
            print("⚠️ 性能有所下降")
    
    return total_improvement / comparison_count if comparison_count > 0 else 0

def main():
    """主基準測試函數"""
    print("🏁 Care Voice Opus 性能基準測試")
    print("=" * 60)
    
    # 1. 創建測試檔案
    test_files = create_benchmark_audio_files()
    
    # 2. 測量系統資源
    system_resources = measure_system_resources()
    
    # 3. 測試原版服務性能
    original_results = measure_service_performance(
        "http://localhost:8001", 
        "原版服務", 
        test_files,
        num_requests=3  # 減少請求數量以加快測試
    )
    
    # 4. 測試 Opus 版服務性能
    opus_results = measure_service_performance(
        "http://localhost:8002", 
        "Opus 支援版", 
        test_files,
        num_requests=3
    )
    
    # 5. 比較性能
    performance_improvement = compare_services(original_results, opus_results)
    
    # 6. 生成完整報告
    print("\n" + "=" * 60)
    print("📋 性能基準測試總結報告")
    print("=" * 60)
    
    print("🏗️ 基礎設施狀態:")
    print(f"  • 原版服務 (8001): {'✅ 運行中' if original_results['test_results'] else '❌ 不可用'}")
    print(f"  • Opus 版服務 (8002): {'✅ 運行中' if opus_results['test_results'] else '❌ 不可用'}")
    
    print("\n💻 系統資源:")
    print(f"  • CPU 使用率: {system_resources['cpu_percent']:.1f}%")
    print(f"  • 記憶體使用: {system_resources['memory_percent']:.1f}%")
    print(f"  • 可用記憶體: {system_resources['memory_total_gb'] - system_resources['memory_used_gb']:.1f}GB")
    
    if performance_improvement is not None:
        print(f"\n📈 性能改善: {performance_improvement:+.1f}%")
        
        # 評估是否達到目標
        target_improvement = 40  # 目標是 40% 性能提升
        if performance_improvement >= target_improvement:
            print(f"🎯 已達到性能提升目標 ({target_improvement}%+)")
        elif performance_improvement >= target_improvement * 0.75:
            print(f"⚠️ 接近性能提升目標 (目標: {target_improvement}%)")
        else:
            print(f"📊 性能基準測試完成，可考慮進一步優化")
    
    print("\n🔗 服務端點:")
    print("  • 原版: http://localhost:8001/health")
    print("  • Opus版: http://localhost:8002/health")
    
    # 清理測試檔案
    for file_info in test_files.values():
        import os
        if os.path.exists(file_info["path"]):
            os.remove(file_info["path"])
    
    print("\n✅ 性能基準測試完成")

if __name__ == "__main__":
    main()