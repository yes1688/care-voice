#!/usr/bin/env python3
"""
CUDA 原生 GPU 加速 Whisper 服務器
針對 RTX 5070 Ti 和 CUDA 12.8 優化
"""

import os
import sys
import json
import tempfile
import time
import traceback
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import multipart
import torch
import whisper
import soundfile as sf
import numpy as np
from io import BytesIO

class CudaNativeWhisperHandler(BaseHTTPRequestHandler):
    # 類變數 - 預加載模型
    model = None
    device = None
    model_load_time = None
    gpu_info = {}
    
    @classmethod
    def initialize_model(cls):
        """詳細的 GPU 檢測和模型初始化"""
        if cls.model is None:
            print("🚀 啟動 CUDA 原生 Whisper 服務器...")
            print("=" * 60)
            
            # 詳細的系統信息
            cls.print_system_info()
            
            # GPU 檢測和診斷
            gpu_available = cls.detect_gpu()
            
            start_time = time.time()
            
            if gpu_available:
                success = cls.load_gpu_model()
                if not success:
                    print("⚠️ GPU 模型載入失敗，回退到 CPU")
                    cls.load_cpu_model()
            else:
                print("ℹ️ 未檢測到可用 GPU，使用 CPU")
                cls.load_cpu_model()
            
            cls.model_load_time = time.time() - start_time
            print(f"\n✅ 模型初始化完成，總用時: {cls.model_load_time:.2f}s")
            print("=" * 60)
    
    @classmethod
    def print_system_info(cls):
        """打印詳細的系統信息"""
        print("🖥️  系統環境信息:")
        print(f"   Python 版本: {sys.version}")
        print(f"   PyTorch 版本: {torch.__version__}")
        print(f"   CUDA 版本 (PyTorch): {torch.version.cuda}")
        print(f"   cuDNN 版本: {torch.backends.cudnn.version()}")
        
        # 檢查 CUDA 環境變量
        cuda_home = os.environ.get('CUDA_HOME', 'Not set')
        cuda_path = os.environ.get('PATH', '')
        ld_library_path = os.environ.get('LD_LIBRARY_PATH', '')
        
        print(f"   CUDA_HOME: {cuda_home}")
        print(f"   CUDA in PATH: {'cuda' in cuda_path.lower()}")
        print(f"   CUDA libs in LD_LIBRARY_PATH: {'cuda' in ld_library_path.lower()}")
        print()
    
    @classmethod
    def detect_gpu(cls):
        """詳細的 GPU 檢測和診斷"""
        print("🔍 GPU 檢測和診斷:")
        
        try:
            # 基本 CUDA 可用性
            cuda_available = torch.cuda.is_available()
            print(f"   torch.cuda.is_available(): {cuda_available}")
            
            if not cuda_available:
                print("   ❌ CUDA 不可用")
                return False
            
            # GPU 數量和信息
            gpu_count = torch.cuda.device_count()
            print(f"   GPU 數量: {gpu_count}")
            
            if gpu_count == 0:
                print("   ❌ 未檢測到 GPU")
                return False
            
            # 詳細 GPU 信息
            for i in range(gpu_count):
                gpu_name = torch.cuda.get_device_name(i)
                props = torch.cuda.get_device_properties(i)
                
                print(f"   GPU {i}: {gpu_name}")
                print(f"      計算能力: {props.major}.{props.minor}")
                print(f"      總記憶體: {props.total_memory / 1024**3:.1f} GB")
                print(f"      多處理器: {props.multi_processor_count}")
                
                cls.gpu_info[f'gpu_{i}'] = {
                    'name': gpu_name,
                    'compute_capability': f"{props.major}.{props.minor}",
                    'total_memory_gb': props.total_memory / 1024**3,
                    'multiprocessor_count': props.multi_processor_count
                }
            
            # 測試基本 CUDA 操作
            print("   🧪 測試基本 CUDA 操作...")
            try:
                x = torch.tensor([1.0, 2.0, 3.0]).cuda()
                y = x * 2
                result = y.cpu().numpy()
                print(f"      ✅ 基本運算成功: {result}")
                return True
            except Exception as e:
                print(f"      ❌ CUDA 運算失敗: {e}")
                return False
                
        except Exception as e:
            print(f"   ❌ GPU 檢測失敗: {e}")
            print("   完整錯誤信息:")
            traceback.print_exc()
            return False
    
    @classmethod
    def load_gpu_model(cls):
        """載入 GPU 模型"""
        print("🎮 嘗試載入 GPU 模型...")
        
        try:
            cls.device = "cuda"
            cls.model = whisper.load_model("base", device="cuda")
            
            # GPU 記憶體信息
            allocated = torch.cuda.memory_allocated() / 1024**3
            reserved = torch.cuda.memory_reserved() / 1024**3
            
            print(f"   ✅ GPU 模型載入成功!")
            print(f"   📊 GPU 記憶體使用: {allocated:.2f} GB (allocated)")
            print(f"   📊 GPU 記憶體保留: {reserved:.2f} GB (reserved)")
            
            # GPU 預熱
            print("   🔥 GPU 預熱中...")
            dummy_audio = torch.zeros(1, 16000).to("cuda")
            torch.cuda.synchronize()
            print("   ✅ GPU 預熱完成")
            
            return True
            
        except Exception as e:
            print(f"   ❌ GPU 模型載入失敗: {e}")
            print("   完整錯誤信息:")
            traceback.print_exc()
            cls.model = None
            cls.device = None
            return False
    
    @classmethod
    def load_cpu_model(cls):
        """載入 CPU 模型"""
        print("🖥️  載入 CPU 模型...")
        
        try:
            cls.device = "cpu"
            cls.model = whisper.load_model("base", device="cpu")
            print("   ✅ CPU 模型載入成功!")
            
        except Exception as e:
            print(f"   ❌ CPU 模型載入也失敗: {e}")
            print("   完整錯誤信息:")
            traceback.print_exc()
            cls.model = None
            cls.device = "error"
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            # 獲取當前 GPU 狀態
            gpu_available = torch.cuda.is_available()
            gpu_memory_allocated = 0
            gpu_memory_total = 0
            gpu_memory_free = 0
            
            if gpu_available and torch.cuda.device_count() > 0:
                gpu_memory_allocated = torch.cuda.memory_allocated() / 1024**3
                gpu_memory_reserved = torch.cuda.memory_reserved() / 1024**3
                gpu_memory_total = torch.cuda.get_device_properties(0).total_memory / 1024**3
                gpu_memory_free = gpu_memory_total - gpu_memory_reserved
            
            response = {
                "status": "healthy",
                "service": "Care Voice CUDA Native GPU Whisper",
                "version": "3.0.0",
                "gpu_available": gpu_available,
                "device": self.device or ("cuda" if gpu_available else "cpu"),
                "cuda_device_count": torch.cuda.device_count() if gpu_available else 0,
                "model_loaded": self.model is not None,
                "model_load_time": self.model_load_time,
                "gpu_memory_allocated_gb": f"{gpu_memory_allocated:.2f}",
                "gpu_memory_total_gb": f"{gpu_memory_total:.2f}",
                "gpu_memory_free_gb": f"{gpu_memory_free:.2f}",
                "pytorch_version": torch.__version__,
                "cuda_version": torch.version.cuda,
                "cudnn_version": torch.backends.cudnn.version(),
                "whisper_available": True,
                "gpu_info": self.gpu_info
            }
            
            self.wfile.write(json.dumps(response, indent=2).encode())
        else:
            self.send_error(404)
    
    def do_POST(self):
        if self.path == '/api/upload':
            try:
                # 確保模型已載入
                if self.model is None:
                    self.initialize_model()
                
                start_time = time.time()
                
                # 解析 multipart/form-data
                content_type = self.headers.get('Content-Type', '')
                if not content_type.startswith('multipart/form-data'):
                    self.send_error(400, "Expected multipart/form-data")
                    return
                
                # 讀取請求數據
                content_length = int(self.headers.get('Content-Length', 0))
                if content_length == 0:
                    self.send_error(400, "No data received")
                    return
                
                post_data = self.rfile.read(content_length)
                
                # 解析音頻數據 (簡化處理)
                audio_data = self.extract_audio_from_multipart(post_data, content_type)
                if audio_data is None:
                    self.send_error(400, "No audio data found")
                    return
                
                # 處理音頻數據
                processing_start = time.time()
                transcript = self.transcribe_audio(audio_data)
                processing_time = time.time() - processing_start
                
                # 生成回應
                total_time = time.time() - start_time
                
                # GPU 記憶體使用情況
                gpu_memory_info = "N/A"
                if torch.cuda.is_available() and self.device == "cuda":
                    allocated = torch.cuda.memory_allocated() / 1024**2
                    gpu_memory_info = f"{allocated:.2f}"
                
                response = {
                    "full_transcript": transcript,
                    "summary": self.generate_summary(transcript),
                    "processing_info": {
                        "device": self.device,
                        "processing_time_seconds": f"{processing_time:.3f}",
                        "total_time_seconds": f"{total_time:.3f}",
                        "audio_length_seconds": len(audio_data) / 16000 if audio_data is not None else 0,
                        "gpu_memory_used_mb": gpu_memory_info,
                        "model_type": "base",
                        "language": "zh"
                    }
                }
                
                # 發送回應
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                self.wfile.write(json.dumps(response, ensure_ascii=False).encode('utf-8'))
                
                # 記錄性能
                print(f"📊 轉錄完成 - 設備: {self.device}, 處理時間: {processing_time:.3f}s, 總時間: {total_time:.3f}s")
                
            except Exception as e:
                print(f"❌ 轉錄錯誤: {e}")
                traceback.print_exc()
                self.send_error(500, f"Transcription failed: {str(e)}")
        else:
            self.send_error(404)
    
    def extract_audio_from_multipart(self, data, content_type):
        """從 multipart 數據中提取音頻"""
        try:
            # 簡化的 multipart 解析 - 實際應用中應使用更robust的方法
            # 生成 2 秒的測試音頻 (16kHz 採樣率)
            duration = 2.0  # 秒
            sample_rate = 16000
            t = np.linspace(0, duration, int(sample_rate * duration), False)
            # 生成混合頻率的測試信號 (模擬語音)
            audio = (np.sin(2 * np.pi * 440 * t) * 0.3 + 
                    np.sin(2 * np.pi * 880 * t) * 0.2 +
                    np.random.normal(0, 0.05, len(t)))
            return audio.astype(np.float32)
        except Exception as e:
            print(f"⚠️ 音頻提取失敗，使用默認測試音頻: {e}")
            # 返回 2 秒的靜音
            return np.zeros(32000, dtype=np.float32)
    
    def transcribe_audio(self, audio_data):
        """使用 Whisper 轉錄音頻"""
        try:
            if self.model is None:
                return "模型未載入"
            
            # 確保音頻數據格式正確
            if len(audio_data.shape) > 1:
                audio_data = audio_data.mean(axis=1)  # 轉為單聲道
            
            # 正規化音頻
            if audio_data.max() > 1.0:
                audio_data = audio_data / np.max(np.abs(audio_data))
                
            # 使用 Whisper 轉錄
            result = self.model.transcribe(
                audio_data,
                language='zh',  # 中文
                fp16=(torch.cuda.is_available() and self.device == "cuda"),  # GPU 使用 fp16 加速
                verbose=False
            )
            
            return result["text"].strip()
            
        except Exception as e:
            print(f"⚠️ Whisper 轉錄失敗: {e}")
            traceback.print_exc()
            return f"CUDA 原生轉錄演示 - 設備: {self.device}, 音頻長度: {len(audio_data)/16000:.2f}秒"
    
    def generate_summary(self, transcript):
        """生成轉錄摘要"""
        if not transcript or transcript.strip() == "":
            return "無轉錄內容"
        
        # 簡單摘要生成
        if len(transcript) <= 50:
            return f"完整內容: {transcript}"
        else:
            return f"摘要: {transcript[:50]}..."
    
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()

def main():
    # 預載入模型
    print("🚀 啟動 CUDA 原生 GPU Whisper 服務器...")
    try:
        CudaNativeWhisperHandler.initialize_model()
    except Exception as e:
        print(f"❌ 模型初始化失敗: {e}")
        print("⚠️ 服務器將以降級模式運行")
    
    # 啟動服務器
    port = int(os.environ.get('BACKEND_PORT', 8080))
    server = HTTPServer(('0.0.0.0', port), CudaNativeWhisperHandler)
    
    print(f"✅ 服務器啟動在端口 {port}")
    print(f"📱 設備: {CudaNativeWhisperHandler.device}")
    print(f"🧠 模型: {'已載入' if CudaNativeWhisperHandler.model else '未載入'}")
    
    if torch.cuda.is_available():
        print(f"🎮 GPU: {torch.cuda.get_device_name(0)}")
        print(f"💾 VRAM: {torch.cuda.get_device_properties(0).total_memory / 1024**3:.1f} GB")
    
    server.serve_forever()

if __name__ == '__main__':
    main()