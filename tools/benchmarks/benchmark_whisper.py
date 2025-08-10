#!/usr/bin/env python3
"""
Care Voice - Whisper 效能基准測試
比較 whisper-rs (Rust) vs PyTorch Whisper 的效能差異
"""

import time
import json
import psutil
import subprocess
import requests
import os
from pathlib import Path

class WhisperBenchmark:
    def __init__(self):
        self.results = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "system_info": self.get_system_info(),
            "tests": {}
        }
        
    def get_system_info(self):
        """獲取系統資訊"""
        import torch
        
        return {
            "cpu_count": psutil.cpu_count(),
            "memory_gb": round(psutil.virtual_memory().total / (1024**3), 2),
            "cuda_available": torch.cuda.is_available(),
            "cuda_device_count": torch.cuda.device_count() if torch.cuda.is_available() else 0,
            "cuda_device_name": torch.cuda.get_device_name(0) if torch.cuda.is_available() else None
        }

    def measure_memory_usage(self, process_name):
        """測量指定進程的記憶體使用量"""
        for proc in psutil.process_iter(['pid', 'name', 'memory_info']):
            if process_name in proc.info['name']:
                return proc.info['memory_info'].rss / (1024**2)  # MB
        return 0

    def test_rust_whisper(self, audio_file=None):
        """測試 Rust whisper-rs 版本"""
        print("🔧 測試 Rust whisper-rs 版本...")
        
        # 啟動 Rust 後端
        rust_process = None
        try:
            start_time = time.time()
            
            # 編譯並啟動 Rust 服務
            print("  編譯 Rust 後端...")
            compile_result = subprocess.run(
                ["cargo", "build", "--release"], 
                cwd="backend",
                capture_output=True,
                text=True,
                timeout=120
            )
            
            if compile_result.returncode != 0:
                return {"error": f"編譯失敗: {compile_result.stderr}"}
            
            compile_time = time.time() - start_time
            
            # 啟動服務
            print("  啟動 Rust 服務...")
            rust_process = subprocess.Popen(
                ["./target/release/care-voice"],
                cwd="backend",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            
            # 等待服務啟動
            time.sleep(3)
            
            # 測試健康檢查
            try:
                response = requests.get("http://localhost:8080/health", timeout=5)
                health_data = response.json()
            except Exception as e:
                return {"error": f"服務連接失敗: {e}"}
            
            # 測量記憶體使用
            memory_usage = self.measure_memory_usage("care-voice")
            
            # 如果有音頻文件，測試轉錄性能
            transcription_time = 0
            if audio_file and os.path.exists(audio_file):
                print("  測試音頻轉錄...")
                transcribe_start = time.time()
                
                with open(audio_file, 'rb') as f:
                    files = {'audio': f}
                    try:
                        response = requests.post(
                            "http://localhost:8080/upload", 
                            files=files,
                            timeout=30
                        )
                        transcription_time = time.time() - transcribe_start
                    except Exception as e:
                        transcription_time = -1  # 錯誤標記
            
            return {
                "compile_time_seconds": round(compile_time, 2),
                "memory_usage_mb": round(memory_usage, 2),
                "transcription_time_seconds": round(transcription_time, 2) if transcription_time > 0 else "N/A",
                "health_info": health_data,
                "status": "success"
            }
            
        except Exception as e:
            return {"error": str(e)}
        finally:
            if rust_process:
                rust_process.terminate()
                rust_process.wait(timeout=5)

    def test_python_whisper(self, audio_file=None):
        """測試 Python PyTorch Whisper 版本"""
        print("🐍 測試 Python PyTorch Whisper 版本...")
        
        python_process = None
        try:
            start_time = time.time()
            
            # 啟動 Python 服務
            python_process = subprocess.Popen(
                ["python3", "gpu_whisper_server.py"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=dict(os.environ, BACKEND_PORT="8081")
            )
            
            # 等待服務啟動
            time.sleep(3)
            
            # 測試健康檢查
            try:
                response = requests.get("http://localhost:8081/health", timeout=5)
                health_data = response.json()
            except Exception as e:
                return {"error": f"服務連接失敗: {e}"}
            
            startup_time = time.time() - start_time
            
            # 測量記憶體使用
            memory_usage = self.measure_memory_usage("python")
            
            # 如果有音頻文件，測試轉錄性能
            transcription_time = 0
            if audio_file and os.path.exists(audio_file):
                print("  測試音頻轉錄...")
                transcribe_start = time.time()
                
                with open(audio_file, 'rb') as f:
                    files = {'audio': f}
                    try:
                        response = requests.post(
                            "http://localhost:8081/upload", 
                            files=files,
                            timeout=30
                        )
                        transcription_time = time.time() - transcribe_start
                    except Exception as e:
                        transcription_time = -1  # 錯誤標記
            
            return {
                "startup_time_seconds": round(startup_time, 2),
                "memory_usage_mb": round(memory_usage, 2),
                "transcription_time_seconds": round(transcription_time, 2) if transcription_time > 0 else "N/A",
                "health_info": health_data,
                "status": "success"
            }
            
        except Exception as e:
            return {"error": str(e)}
        finally:
            if python_process:
                python_process.terminate()
                python_process.wait(timeout=5)

    def run_benchmark(self, audio_file=None):
        """執行完整基準測試"""
        print("🚀 開始 Care Voice Whisper 效能基準測試")
        print("=" * 50)
        
        # 測試 Rust 版本
        self.results["tests"]["rust_whisper"] = self.test_rust_whisper(audio_file)
        
        # 等待一下，避免端口衝突
        time.sleep(2)
        
        # 測試 Python 版本
        self.results["tests"]["python_whisper"] = self.test_python_whisper(audio_file)
        
        return self.results

    def save_results(self, filename="benchmark_results.json"):
        """保存測試結果"""
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(self.results, f, indent=2, ensure_ascii=False)
        print(f"📊 測試結果已保存到: {filename}")

    def print_comparison(self):
        """打印比較結果"""
        print("\n📊 效能比較結果")
        print("=" * 50)
        
        rust_result = self.results["tests"].get("rust_whisper", {})
        python_result = self.results["tests"].get("python_whisper", {})
        
        if rust_result.get("status") == "success" and python_result.get("status") == "success":
            print(f"記憶體使用:")
            print(f"  🦀 Rust whisper-rs: {rust_result.get('memory_usage_mb', 'N/A')} MB")
            print(f"  🐍 Python PyTorch:  {python_result.get('memory_usage_mb', 'N/A')} MB")
            
            # 計算記憶體效率差異
            if rust_result.get('memory_usage_mb') and python_result.get('memory_usage_mb'):
                memory_diff = python_result['memory_usage_mb'] / rust_result['memory_usage_mb']
                print(f"  📈 Python 使用記憶體為 Rust 的 {memory_diff:.1f} 倍")
            
            print(f"\n轉錄時間:")
            print(f"  🦀 Rust whisper-rs: {rust_result.get('transcription_time_seconds', 'N/A')} 秒")
            print(f"  🐍 Python PyTorch:  {python_result.get('transcription_time_seconds', 'N/A')} 秒")
            
        else:
            print("❌ 測試過程中出現錯誤，無法進行完整比較")
            if rust_result.get("error"):
                print(f"Rust 錯誤: {rust_result['error']}")
            if python_result.get("error"):
                print(f"Python 錯誤: {python_result['error']}")

def main():
    # 檢查是否有測試音頻文件
    test_audio = None
    audio_paths = [
        "test_audio.wav",
        "sample.wav", 
        "demo.wav",
        "backend/test_audio.wav"
    ]
    
    for path in audio_paths:
        if os.path.exists(path):
            test_audio = path
            break
    
    if not test_audio:
        print("⚠️  未找到測試音頻文件，將只進行基本服務測試")
        print("   建議放置一個 WAV 音頻文件來進行完整測試")
    else:
        print(f"🎵 使用測試音頻: {test_audio}")
    
    # 執行基準測試
    benchmark = WhisperBenchmark()
    results = benchmark.run_benchmark(test_audio)
    
    # 保存和顯示結果
    benchmark.save_results()
    benchmark.print_comparison()

if __name__ == "__main__":
    main()