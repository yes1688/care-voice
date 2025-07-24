#!/usr/bin/env python3
import os
import sys
import json
import tempfile
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import cgi
import torch
# import whisper  # 暫時註釋，先測試 GPU 訪問

class WhisperHandler(BaseHTTPRequestHandler):
    model = None  # 類變數
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            # 檢查 GPU 可用性
            gpu_available = torch.cuda.is_available()
            device = "cuda" if gpu_available else "cpu"
            
            response = {
                "status": "healthy",
                "service": "Care Voice GPU Whisper",
                "version": "1.0.0",
                "gpu_available": gpu_available,
                "device": device,
                "cuda_device_count": torch.cuda.device_count() if gpu_available else 0
            }
            
            self.wfile.write(json.dumps(response, indent=2).encode())
        else:
            self.send_error(404)
    
    def do_POST(self):
        if self.path == '/upload':
            try:
                # 檢測 GPU 並模擬 Whisper 處理
                device = "cuda" if torch.cuda.is_available() else "cpu"
                print(f"Processing on {device}")
                # 模擬 GPU 計算
                if torch.cuda.is_available():
                    x = torch.randn(1000, 1000).cuda()
                    result = torch.mm(x, x.t())
                    print(f"GPU computation result shape: {result.shape}")
                
                # 簡化的轉錄響應 (演示用)
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                
                response = {
                    "full_transcript": "GPU 加速 Whisper 轉錄測試成功！",
                    "summary": "GPU 轉錄摘要: 系統正常運行，GPU 加速功能已啟用。"
                }
                
                self.wfile.write(json.dumps(response, ensure_ascii=False).encode('utf-8'))
                
            except Exception as e:
                print(f"Error: {e}")
                self.send_error(500, str(e))
        else:
            self.send_error(404)
    
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()

if __name__ == '__main__':
    port = int(os.environ.get('BACKEND_PORT', 8080))
    server = HTTPServer(('0.0.0.0', port), WhisperHandler)
    print(f"GPU Whisper server starting on port {port}")
    print(f"CUDA available: {torch.cuda.is_available()}")
    if torch.cuda.is_available():
        print(f"CUDA devices: {torch.cuda.device_count()}")
        print(f"Current device: {torch.cuda.current_device()}")
    server.serve_forever()