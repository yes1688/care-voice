#!/usr/bin/env python3
"""
RTX 50 系列通用 GPU 診斷工具
支援 RTX 50/40/30/20 系列，向下兼容到 GTX 10 系列
檢測和驗證 Ubuntu 24.04 + CUDA 12.8 + PyTorch nightly cu128 環境
"""

import os
import sys
import json
import time
import traceback
import torch
import numpy as np

def print_header(title):
    """打印標題"""
    print("\n" + "=" * 70)
    print(f" {title}")
    print("=" * 70)

def print_section(title):
    """打印章節標題"""
    print(f"\n🔍 {title}")
    print("-" * 50)

def check_basic_environment():
    """檢查基本環境"""
    print_section("基本環境檢查")
    
    print(f"Python 版本: {sys.version}")
    print(f"PyTorch 版本: {torch.__version__}")
    print(f"CUDA 版本 (PyTorch): {torch.version.cuda}")
    print(f"cuDNN 版本: {torch.backends.cudnn.version()}")
    
    # 環境變量
    print(f"CUDA_HOME: {os.environ.get('CUDA_HOME', 'Not set')}")
    print(f"CUDA_VISIBLE_DEVICES: {os.environ.get('CUDA_VISIBLE_DEVICES', 'Not set')}")
    print(f"TORCH_CUDA_ARCH_LIST: {os.environ.get('TORCH_CUDA_ARCH_LIST', 'Not set')}")

def check_cuda_availability():
    """檢查 CUDA 可用性"""
    print_section("CUDA 可用性檢查")
    
    try:
        cuda_available = torch.cuda.is_available()
        print(f"torch.cuda.is_available(): {cuda_available}")
        
        if cuda_available:
            device_count = torch.cuda.device_count()
            print(f"CUDA 設備數量: {device_count}")
            
            current_device = torch.cuda.current_device()
            print(f"當前設備: {current_device}")
            
            return True
        else:
            print("❌ CUDA 不可用")
            return False
            
    except Exception as e:
        print(f"❌ CUDA 檢查失敗: {e}")
        return False

def check_gpu_details():
    """檢查 GPU 詳細信息"""
    print_section("GPU 詳細信息")
    
    if not torch.cuda.is_available():
        print("❌ 無 GPU 可檢查")
        return False
    
    rtx_series_found = False
    best_gpu_arch = 0
    
    for i in range(torch.cuda.device_count()):
        print(f"\n📱 GPU {i}:")
        
        try:
            gpu_name = torch.cuda.get_device_name(i)
            props = torch.cuda.get_device_properties(i)
            
            print(f"   名稱: {gpu_name}")
            print(f"   計算能力: {props.major}.{props.minor} (sm_{props.major}{props.minor})")
            print(f"   總記憶體: {props.total_memory / 1024**3:.2f} GB")
            print(f"   多處理器數量: {props.multi_processor_count}")
            print(f"   最大執行緒/塊: {props.max_threads_per_block}")
            print(f"   最大共享記憶體/塊: {props.max_shared_memory_per_block / 1024:.1f} KB")
            print(f"   Warp 大小: {props.warp_size}")
            
            # 檢查 RTX 系列支援 (多架構兼容)
            is_supported = False
            gpu_series = "未知"
            
            if props.major >= 12:  # RTX 50 系列 (sm_120+)
                is_supported = True
                gpu_series = "RTX 50 系列"
                print(f"   ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功! [最新架構]")
            elif props.major == 8 and props.minor == 9:  # RTX 40 系列 (sm_89)
                is_supported = True
                gpu_series = "RTX 40 系列"
                print(f"   ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
            elif props.major == 8 and props.minor == 6:  # RTX 30 系列 (sm_86)
                is_supported = True
                gpu_series = "RTX 30 系列"
                print(f"   ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
            elif props.major == 7 and props.minor == 5:  # RTX 20 系列/GTX 16 系列 (sm_75)
                is_supported = True
                gpu_series = "RTX 20/GTX 16 系列"
                print(f"   ✅ {gpu_series} (sm_{props.major}{props.minor}) 檢測成功!")
            elif props.major >= 6:  # GTX 10 系列及以上 (sm_60+)
                is_supported = True
                gpu_series = "GTX 10 系列+"
                print(f"   ⚠️ {gpu_series} (sm_{props.major}{props.minor}) 基本支援")
            else:
                print(f"   ❌ 不支援的架構 (sm_{props.major}{props.minor})")
            
            if is_supported:
                rtx_series_found = True
                best_gpu_arch = max(best_gpu_arch, props.major * 10 + props.minor)
                print(f"   📊 GPU 系列: {gpu_series}")
            
        except Exception as e:
            print(f"   ❌ GPU {i} 信息獲取失敗: {e}")
    
    if rtx_series_found:
        print(f"\n✅ 檢測到支援的 GPU，最高架構: sm_{best_gpu_arch // 10}{best_gpu_arch % 10}")
    else:
        print("\n⚠️ 未檢測到支援的 RTX/GTX 系列 GPU")
    
    return rtx_series_found

def test_basic_cuda_operations():
    """測試基本 CUDA 操作"""
    print_section("基本 CUDA 操作測試")
    
    if not torch.cuda.is_available():
        print("❌ CUDA 不可用，跳過測試")
        return False
    
    try:
        # 基本張量操作
        print("🧪 測試基本張量操作...")
        x = torch.tensor([1.0, 2.0, 3.0, 4.0, 5.0], device='cuda')
        y = x * 2
        z = torch.sum(y)
        result = z.cpu().item()
        print(f"   ✅ 基本運算成功: sum([1,2,3,4,5] * 2) = {result}")
        
        # 矩陣乘法
        print("🧪 測試矩陣乘法...")
        a = torch.randn(128, 128, device='cuda')
        b = torch.randn(128, 128, device='cuda')
        c = torch.matmul(a, b)
        print(f"   ✅ 矩陣乘法成功: {c.shape}")
        
        # 大型張量操作
        print("🧪 測試大型張量操作...")
        large_tensor = torch.randn(1024, 1024, device='cuda')
        result = torch.sum(large_tensor)
        print(f"   ✅ 大型張量操作成功: sum = {result.cpu().item():.2f}")
        
        return True
        
    except Exception as e:
        print(f"   ❌ CUDA 操作失敗: {e}")
        traceback.print_exc()
        return False

def test_mixed_precision():
    """測試混合精度"""
    print_section("混合精度測試")
    
    if not torch.cuda.is_available():
        print("❌ CUDA 不可用，跳過測試")
        return False
    
    try:
        # 自動混合精度
        print("🧪 測試自動混合精度 (AMP)...")
        
        with torch.cuda.amp.autocast():
            x = torch.randn(256, 256, device='cuda')
            y = torch.randn(256, 256, device='cuda')
            z = torch.matmul(x, y)
            result = torch.sum(z)
        
        print(f"   ✅ AMP 測試成功: {result.cpu().item():.2f}")
        
        # FP16 測試
        print("🧪 測試 FP16 操作...")
        x_fp16 = torch.randn(512, 512, dtype=torch.float16, device='cuda')
        y_fp16 = torch.randn(512, 512, dtype=torch.float16, device='cuda')
        z_fp16 = torch.matmul(x_fp16, y_fp16)
        print(f"   ✅ FP16 測試成功: {z_fp16.shape}")
        
        return True
        
    except Exception as e:
        print(f"   ❌ 混合精度測試失敗: {e}")
        traceback.print_exc()
        return False

def test_whisper_compatibility():
    """測試 Whisper 兼容性"""
    print_section("Whisper 兼容性測試")
    
    try:
        import whisper
        print("✅ Whisper 模組導入成功")
        
        # 測試模型列表
        available_models = whisper.available_models()
        print(f"可用模型: {available_models}")
        
        if torch.cuda.is_available():
            print("🧪 測試 GPU Whisper 模型載入...")
            
            try:
                # 嘗試載入最小的模型
                model = whisper.load_model("tiny", device="cuda")
                print("✅ GPU Whisper 模型載入成功!")
                
                # 測試簡單推理
                print("🧪 測試 GPU 推理...")
                dummy_audio = np.random.randn(16000).astype(np.float32)
                result = model.transcribe(dummy_audio, fp16=True, verbose=False)
                print(f"✅ GPU 推理成功: '{result['text'][:50]}...'")
                
                return True
                
            except Exception as gpu_error:
                print(f"❌ GPU Whisper 測試失敗: {gpu_error}")
                print("完整錯誤信息:")
                traceback.print_exc()
                
                # 嘗試 CPU 模式
                print("🧪 回退到 CPU 模式測試...")
                model = whisper.load_model("tiny", device="cpu")
                print("✅ CPU Whisper 模型載入成功")
                return False
        else:
            print("⚠️ CUDA 不可用，僅測試 CPU 模式")
            model = whisper.load_model("tiny", device="cpu")
            print("✅ CPU Whisper 模型載入成功")
            return False
            
    except ImportError:
        print("❌ 無法導入 Whisper")
        return False
    except Exception as e:
        print(f"❌ Whisper 測試失敗: {e}")
        traceback.print_exc()
        return False

def benchmark_gpu_performance():
    """GPU 性能基準測試"""
    print_section("GPU 性能基準測試")
    
    if not torch.cuda.is_available():
        print("❌ CUDA 不可用，跳過性能測試")
        return
    
    try:
        # 矩陣乘法性能測試
        sizes = [512, 1024, 2048]
        
        for size in sizes:
            print(f"🏃 測試 {size}x{size} 矩陣乘法性能...")
            
            # 預熱
            a = torch.randn(size, size, device='cuda')
            b = torch.randn(size, size, device='cuda')
            _ = torch.matmul(a, b)
            torch.cuda.synchronize()
            
            # 性能測試
            num_iterations = 10
            start_time = time.time()
            
            for _ in range(num_iterations):
                c = torch.matmul(a, b)
                torch.cuda.synchronize()
            
            end_time = time.time()
            avg_time = (end_time - start_time) / num_iterations
            gflops = (2 * size**3) / (avg_time * 1e9)
            
            print(f"   平均時間: {avg_time*1000:.2f} ms")
            print(f"   性能: {gflops:.2f} GFLOPS")
            
            # 記憶體使用
            memory_used = torch.cuda.memory_allocated() / 1024**3
            print(f"   GPU 記憶體使用: {memory_used:.2f} GB")
        
    except Exception as e:
        print(f"❌ 性能測試失敗: {e}")
        traceback.print_exc()

def generate_report():
    """生成診斷報告"""
    print_header("RTX 50 系列 GPU 診斷報告")
    
    results = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "basic_environment": True,
        "cuda_available": False,
        "rtx_series_detected": False,
        "basic_operations": False,
        "mixed_precision": False,
        "whisper_gpu_compatible": False,
        "overall_status": "未知"
    }
    
    try:
        # 基本環境檢查
        check_basic_environment()
        
        # CUDA 可用性
        results["cuda_available"] = check_cuda_availability()
        
        if results["cuda_available"]:
            # GPU 詳細信息
            results["rtx_series_detected"] = check_gpu_details()
            
            # 基本操作測試
            results["basic_operations"] = test_basic_cuda_operations()
            
            # 混合精度測試
            results["mixed_precision"] = test_mixed_precision()
            
            # 性能基準測試
            benchmark_gpu_performance()
        
        # Whisper 兼容性測試
        results["whisper_gpu_compatible"] = test_whisper_compatibility()
        
        # 總體狀態評估
        if results["rtx_series_detected"] and results["basic_operations"] and results["whisper_gpu_compatible"]:
            results["overall_status"] = "✅ 完全兼容"
        elif results["cuda_available"] and results["basic_operations"]:
            results["overall_status"] = "🟡 部分兼容"
        else:
            results["overall_status"] = "❌ 不兼容"
        
        # 打印總結
        print_header("診斷總結")
        print(f"時間戳: {results['timestamp']}")
        print(f"CUDA 可用: {'✅' if results['cuda_available'] else '❌'}")
        print(f"RTX 系列檢測: {'✅' if results['rtx_series_detected'] else '❌'}")
        print(f"基本 CUDA 操作: {'✅' if results['basic_operations'] else '❌'}")
        print(f"混合精度支持: {'✅' if results['mixed_precision'] else '❌'}")
        print(f"Whisper GPU 兼容: {'✅' if results['whisper_gpu_compatible'] else '❌'}")
        print(f"總體狀態: {results['overall_status']}")
        
        # 建議
        print_section("建議")
        if results["overall_status"] == "✅ 完全兼容":
            print("🎉 恭喜！您的 RTX 系列 GPU 已完全支持 GPU 加速")
            print("💡 建議使用混合精度 (fp16) 以獲得最佳性能")
        elif results["overall_status"] == "🟡 部分兼容":
            print("⚠️ GPU 可用但 Whisper 兼容性有問題")
            print("💡 建議檢查 PyTorch 版本是否為 nightly cu128")
        else:
            print("❌ GPU 加速不可用")
            print("💡 建議使用 CPU 回退模式")
        
        return results
        
    except Exception as e:
        print(f"❌ 診斷過程中出現錯誤: {e}")
        traceback.print_exc()
        results["overall_status"] = "❌ 診斷失敗"
        return results

def main():
    """主函數"""
    results = generate_report()
    
    # 保存結果到文件
    try:
        with open('/app/logs/gpu_diagnostics_report.json', 'w') as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        print(f"\n📄 診斷報告已保存到: /app/logs/gpu_diagnostics_report.json")
    except Exception as e:
        print(f"⚠️ 無法保存報告: {e}")
    
    return results

if __name__ == "__main__":
    main()