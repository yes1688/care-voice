#!/usr/bin/env python3
"""
GPU 加速 Whisper 性能基準測試
測試不同音頻長度和格式的轉錄性能
"""

import time
import requests
import json
import numpy as np
import soundfile as sf
from io import BytesIO
import os

class GPUPerformanceBenchmark:
    def __init__(self, base_url="http://localhost:8007"):
        self.base_url = base_url
        self.results = []
    
    def check_service_health(self):
        """檢查服務健康狀況"""
        try:
            response = requests.get(f"{self.base_url}/health", timeout=10)
            if response.status_code == 200:
                return response.json()
            return None
        except Exception as e:
            print(f"❌ 服務健康檢查失敗: {e}")
            return None
    
    def generate_test_audio(self, duration_seconds, sample_rate=16000):
        """生成測試音頻數據"""
        t = np.linspace(0, duration_seconds, int(sample_rate * duration_seconds), False)
        # 生成混合頻率的測試信號 (模擬語音)
        frequency1, frequency2 = 440, 880  # A4 和 A5 音符
        audio = (np.sin(2 * np.pi * frequency1 * t) * 0.3 + 
                np.sin(2 * np.pi * frequency2 * t) * 0.2 +
                np.random.normal(0, 0.05, len(t)))  # 添加輕微噪音
        
        return audio.astype(np.float32)
    
    def audio_to_wav_bytes(self, audio_data, sample_rate=16000):
        """將音頻數據轉換為 WAV 格式的字節流"""
        buffer = BytesIO()
        sf.write(buffer, audio_data, sample_rate, format='WAV')
        buffer.seek(0)
        return buffer.getvalue()
    
    def benchmark_transcription(self, duration_seconds, test_name, iterations=3):
        """基準測試特定長度的音頻轉錄"""
        print(f"\n🎯 測試 {test_name} ({duration_seconds}秒音頻)")
        print("-" * 50)
        
        # 生成測試音頻
        audio_data = self.generate_test_audio(duration_seconds)
        wav_bytes = self.audio_to_wav_bytes(audio_data)
        
        times = []
        
        for i in range(iterations):
            start_time = time.time()
            
            try:
                files = {'audio': ('test.wav', wav_bytes, 'audio/wav')}
                response = requests.post(f"{self.base_url}/api/upload", files=files, timeout=60)
                
                end_time = time.time()
                duration = end_time - start_time
                times.append(duration)
                
                if response.status_code == 200:
                    result = response.json()
                    processing_info = result.get('processing_info', {})
                    print(f"   迭代 {i+1}: {duration:.3f}s (處理: {processing_info.get('processing_time_seconds', 'N/A')}s)")
                else:
                    print(f"   迭代 {i+1}: {duration:.3f}s ❌ HTTP {response.status_code}")
                    
            except Exception as e:
                end_time = time.time()
                duration = end_time - start_time
                print(f"   迭代 {i+1}: {duration:.3f}s ❌ 錯誤: {e}")
                times.append(duration)
        
        if times:
            avg_time = sum(times) / len(times)
            min_time = min(times)
            max_time = max(times)
            
            result = {
                'test_name': test_name,
                'duration_seconds': duration_seconds,
                'avg_time': avg_time,
                'min_time': min_time,
                'max_time': max_time,
                'times': times,
                'throughput_ratio': duration_seconds / avg_time if avg_time > 0 else 0
            }
            
            self.results.append(result)
            
            print(f"📈 平均時間: {avg_time:.3f}s")
            print(f"   最快時間: {min_time:.3f}s")
            print(f"   最慢時間: {max_time:.3f}s")
            print(f"   吞吐比率: {result['throughput_ratio']:.2f}x")
            
            return result
        
        return None
    
    def run_full_benchmark(self):
        """運行完整的性能基準測試"""
        print("🚀 GPU 加速 Whisper 性能基準測試")
        print("=" * 60)
        
        # 檢查服務狀態
        health = self.check_service_health()
        if not health:
            print("❌ 服務不可用，無法進行測試")
            return
        
        print(f"✅ 服務狀態: {health.get('status')}")
        print(f"   設備: {health.get('device')}")
        print(f"   GPU: {health.get('gpu_available')}")
        print(f"   模型: {'已載入' if health.get('model_loaded') else '未載入'}")
        print(f"   PyTorch: {health.get('pytorch_version')}")
        
        # 定義測試案例
        test_cases = [
            (1, "短音頻"),
            (5, "中等音頻"),
            (10, "長音頻"),
            (30, "超長音頻")
        ]
        
        # 執行各種長度的測試
        for duration, name in test_cases:
            self.benchmark_transcription(duration, name, iterations=3)
        
        # 生成綜合報告
        self.generate_report()
    
    def generate_report(self):
        """生成性能測試報告"""
        print("\n📊 性能測試綜合報告")
        print("=" * 60)
        
        if not self.results:
            print("❌ 沒有測試結果")
            return
        
        print(f"{'測試項目':<12} {'音頻長度':<8} {'平均時間':<10} {'吞吐比率':<10} {'效率'}")
        print("-" * 60)
        
        total_audio_time = 0
        total_processing_time = 0
        
        for result in self.results:
            total_audio_time += result['duration_seconds']
            total_processing_time += result['avg_time']
            
            efficiency = "🟢 優秀" if result['throughput_ratio'] > 5 else \
                        "🟡 良好" if result['throughput_ratio'] > 2 else \
                        "🔴 需優化"
            
            print(f"{result['test_name']:<12} {result['duration_seconds']:<8}s "
                  f"{result['avg_time']:<10.3f}s {result['throughput_ratio']:<10.2f}x {efficiency}")
        
        overall_throughput = total_audio_time / total_processing_time if total_processing_time > 0 else 0
        
        print("-" * 60)
        print(f"📈 總體性能:")
        print(f"   總音頻時長: {total_audio_time}s")
        print(f"   總處理時間: {total_processing_time:.3f}s")
        print(f"   總體吞吐比率: {overall_throughput:.2f}x")
        print(f"   平均處理效率: {'🟢 優秀' if overall_throughput > 5 else '🟡 良好' if overall_throughput > 2 else '🔴 需優化'}")
        
        # 保存結果到文件
        self.save_results()
    
    def save_results(self):
        """保存測試結果到文件"""
        filename = f"gpu_benchmark_results_{int(time.time())}.json"
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump({
                'timestamp': time.time(),
                'results': self.results,
                'summary': {
                    'total_tests': len(self.results),
                    'overall_throughput': sum(r['duration_seconds'] for r in self.results) / 
                                        sum(r['avg_time'] for r in self.results)
                }
            }, f, indent=2, ensure_ascii=False)
        
        print(f"\n💾 結果已保存到: {filename}")

def main():
    benchmark = GPUPerformanceBenchmark()
    benchmark.run_full_benchmark()

if __name__ == "__main__":
    main()