#!/bin/bash
# 快速部署 Opus 支援修復腳本

set -e

echo "🔧 開始部署 Opus 支援修復..."

# 檢查容器狀態
if ! podman ps | grep -q "care-voice-opus-test"; then
    echo "❌ Opus 測試容器未運行"
    exit 1
fi

echo "📦 複製更新的源代碼到容器..."
podman cp /mnt/datadrive/MyProjects/care-voice/backend/src/ care-voice-opus-test:/tmp/src/
podman cp /mnt/datadrive/MyProjects/care-voice/backend/Cargo.toml care-voice-opus-test:/tmp/

echo "🔍 檢查容器內Rust環境..."
if podman exec care-voice-opus-test which cargo > /dev/null 2>&1; then
    echo "✅ Rust 已安裝"
    CARGO_CMD="cargo"
else
    echo "⚠️ 需要安裝 Rust"
    # 安裝 Rust
    podman exec care-voice-opus-test bash -c "
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    "
    CARGO_CMD="~/.cargo/bin/cargo"
fi

echo "🔧 在容器內編譯 Opus 支援版本..."
podman exec -w /tmp care-voice-opus-test bash -c "
    export PATH=\"~/.cargo/bin:\$PATH\"
    echo '🧹 清理舊的編譯...'
    rm -rf target/
    
    echo '📦 開始編譯...'
    ~/.cargo/bin/cargo build --release --features opus-support 2>&1 | tee /tmp/build.log
    
    echo '📋 檢查編譯結果...'
    if [ -f target/release/care-voice ]; then
        echo '✅ 編譯成功'
        ls -la target/release/care-voice
    else
        echo '❌ 編譯失敗'
        tail -20 /tmp/build.log
        exit 1
    fi
"

if [ $? -eq 0 ]; then
    echo "🔄 替換運行中的二進制檔案..."
    
    # 備份原始檔案
    podman exec care-voice-opus-test cp /app/care-voice /app/care-voice.backup
    
    # 複製新編譯的檔案
    podman exec care-voice-opus-test cp /tmp/target/release/care-voice /app/care-voice
    
    # 設定執行權限
    podman exec care-voice-opus-test chmod +x /app/care-voice
    
    echo "🔄 重啟服務..."
    # 重啟容器以載入新的二進制檔案
    podman restart care-voice-opus-test
    
    echo "⏳ 等待服務重啟..."
    sleep 10
    
    echo "🔍 測試新版本..."
    for i in {1..30}; do
        if curl -s http://localhost:8002/health > /dev/null; then
            echo "✅ 服務重啟成功"
            break
        fi
        echo "⏳ 等待服務啟動... ($i/30)"
        sleep 2
    done
    
    # 顯示新版本資訊
    echo "📊 新版本健康檢查:"
    curl -s http://localhost:8002/health | jq . || curl -s http://localhost:8002/health
    
    echo "🎉 Opus 支援部署完成！"
    echo "🔗 測試端點: http://localhost:8002/api/upload"
    
else
    echo "❌ 編譯失敗，請檢查日誌"
    podman exec care-voice-opus-test cat /tmp/build.log | tail -20
    exit 1
fi

echo "✅ 部署修復完成"