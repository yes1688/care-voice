#!/usr/bin/env python3
"""
測試 PyTorch nightly 版本對 RTX 5070 Ti (sm_120) 的支持
"""

import subprocess
import sys

def install_pytorch_nightly():
    """安裝 PyTorch nightly 版本"""
    print("🔄 安裝 PyTorch nightly 版本...")
    
    # 先卸載現有版本
    subprocess.run([sys.executable, "-m", "pip", "uninstall", "-y", "torch", "torchvision", "torchaudio"])
    
    # 安裝 nightly 版本
    cmd = [
        sys.executable, "-m", "pip", "install", "--pre", 
        "torch", "torchvision", "torchaudio", 
        "--index-url", "https://download.pytorch.org/whl/nightly/cu124"
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    print(f"安裝結果: {result.returncode}")
    if result.returncode != 0:
        print(f"錯誤: {result.stderr}")
        return False
    
    return True

def test_gpu_support():
    """測試 GPU 支持"""
    print("🧪 測試 GPU 支持...")
    
    try:
        import torch
        print(f"PyTorch 版本: {torch.__version__}")
        print(f"CUDA 可用: {torch.cuda.is_available()}")
        
        if torch.cuda.is_available():
            print(f"GPU 名稱: {torch.cuda.get_device_name(0)}")
            props = torch.cuda.get_device_properties(0)
            print(f"計算能力: {props.major}.{props.minor}")
            
            # 測試基本運算
            try:
                x = torch.tensor([1.0, 2.0, 3.0]).cuda()
                y = x * 2
                print(f"✅ GPU 運算成功: {y.cpu().numpy()}")
                return True
            except Exception as e:
                print(f"❌ GPU 運算失敗: {e}")
                return False
        else:
            print("❌ CUDA 不可用")
            return False
            
    except ImportError as e:
        print(f"❌ 無法導入 PyTorch: {e}")
        return False

def test_whisper_gpu():
    """測試 Whisper GPU 支持"""
    print("🎤 測試 Whisper GPU 支持...")
    
    try:
        import whisper
        import torch
        import numpy as np
        
        # 嘗試載入 GPU 模型
        model = whisper.load_model("tiny", device="cuda")
        print("✅ Whisper GPU 模型載入成功")
        
        # 測試轉錄
        audio = np.random.randn(16000).astype(np.float32)  # 1秒隨機音頻
        result = model.transcribe(audio, fp16=True)
        print(f"✅ GPU 轉錄測試成功: {result['text'][:50]}...")
        
        return True
        
    except Exception as e:
        print(f"❌ Whisper GPU 測試失敗: {e}")
        return False

if __name__ == "__main__":
    print("🚀 PyTorch nightly 測試開始")
    print("=" * 50)
    
    # 安裝 nightly 版本
    if install_pytorch_nightly():
        print("✅ PyTorch nightly 安裝成功")
        
        # 測試 GPU 支持
        if test_gpu_support():
            print("✅ GPU 支持測試成功")
            
            # 測試 Whisper
            if test_whisper_gpu():
                print("✅ 所有測試通過！RTX 5070 Ti GPU 加速可用")
            else:
                print("⚠️ Whisper GPU 測試失敗，但基礎 GPU 運算可用")
        else:
            print("❌ GPU 支持測試失敗")
    else:
        print("❌ PyTorch nightly 安裝失敗")