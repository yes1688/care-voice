#!/usr/bin/env python3
"""
Care Voice whisper-rs GPU 診斷工具
監控 whisper-rs Rust 後端的 GPU 使用狀況
"""

import time
import sys
import json
import subprocess
import argparse
from datetime import datetime

class WhisperRsGPUDiagnostics:
    def __init__(self):
        self.service_name = "Care Voice whisper-rs GPU"
        self.start_time = datetime.now()
        
    def check_nvidia_smi(self):
        """檢查 nvidia-smi 可用性"""
        try:
            result = subprocess.run(['nvidia-smi', '--query-gpu=name,memory.total,memory.used,utilization.gpu', '--format=csv,noheader,nounits'], 
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                return True, result.stdout.strip()
            else:
                return False, f"nvidia-smi 錯誤: {result.stderr}"
        except FileNotFoundError:
            return False, "nvidia-smi 未安裝"
        except subprocess.TimeoutExpired:
            return False, "nvidia-smi 超時"
        except Exception as e:
            return False, f"nvidia-smi 異常: {e}"

    def get_gpu_info(self):
        """獲取 GPU 詳細資訊"""
        available, info = self.check_nvidia_smi()
        
        if not available:
            return {
                "available": False,
                "error": info,
                "device_count": 0,
                "devices": []
            }
        
        devices = []
        for line in info.split('\n'):
            if line.strip():
                parts = line.split(', ')
                if len(parts) >= 4:
                    devices.append({
                        "name": parts[0],
                        "memory_total_mb": int(parts[1]),
                        "memory_used_mb": int(parts[2]),
                        "utilization_percent": int(parts[3])
                    })
        
        return {
            "available": True,
            "device_count": len(devices),
            "devices": devices
        }

    def check_whisper_rs_process(self):
        """檢查 whisper-rs 進程狀態"""
        try:
            result = subprocess.run(['pgrep', '-f', 'care-voice'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                pid = result.stdout.strip()
                return True, f"PID: {pid}"
            else:
                return False, "進程未找到"
        except Exception as e:
            return False, f"檢查失敗: {e}"

    def check_health_endpoint(self):
        """檢查 whisper-rs 健康端點"""
        try:
            import urllib.request
            import urllib.error
            
            with urllib.request.urlopen('http://localhost:8080/health', timeout=5) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode())
                    return True, data
                else:
                    return False, f"HTTP {response.status}"
        except urllib.error.URLError as e:
            return False, f"連接失敗: {e}"
        except Exception as e:
            return False, f"請求異常: {e}"

    def get_system_info(self):
        """獲取系統資訊"""
        try:
            import psutil
            return {
                "cpu_percent": psutil.cpu_percent(interval=1),
                "memory_percent": psutil.virtual_memory().percent,
                "disk_percent": psutil.disk_usage('/').percent,
                "uptime_seconds": (datetime.now() - self.start_time).total_seconds()
            }
        except ImportError:
            return {"error": "psutil 未安裝"}
        except Exception as e:
            return {"error": f"系統資訊獲取失敗: {e}"}

    def generate_report(self):
        """生成完整診斷報告"""
        report = {
            "timestamp": datetime.now().isoformat(),
            "service": self.service_name,
            "version": "1.0.0",
            "diagnostics": {
                "gpu": self.get_gpu_info(),
                "whisper_rs_process": {},
                "whisper_rs_health": {},
                "system": self.get_system_info()
            }
        }
        
        # 檢查 whisper-rs 進程
        process_ok, process_info = self.check_whisper_rs_process()
        report["diagnostics"]["whisper_rs_process"] = {
            "running": process_ok,
            "info": process_info
        }
        
        # 檢查健康端點
        health_ok, health_info = self.check_health_endpoint()
        report["diagnostics"]["whisper_rs_health"] = {
            "accessible": health_ok,
            "response": health_info if health_ok else {"error": health_info}
        }
        
        return report

    def print_report(self):
        """打印格式化的診斷報告"""
        report = self.generate_report()
        
        print(f"\n🔍 {self.service_name} 診斷報告")
        print("=" * 60)
        print(f"⏰ 時間: {report['timestamp']}")
        
        # GPU 狀態
        gpu = report["diagnostics"]["gpu"]
        print(f"\n🎮 GPU 狀態:")
        if gpu["available"]:
            print(f"  ✅ 可用 - {gpu['device_count']} 個設備")
            for i, device in enumerate(gpu["devices"]):
                print(f"  📊 GPU {i}: {device['name']}")
                print(f"     記憶體: {device['memory_used_mb']}/{device['memory_total_mb']} MB")
                print(f"     使用率: {device['utilization_percent']}%")
        else:
            print(f"  ❌ 不可用 - {gpu['error']}")
        
        # whisper-rs 進程
        process = report["diagnostics"]["whisper_rs_process"]
        print(f"\n🦀 whisper-rs 進程:")
        if process["running"]:
            print(f"  ✅ 運行中 - {process['info']}")
        else:
            print(f"  ❌ 未運行 - {process['info']}")
        
        # 健康檢查
        health = report["diagnostics"]["whisper_rs_health"]
        print(f"\n🏥 健康檢查:")
        if health["accessible"]:
            print(f"  ✅ 正常 - 服務可訪問")
            if isinstance(health["response"], dict):
                if "status" in health["response"]:
                    print(f"     狀態: {health['response']['status']}")
        else:
            print(f"  ❌ 異常 - {health['response']['error']}")
        
        # 系統資源
        system = report["diagnostics"]["system"]
        if "error" not in system:
            print(f"\n💻 系統資源:")
            print(f"  CPU: {system.get('cpu_percent', 'N/A')}%")
            print(f"  記憶體: {system.get('memory_percent', 'N/A')}%")
            print(f"  磁碟: {system.get('disk_percent', 'N/A')}%")
        
        print("\n" + "=" * 60)

    def monitor_loop(self, interval=30):
        """持續監控模式"""
        print(f"🔄 開始監控 {self.service_name} (間隔: {interval}秒)")
        print("按 Ctrl+C 停止監控")
        
        try:
            while True:
                report = self.generate_report()
                
                # 簡化的監控輸出
                gpu_status = "✅" if report["diagnostics"]["gpu"]["available"] else "❌"
                process_status = "✅" if report["diagnostics"]["whisper_rs_process"]["running"] else "❌"
                health_status = "✅" if report["diagnostics"]["whisper_rs_health"]["accessible"] else "❌"
                
                timestamp = datetime.now().strftime("%H:%M:%S")
                print(f"[{timestamp}] GPU:{gpu_status} 進程:{process_status} 健康:{health_status}")
                
                time.sleep(interval)
                
        except KeyboardInterrupt:
            print("\n🛑 監控已停止")

def main():
    parser = argparse.ArgumentParser(description="whisper-rs GPU 診斷工具")
    parser.add_argument('--monitor', action='store_true', help='持續監控模式')
    parser.add_argument('--interval', type=int, default=30, help='監控間隔(秒)')
    parser.add_argument('--json', action='store_true', help='JSON 格式輸出')
    
    args = parser.parse_args()
    
    diagnostics = WhisperRsGPUDiagnostics()
    
    if args.monitor:
        diagnostics.monitor_loop(args.interval)
    else:
        if args.json:
            report = diagnostics.generate_report()
            print(json.dumps(report, indent=2, ensure_ascii=False))
        else:
            diagnostics.print_report()

if __name__ == "__main__":
    main()