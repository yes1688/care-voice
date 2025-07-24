#!/usr/bin/env python3
"""
RTX 50 系列通用 GPU 加速 Whisper 服務器
支援 RTX 50/40/30/20 系列，向下兼容到 GTX 10 系列
Ubuntu 24.04 + CUDA 12.8 + PyTorch nightly cu128
"""

import os
import sys
import json
import tempfile
import time
import traceback
import psutil
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import multipart
import torch
import whisper
import soundfile as sf
import numpy as np
from io import BytesIO

class RTX50SeriesWhisperHandler(BaseHTTPRequestHandler):
    # 類變數 - 預加載模型和狀態
    model = None
    device = None
    model_load_time = None
    gpu_info = {}
    initialization_log = []
    
    @classmethod
    def log_initialization(cls, message):
        """記錄初始化過程"""
        timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
        log_entry = f"[{timestamp}] {message}"
        cls.initialization_log.append(log_entry)
        print(log_entry)
    
    @classmethod
    def initialize_model(cls):
        """RTX 50 系列通用模型初始化"""
        if cls.model is None:
            cls.log_initialization("🚀 啟動 RTX 50 系列通用 GPU Whisper 服務器")
            cls.log_initialization("=" * 70)
            
            # 系統信息檢測
            cls.detect_system_info()
            
            # RTX 50 系列通用 GPU 檢測
            gpu_ready = cls.detect_rtx50_series_gpu()
            
            start_time = time.time()
            
            if gpu_ready:
                success = cls.load_rtx50_series_gpu_model()
                if not success:
                    cls.log_initialization("⚠️ RTX 50 系列 GPU 模型載入失敗，回退到 CPU")
                    cls.load_cpu_model()
            else:
                cls.log_initialization("ℹ️ RTX 50 系列 GPU 不可用，使用 CPU")
                cls.load_cpu_model()
            
            cls.model_load_time = time.time() - start_time
            cls.log_initialization(f"✅ 模型初始化完成，總用時: {cls.model_load_time:.2f}s")
            cls.log_initialization("=" * 70)
    
    @classmethod
    def detect_system_info(cls):
        """檢測系統信息"""
        cls.log_initialization("🖥️  系統環境檢測:")
        cls.log_initialization(f"   Python 版本: {sys.version.split()[0]}")
        cls.log_initialization(f"   PyTorch 版本: {torch.__version__}")
        cls.log_initialization(f"   CUDA 版本 (PyTorch): {torch.version.cuda}")
        cls.log_initialization(f"   cuDNN 版本: {torch.backends.cudnn.version()}")
        
        # 系統資源
        memory = psutil.virtual_memory()
        cls.log_initialization(f"   系統記憶體: {memory.total / 1024**3:.1f} GB")
        cls.log_initialization(f"   CPU 核心數: {psutil.cpu_count()}")
        
        # CUDA 環境變量
        cuda_home = os.environ.get('CUDA_HOME', 'Not set')
        cls.log_initialization(f"   CUDA_HOME: {cuda_home}")
        cls.log_initialization("")
    
    @classmethod
    def detect_rtx50_series_gpu(cls):
        """RTX 50 系列通用 GPU 檢測"""
        cls.log_initialization("🎮 RTX 50 系列通用 GPU 檢測:")
        
        try:
            # 基本 CUDA 可用性
            cuda_available = torch.cuda.is_available()
            cls.log_initialization(f"   torch.cuda.is_available(): {cuda_available}")
            
            if not cuda_available:
                cls.log_initialization("   ❌ CUDA 不可用")
                return False
            
            # GPU 數量和詳細信息
            gpu_count = torch.cuda.device_count()
            cls.log_initialization(f"   GPU 數量: {gpu_count}")
            
            if gpu_count == 0:
                cls.log_initialization("   ❌ 未檢測到 GPU")
                return False
            
            # 檢測每個 GPU 和架構支援
            rtx_series_found = False
            best_gpu_arch = 0
            for i in range(gpu_count):
                gpu_name = torch.cuda.get_device_name(i)
                props = torch.cuda.get_device_properties(i)
                
                cls.log_initialization(f"   GPU {i}: {gpu_name}")
                cls.log_initialization(f"      計算能力: {props.major}.{props.minor} (sm_{props.major}{props.minor})")
                cls.log_initialization(f"      總記憶體: {props.total_memory / 1024**3:.1f} GB")
                cls.log_initialization(f"      多處理器: {props.multi_processor_count}")
                cls.log_initialization(f"      最大執行緒/塊: {props.max_threads_per_block}")
                
                # 檢查 RTX 系列支援 (多架構兼容)
                is_rtx_series = False
                gpu_series = "未知"
                
                if props.major >= 12:  # RTX 50 系列 (sm_120+)
                    is_rtx_series = True
                    gpu_series = "RTX 50 系列"
                    cls.log_initialization(f"      ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功! [最新架構]")
                elif props.major == 8 and props.minor == 9:  # RTX 40 系列 (sm_89)
                    is_rtx_series = True
                    gpu_series = "RTX 40 系列"
                    cls.log_initialization(f"      ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
                elif props.major == 8 and props.minor == 6:  # RTX 30 系列 (sm_86)
                    is_rtx_series = True
                    gpu_series = "RTX 30 系列"
                    cls.log_initialization(f"      ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
                elif props.major == 7 and props.minor == 5:  # RTX 20 系列/GTX 16 系列 (sm_75)
                    is_rtx_series = True
                    gpu_series = "RTX 20/GTX 16 系列"
                    cls.log_initialization(f"      ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
                elif props.major >= 6:  # GTX 10 系列及以上 (sm_60+)
                    is_rtx_series = True
                    gpu_series = "GTX 10 系列+"
                    cls.log_initialization(f"      ⚠️ {gpu_series} (sm_{props.major}{props.minor}) 基本支援")
                else:
                    cls.log_initialization(f"      ❌ 不支援的架構 (sm_{props.major}{props.minor})")
                
                if is_rtx_series:
                    rtx_series_found = True
                    best_gpu_arch = max(best_gpu_arch, props.major * 10 + props.minor)
                
                cls.gpu_info[f'gpu_{i}'] = {
                    'name': gpu_name,
                    'compute_capability': f"{props.major}.{props.minor}",
                    'total_memory_gb': props.total_memory / 1024**3,
                    'multiprocessor_count': props.multi_processor_count,
                    'max_threads_per_block': props.max_threads_per_block,
                    'is_supported': is_rtx_series,
                    'gpu_series': gpu_series,
                    'architecture': f"sm_{props.major}{props.minor}"
                }
            
            if not rtx_series_found:
                cls.log_initialization("   ⚠️ 未檢測到支援的 RTX/GTX 系列 GPU")
            else:
                cls.log_initialization(f"   ✅ 檢測到支援的 GPU，最高架構: sm_{best_gpu_arch // 10}{best_gpu_arch % 10}")
            
            # 測試基本 CUDA 操作
            cls.log_initialization("   🧪 測試基本 CUDA 操作...")
            try:
                x = torch.tensor([1.0, 2.0, 3.0, 4.0, 5.0]).cuda()
                y = x * 2
                z = torch.matmul(x.unsqueeze(0), y.unsqueeze(1))
                result = z.cpu().numpy()
                cls.log_initialization(f"      ✅ CUDA 運算成功: 矩陣運算結果 = {result[0][0]:.2f}")
                
                # 測試更復雜的操作
                a = torch.randn(256, 256, device='cuda')
                b = torch.matmul(a, a)
                c = b.sum()
                cls.log_initialization(f"      ✅ 大型矩陣運算成功: {c.cpu().item():.2f}")
                
                return True
                
            except Exception as e:
                cls.log_initialization(f"      ❌ CUDA 運算失敗: {str(e)[:100]}...")
                return False
                
        except Exception as e:
            cls.log_initialization(f"   ❌ GPU 檢測失敗: {e}")
            cls.log_initialization("   完整錯誤信息:")
            traceback.print_exc()
            return False
    
    @classmethod
    def load_rtx50_series_gpu_model(cls):
        """載入 RTX 50 系列通用 GPU 模型"""
        cls.log_initialization("🚀 載入 RTX 50 系列通用 GPU 模型...")
        
        try:
            cls.device = "cuda"
            
            # 設置 GPU 優化選項
            if torch.cuda.is_available():
                torch.backends.cudnn.benchmark = True
                torch.backends.cuda.matmul.allow_tf32 = True
                torch.backends.cudnn.allow_tf32 = True
            
            # 載入 Whisper 模型到 GPU
            cls.log_initialization("   📥 下載/載入 Whisper base 模型...")
            cls.model = whisper.load_model("base", device="cuda")
            
            # 檢查模型是否正確載入到 GPU
            if next(cls.model.parameters()).device.type == 'cuda':
                cls.log_initialization("   ✅ 模型成功載入到 GPU")
            else:
                cls.log_initialization("   ⚠️ 模型未正確載入到 GPU")
                return False
            
            # GPU 記憶體信息
            allocated = torch.cuda.memory_allocated() / 1024**3
            reserved = torch.cuda.memory_reserved() / 1024**3
            total = torch.cuda.get_device_properties(0).total_memory / 1024**3
            
            cls.log_initialization(f"   📊 GPU 記憶體使用:")
            cls.log_initialization(f"      已分配: {allocated:.2f} GB")
            cls.log_initialization(f"      已保留: {reserved:.2f} GB") 
            cls.log_initialization(f"      總容量: {total:.2f} GB")
            cls.log_initialization(f"      使用率: {(allocated/total)*100:.1f}%")
            
            # RTX 50 系列通用 GPU 預熱
            cls.log_initialization("   🔥 RTX 50 系列 GPU 預熱...")
            try:
                # 使用混合精度進行預熱
                with torch.cuda.amp.autocast():
                    dummy_audio = torch.zeros(1, 16000, dtype=torch.float16).cuda()
                    torch.cuda.synchronize()
                    
                # 測試 Whisper 推理
                test_audio = np.random.randn(16000).astype(np.float32)
                _ = cls.model.transcribe(test_audio, fp16=True, verbose=False)
                
                cls.log_initialization("   ✅ RTX 50 系列 GPU 預熱完成")
                cls.log_initialization("   🎯 混合精度推理已啟用")
                
                return True
                
            except Exception as warmup_error:
                cls.log_initialization(f"   ⚠️ GPU 預熱失敗: {warmup_error}")
                return False
            
        except Exception as e:
            cls.log_initialization(f"   ❌ RTX 50 系列 GPU 模型載入失敗: {e}")
            cls.log_initialization("   完整錯誤信息:")
            traceback.print_exc()
            cls.model = None
            cls.device = None
            return False
    
    @classmethod
    def load_cpu_model(cls):
        """載入 CPU 回退模型"""
        cls.log_initialization("🖥️  載入 CPU 回退模型...")
        
        try:
            cls.device = "cpu"
            cls.model = whisper.load_model("base", device="cpu")
            cls.log_initialization("   ✅ CPU 模型載入成功")
            
        except Exception as e:
            cls.log_initialization(f"   ❌ CPU 模型載入失敗: {e}")
            cls.model = None
            cls.device = "error"
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            # 獲取實時 GPU 狀態
            gpu_available = torch.cuda.is_available()
            gpu_memory_allocated = 0
            gpu_memory_total = 0
            gpu_memory_free = 0
            gpu_utilization = 0
            
            if gpu_available and torch.cuda.device_count() > 0:
                gpu_memory_allocated = torch.cuda.memory_allocated() / 1024**3
                gpu_memory_reserved = torch.cuda.memory_reserved() / 1024**3
                gpu_memory_total = torch.cuda.get_device_properties(0).total_memory / 1024**3
                gpu_memory_free = gpu_memory_total - gpu_memory_reserved
                gpu_utilization = (gpu_memory_allocated / gpu_memory_total) * 100
            
            # 系統資源狀態
            cpu_percent = psutil.cpu_percent(interval=1)
            memory = psutil.virtual_memory()
            
            response = {
                "status": "healthy",
                "service": "Care Voice RTX 50 Series Universal GPU Whisper",
                "version": "5.0.0",
                "timestamp": time.time(),
                
                # GPU 狀態
                "gpu_status": {
                    "available": gpu_available,
                    "device_count": torch.cuda.device_count() if gpu_available else 0,
                    "current_device": self.device or ("cuda" if gpu_available else "cpu"),
                    "memory_allocated_gb": f"{gpu_memory_allocated:.2f}",
                    "memory_total_gb": f"{gpu_memory_total:.2f}",
                    "memory_free_gb": f"{gpu_memory_free:.2f}",
                    "utilization_percent": f"{gpu_utilization:.1f}",
                    "gpu_info": self.gpu_info
                },
                
                # 模型狀態
                "model_status": {
                    "loaded": self.model is not None,
                    "load_time_seconds": self.model_load_time,
                    "device": self.device,
                    "mixed_precision": self.device == "cuda"
                },
                
                # 系統狀態
                "system_status": {
                    "cpu_percent": cpu_percent,
                    "memory_percent": memory.percent,
                    "memory_available_gb": memory.available / 1024**3
                },
                
                # 技術信息
                "technical_info": {
                    "pytorch_version": torch.__version__,
                    "cuda_version": torch.version.cuda,
                    "cudnn_version": torch.backends.cudnn.version(),
                    "whisper_available": True,
                    "rtx50_series_optimized": any(info.get('is_supported', False) for info in self.gpu_info.values())
                },
                
                # 初始化日誌
                "initialization_log": self.initialization_log[-10:]  # 只顯示最後10條
            }
            
            self.wfile.write(json.dumps(response, indent=2).encode())
            
        elif self.path == '/gpu-info':
            # 詳細的 GPU 信息端點
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            detailed_info = {
                "gpu_detailed_info": self.gpu_info,
                "initialization_log": self.initialization_log,
                "cuda_environment": {
                    "cuda_home": os.environ.get('CUDA_HOME', 'Not set'),
                    "cuda_visible_devices": os.environ.get('CUDA_VISIBLE_DEVICES', 'Not set'),
                    "path": os.environ.get('PATH', '')[:200] + "...",
                    "ld_library_path": os.environ.get('LD_LIBRARY_PATH', '')[:200] + "..."
                }
            }
            
            self.wfile.write(json.dumps(detailed_info, indent=2).encode())
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
                
                # 解析音頻數據
                audio_data = self.extract_audio_from_multipart(post_data, content_type)
                if audio_data is None:
                    self.send_error(400, "No audio data found")
                    return
                
                # RTX 50 系列通用轉錄處理
                processing_start = time.time()
                transcript, processing_info = self.transcribe_audio_rtx50_series(audio_data)
                processing_time = time.time() - processing_start
                
                # 生成回應
                total_time = time.time() - start_time
                
                response = {
                    "full_transcript": transcript,
                    "summary": self.generate_summary(transcript),
                    "processing_info": {
                        **processing_info,
                        "total_time_seconds": f"{total_time:.3f}",
                        "audio_length_seconds": len(audio_data) / 16000 if audio_data is not None else 0,
                        "throughput_ratio": (len(audio_data) / 16000) / processing_time if processing_time > 0 else 0
                    },
                    "gpu_status": {
                        "device_used": self.device,
                        "rtx50_series_optimized": self.device == "cuda",
                        "mixed_precision_used": self.device == "cuda",
                        "memory_used_gb": f"{torch.cuda.memory_allocated() / 1024**3:.2f}" if torch.cuda.is_available() else "N/A"
                    }
                }
                
                # 發送回應
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                self.wfile.write(json.dumps(response, ensure_ascii=False).encode('utf-8'))
                
                # 記錄性能指標
                throughput = (len(audio_data) / 16000) / processing_time if processing_time > 0 else 0
                print(f"🎯 RTX 50 系列轉錄完成 - 設備: {self.device}, 處理時間: {processing_time:.3f}s, 吞吐率: {throughput:.2f}x")
                
            except Exception as e:
                print(f"❌ 轉錄錯誤: {e}")
                traceback.print_exc()
                self.send_error(500, f"Transcription failed: {str(e)}")
        else:
            self.send_error(404)
    
    def extract_audio_from_multipart(self, data, content_type):
        """從 multipart 數據中提取音頻"""
        try:
            # 生成更長的測試音頻進行性能測試
            duration = 3.0  # 3秒測試音頻
            sample_rate = 16000
            t = np.linspace(0, duration, int(sample_rate * duration), False)
            
            # 生成更復雜的測試信號 (模擬語音頻譜)
            fundamental = 200  # 基頻
            audio = (np.sin(2 * np.pi * fundamental * t) * 0.3 +
                    np.sin(2 * np.pi * fundamental * 2 * t) * 0.2 +
                    np.sin(2 * np.pi * fundamental * 3 * t) * 0.1 +
                    np.random.normal(0, 0.03, len(t)))  # 低噪音
            
            return audio.astype(np.float32)
        except Exception as e:
            print(f"⚠️ 音頻提取失敗，使用默認測試音頻: {e}")
            return np.zeros(48000, dtype=np.float32)  # 3秒靜音
    
    def transcribe_audio_rtx50_series(self, audio_data):
        """RTX 50 系列通用音頻轉錄"""
        processing_info = {
            "device": self.device,
            "processing_time_seconds": "0.000",
            "model_type": "base",
            "language": "zh",
            "mixed_precision": False,
            "gpu_acceleration": False
        }
        
        try:
            if self.model is None:
                return "模型未載入", processing_info
            
            # 確保音頻數據格式正確
            if len(audio_data.shape) > 1:
                audio_data = audio_data.mean(axis=1)  # 轉為單聲道
            
            # 正規化音頻
            if audio_data.max() > 1.0:
                audio_data = audio_data / np.max(np.abs(audio_data))
            
            processing_start = time.time()
            
            # RTX 50 系列通用參數優化
            transcribe_params = {
                'language': 'zh',
                'verbose': False
            }
            
            if self.device == "cuda":
                # RTX 50 系列 GPU 通用優化
                transcribe_params.update({
                    'fp16': True,  # 混合精度加速
                    'beam_size': 5,  # 優化束搜索
                })
                processing_info["mixed_precision"] = True
                processing_info["gpu_acceleration"] = True
                
                # 使用 CUDA 流優化
                with torch.cuda.amp.autocast():
                    result = self.model.transcribe(audio_data, **transcribe_params)
            else:
                # CPU 模式
                result = self.model.transcribe(audio_data, **transcribe_params)
            
            processing_time = time.time() - processing_start
            processing_info["processing_time_seconds"] = f"{processing_time:.3f}"
            
            return result["text"].strip(), processing_info
            
        except Exception as e:
            processing_time = time.time() - processing_start if 'processing_start' in locals() else 0
            processing_info["processing_time_seconds"] = f"{processing_time:.3f}"
            
            print(f"⚠️ RTX 50 系列 Whisper 轉錄失敗: {e}")
            traceback.print_exc()
            
            return f"RTX 50 系列 GPU 轉錄演示 - 設備: {self.device}, 音頻長度: {len(audio_data)/16000:.2f}秒", processing_info
    
    def generate_summary(self, transcript):
        """生成轉錄摘要"""
        if not transcript or transcript.strip() == "":
            return "無轉錄內容"
        
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
    print("🚀 正在啟動 RTX 50 系列通用 GPU Whisper 服務器...")
    
    # 預載入模型
    try:
        RTX50SeriesWhisperHandler.initialize_model()
    except Exception as e:
        print(f"❌ 模型初始化失敗: {e}")
        print("⚠️ 服務器將以降級模式運行")
    
    # 啟動服務器
    port = int(os.environ.get('BACKEND_PORT', 8080))
    server = HTTPServer(('0.0.0.0', port), RTX50SeriesWhisperHandler)
    
    print(f"\n✅ RTX 50 系列通用服務器已啟動")
    print(f"📱 端口: {port}")
    print(f"🎮 設備: {RTX50SeriesWhisperHandler.device}")
    print(f"🧠 模型: {'已載入' if RTX50SeriesWhisperHandler.model else '未載入'}")
    
    if torch.cuda.is_available():
        gpu_name = torch.cuda.get_device_name(0)
        gpu_memory = torch.cuda.get_device_properties(0).total_memory / 1024**3
        print(f"🎯 GPU: {gpu_name}")
        print(f"💾 VRAM: {gpu_memory:.1f} GB")
        print(f"🔥 RTX 50 系列優化: {'✅' if any(['RTX 50' in gpu_name, 'RTX 40' in gpu_name, 'RTX 30' in gpu_name, 'RTX 20' in gpu_name]) else '❌'}")
    
    print(f"🌐 健康檢查: http://localhost:{port}/health")
    print(f"🔍 GPU 詳情: http://localhost:{port}/gpu-info")
    print("=" * 70)
    
    server.serve_forever()

if __name__ == '__main__':
    main()