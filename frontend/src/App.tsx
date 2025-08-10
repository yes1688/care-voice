import { createSignal, Show, onMount } from 'solid-js';

// 介面定義
interface TranscriptResult {
  full_transcript: string;
  summary: string;
}

interface ErrorResponse {
  error: string;
}

interface BrowserInfo {
  name: string;
  mimeType: string;
  ext: string;
  isSupported: boolean;
  webCodecsSupported?: boolean;
  recordingMethod?: 'webcodecs' | 'mediarecorder';
}

interface WebCodecsInfo {
  audioEncoder: boolean;
  audioDecoder: boolean;
  opusSupported: boolean;
  fullSupported: boolean;
}

interface HealthCheckResult {
  status: string;
  message: string;
  timestamp: string;
}

function App() {
  const [isRecording, setIsRecording] = createSignal(false);
  const [audioBlob, setAudioBlob] = createSignal<Blob | null>(null);
  const [isUploading, setIsUploading] = createSignal(false);
  const [result, setResult] = createSignal<TranscriptResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [recordingTime, setRecordingTime] = createSignal(0);
  const [browserInfo, setBrowserInfo] = createSignal<BrowserInfo | null>(null);
  const [healthStatus, setHealthStatus] = createSignal<HealthCheckResult | null>(null);
  const [isHealthy, setIsHealthy] = createSignal(false);
  const [webCodecsInfo, setWebCodecsInfo] = createSignal<WebCodecsInfo | null>(null);
  
  let mediaRecorder: MediaRecorder | null = null;
  let audioEncoder: AudioEncoder | null = null;
  let recordingInterval: number | null = null;
  let audioPackets: Uint8Array[] = []; // 🎯 修復：改用獨立包收集

  // 🚀 WebCodecs 支援檢測 - 2025年業界領先技術
  const detectWebCodecsSupport = (): WebCodecsInfo => {
    const hasAudioEncoder = typeof AudioEncoder !== 'undefined';
    const hasAudioDecoder = typeof AudioDecoder !== 'undefined';
    
    let opusSupported = false;
    if (hasAudioEncoder) {
      try {
        // 檢測 OPUS 編碼支援 - 使用實際配置參數
        const testConfig = {
          codec: 'opus',
          sampleRate: 48000,      // 🎯 修復: 使用實際48kHz配置
          numberOfChannels: 1,
          bitrate: 128000         // 🎯 修復: 使用實際128kbps配置
        };
        opusSupported = AudioEncoder.isConfigSupported && 
                       AudioEncoder.isConfigSupported(testConfig);
      } catch (e) {
        console.warn('WebCodecs OPUS 支援檢測失敗:', e);
        opusSupported = false;
      }
    }
    
    const fullSupported = hasAudioEncoder && hasAudioDecoder && opusSupported;
    
    const result = {
      audioEncoder: hasAudioEncoder,
      audioDecoder: hasAudioDecoder,
      opusSupported: opusSupported,
      fullSupported: fullSupported
    };
    
    console.log('🚀 WebCodecs 支援檢測結果:', result);
    return result;
  };

  // 檢測瀏覽器和支援的格式 - 業界領先實現 + WebCodecs 整合
  const detectBrowser = (): BrowserInfo => {
    const ua = navigator.userAgent;
    const webCodecs = detectWebCodecsSupport();
    
    if (ua.includes('Chrome') && !ua.includes('Edge')) {
      const mimeType = 'audio/webm;codecs=opus';
      return {
        name: 'Chrome',
        mimeType,
        ext: 'webm',
        isSupported: MediaRecorder.isTypeSupported(mimeType),
        webCodecsSupported: webCodecs.fullSupported,
        recordingMethod: webCodecs.fullSupported ? 'webcodecs' : 'mediarecorder'
      };
    } else if (ua.includes('Edge')) {
      const mimeType = 'audio/webm;codecs=opus';
      return {
        name: 'Edge',
        mimeType,
        ext: 'webm',
        isSupported: MediaRecorder.isTypeSupported(mimeType),
        webCodecsSupported: webCodecs.fullSupported,
        recordingMethod: webCodecs.fullSupported ? 'webcodecs' : 'mediarecorder'
      };
    } else if (ua.includes('Firefox')) {
      const mimeType = 'audio/ogg;codecs=opus';
      return {
        name: 'Firefox',
        mimeType,
        ext: 'ogg',
        isSupported: MediaRecorder.isTypeSupported(mimeType),
        webCodecsSupported: webCodecs.fullSupported,
        recordingMethod: webCodecs.fullSupported ? 'webcodecs' : 'mediarecorder'
      };
    } else if (ua.includes('Safari')) {
      const mimeType = 'audio/mp4';
      return {
        name: 'Safari',
        mimeType,
        ext: 'mp4',
        isSupported: MediaRecorder.isTypeSupported(mimeType),
        webCodecsSupported: webCodecs.fullSupported,
        recordingMethod: webCodecs.fullSupported ? 'webcodecs' : 'mediarecorder'
      };
    }
    
    // 未知瀏覽器，嘗試通用格式
    const fallbackMime = 'audio/webm';
    return {
      name: 'Unknown',
      mimeType: fallbackMime,
      ext: 'webm',
      isSupported: MediaRecorder.isTypeSupported(fallbackMime),
      webCodecsSupported: webCodecs.fullSupported,
      recordingMethod: webCodecs.fullSupported ? 'webcodecs' : 'mediarecorder'
    };
  };

  // 健康檢查功能
  const performHealthCheck = async () => {
    try {
      const response = await fetch('/health');
      const result = await response.text();
      
      const healthResult: HealthCheckResult = {
        status: response.ok ? 'healthy' : 'unhealthy',
        message: result,
        timestamp: new Date().toISOString()
      };
      
      setHealthStatus(healthResult);
      setIsHealthy(response.ok);
    } catch (error) {
      const healthResult: HealthCheckResult = {
        status: 'error',
        message: `連接失敗: ${error instanceof Error ? error.message : 'Unknown error'}`,
        timestamp: new Date().toISOString()
      };
      
      setHealthStatus(healthResult);
      setIsHealthy(false);
    }
  };

  // 頁面載入時初始化
  onMount(() => {
    const webCodecs = detectWebCodecsSupport();
    setWebCodecsInfo(webCodecs);
    
    const browser = detectBrowser();
    setBrowserInfo(browser);
    
    console.log('🌐 檢測到瀏覽器:', browser);
    console.log('🚀 WebCodecs 功能:', webCodecs);
    
    // 自動執行健康檢查
    performHealthCheck();
  });

  // 🚀 WebCodecs 錄音實現 - 2025年業界領先技術（修復版）
  const startWebCodecsRecording = async (stream: MediaStream) => {
    console.log('🚀 啟動 WebCodecs 硬體加速錄音（修復版）');
    
    // 🎯 修復：重置獨立包數組
    audioPackets = [];
    
    try {
      audioEncoder = new AudioEncoder({
        output: (chunk, metadata) => {
          console.log(`🎵 WebCodecs 獨立包輸出: ${chunk.byteLength} bytes`);
          // 🎯 關鍵修復：每個 chunk 已經是完整的 OPUS 包
          const packetData = new Uint8Array(chunk.byteLength);
          chunk.copyTo(packetData);
          audioPackets.push(packetData); // 直接添加完整包，不合併
          console.log(`📦 收集到 OPUS 包 ${audioPackets.length}: ${packetData.length} bytes`);
        },
        error: (error) => {
          console.error('🚨 WebCodecs 編碼錯誤:', error);
          setError(`WebCodecs 編碼失敗: ${error.message}`);
          // 🎯 診斷模式：不降級，直接顯示錯誤以便分析
        }
      });

      // 🎯 優化配置：平衡品質與處理效率
      const optimizedEncoderConfig = {
        codec: 'opus',
        sampleRate: 48000,        // 固定48kHz（瀏覽器標準）
        numberOfChannels: 1,      // 單聲道（Whisper要求）
        bitrate: 96000,           // 🔧 優化：96kbps平衡品質與檔案大小
      };

      console.log('🔧 WebCodecs 優化編碼器配置:', optimizedEncoderConfig);
      
      // 立即配置編碼器
      try {
        audioEncoder.configure(optimizedEncoderConfig);
        console.log('✅ 編碼器初始化配置成功');
      } catch (configError) {
        console.error('🚨 編碼器初始配置失敗:', configError);
        setError(`WebCodecs 編碼器配置失敗: ${configError.message}`);
        return;
      }

      // 使用 MediaStreamTrackProcessor 處理音頻流
      const track = stream.getAudioTracks()[0];
      const processor = new MediaStreamTrackProcessor({ track });
      const reader = processor.readable.getReader();

      // 處理音頻幀 - 簡化版本
      const processAudioFrames = async () => {
        let frameCount = 0;
        
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          
          // 🔍 首幀診斷（簡化版）
          if (frameCount === 0) {
            console.log('🎵 AudioFrame 格式確認:');
            console.log(`  - 聲道數: ${value.numberOfChannels}`);
            console.log(`  - 採樣率: ${value.sampleRate}Hz`);
            console.log(`  - 持續時間: ${value.duration}μs`);
            console.log('📊 使用固定48kHz配置進行OPUS編碼');
          }
          frameCount++;
          
          // 直接編碼（編碼器已在初始化時配置）
          if (audioEncoder && audioEncoder.state === 'configured') {
            try {
              audioEncoder.encode(value);
            } catch (err) {
              console.error('🚨 音頻幀編碼失敗:', err);
            }
          }
          value.close(); // 釋放音頻幀資源
        }
        console.log(`📊 總共處理了 ${frameCount} 個 AudioFrame`);
      };

      // 開始處理音頻幀
      processAudioFrames().catch(err => {
        console.error('🚨 音頻處理流程錯誤:', err);
        setError(`WebCodecs 音頻處理失敗: ${err.message}`);
        // 🎯 診斷模式：不降級，保持錯誤狀態以便分析
      });

      console.log('✅ WebCodecs 錄音已啟動');
      
    } catch (error) {
      console.error('🚨 WebCodecs 初始化失敗:', error);
      setError(`WebCodecs 初始化失敗: ${error.message}`);
      // 🎯 診斷模式：不降級，直接報錯以便分析問題
    }
  };

  // 📼 MediaRecorder 錄音實現 - 相容模式
  const startMediaRecorderRecording = async (stream: MediaStream) => {
    console.log('📼 啟動 MediaRecorder 相容模式錄音');
    
    // 業界領先：使用瀏覽器最佳格式
    const browser = browserInfo();
    const options: MediaRecorderOptions = {};
    
    if (browser && browser.isSupported) {
      options.mimeType = browser.mimeType;
      console.log(`✅ 使用 ${browser.name} 最佳格式: ${browser.mimeType}`);
    } else {
      // Fallback 到通用格式
      const fallbackFormats = [
        'audio/webm;codecs=opus',
        'audio/ogg;codecs=opus', 
        'audio/webm',
        'audio/wav'
      ];
      
      for (const format of fallbackFormats) {
        if (MediaRecorder.isTypeSupported(format)) {
          options.mimeType = format;
          console.log(`⚠️ 使用 fallback 格式: ${format}`);
          break;
        }
      }
    }
    
    mediaRecorder = new MediaRecorder(stream, options);
    const chunks: Blob[] = [];
    
    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        chunks.push(event.data);
      }
    };
    
    mediaRecorder.onstop = () => {
      const finalMimeType = options.mimeType || 'audio/webm';
      const blob = new Blob(chunks, { type: finalMimeType });
      setAudioBlob(blob);
      
      console.log(`✅ MediaRecorder 錄音完成 - 格式: ${finalMimeType}, 大小: ${blob.size} bytes, 瀏覽器: ${browser?.name}`);
      
      // Stop all tracks to free up the microphone
      stream.getTracks().forEach(track => track.stop());
      
      if (recordingInterval) {
        clearInterval(recordingInterval);
        recordingInterval = null;
      }
    };
    
    mediaRecorder.start();
    console.log('✅ MediaRecorder 錄音已啟動');
  };

  const startRecording = async () => {
    try {
      setError(null);
      setResult(null);
      
      // 🔧 修復音頻配置一致性 - 統一使用48kHz避免瀏覽器重採樣
      const browser = browserInfo();
      const audioConstraints = {
        sampleRate: 48000,        // 🎯 修復: 統一使用48kHz (與WebCodecs編碼器一致)
        channelCount: 1,          // 單聲道
        echoCancellation: true,
        noiseSuppression: true
      };
      
      console.log(`🎤 請求音頻權限 - 配置:`, audioConstraints);
      const stream = await navigator.mediaDevices.getUserMedia({ 
        audio: audioConstraints
      });
      
      // 🔍 診斷: 檢查瀏覽器實際提供的音頻配置
      const audioTrack = stream.getAudioTracks()[0];
      if (audioTrack) {
        const trackSettings = audioTrack.getSettings();
        console.log(`🔍 瀏覽器實際音頻配置:`, trackSettings);
        console.log(`  - 實際採樣率: ${trackSettings.sampleRate}Hz`);
        console.log(`  - 實際聲道數: ${trackSettings.channelCount}`);
        console.log(`  - 配置匹配: ${trackSettings.sampleRate === 48000 ? '✅ 一致' : '⚠️ 不匹配'}`);
      }
      
      // 🚀 智能錄音方式選擇 - 2025年業界領先
      if (browser?.recordingMethod === 'webcodecs' && browser.webCodecsSupported) {
        console.log('🚀 使用 WebCodecs 硬體加速錄音 (2025年業界領先)');
        await startWebCodecsRecording(stream);
      } else {
        console.log('📼 使用 MediaRecorder 相容模式錄音');
        await startMediaRecorderRecording(stream);
      }
      
      // 設置共同的錄音狀態
      setIsRecording(true);
      setRecordingTime(0);
      
      // Start recording timer
      recordingInterval = setInterval(() => {
        setRecordingTime(prev => prev + 1);
      }, 1000);
      
    } catch (err) {
      console.error('Failed to start recording:', err);
      setError('無法訪問麥克風。請確保已授予麥克風權限。');
    }
  };

  const stopRecording = () => {
    const browser = browserInfo();
    
    if (browser?.recordingMethod === 'webcodecs' && audioEncoder) {
      console.log('🛑 停止 WebCodecs 錄音');
      try {
        // 完成編碼並清理
        audioEncoder.flush();
        audioEncoder.close();
        audioEncoder = null;
        
        // 🎯 修復：將獨立包轉換為 JSON 格式上傳
        if (audioPackets.length > 0) {
          // 創建包含獨立包的數據結構
          const packetsData = {
            format: 'webcodecs_opus_packets',
            packet_count: audioPackets.length,
            packets: audioPackets.map(packet => Array.from(packet)) // 轉換為數字陣列以便 JSON 序列化
          };
          
          // 創建 JSON Blob
          const jsonBlob = new Blob([JSON.stringify(packetsData)], { type: 'application/json' });
          setAudioBlob(jsonBlob);
          
          console.log(`✅ WebCodecs 錄音完成 - 格式: 獨立包模式, 包數量: ${audioPackets.length}, JSON 大小: ${jsonBlob.size} bytes`);
          
          // 統計包大小分佈
          const sizes = audioPackets.map(p => p.length);
          const minSize = Math.min(...sizes);
          const maxSize = Math.max(...sizes);
          const avgSize = Math.round(sizes.reduce((a, b) => a + b, 0) / sizes.length);
          console.log(`📊 包大小分佈: 最小=${minSize}b, 最大=${maxSize}b, 平均=${avgSize}b`);
        } else {
          console.warn('⚠️ WebCodecs 錄音沒有收集到獨立包');
          setError('錄音失敗：沒有收集到音頻包數據');
        }
        
      } catch (error) {
        console.error('🚨 WebCodecs 停止錄音時出錯:', error);
        setError('停止錄音時發生錯誤');
      }
      
    } else if (mediaRecorder && mediaRecorder.state === 'recording') {
      console.log('🛑 停止 MediaRecorder 錄音');
      mediaRecorder.stop();
    }
    
    setIsRecording(false);
    
    // 清理計時器
    if (recordingInterval) {
      clearInterval(recordingInterval);
      recordingInterval = null;
    }
  };

  const uploadAndProcess = async () => {
    const blob = audioBlob();
    if (!blob) {
      setError('沒有音頻數據可上傳');
      return;
    }
    
    setIsUploading(true);
    setError(null);
    
    try {
      const formData = new FormData();
      const mimeType = blob.type;
      const browser = browserInfo();
      
      // 🚀 智能端點選擇 - WebCodecs vs MediaRecorder
      let endpoint: string;
      let filename: string;
      
      if (mimeType === 'application/json' && browser?.recordingMethod === 'webcodecs') {
        // 🚀 WebCodecs 獨立包模式 - 修復版實現
        endpoint = '/upload';
        filename = 'webcodecs-packets.json';
        
        // 上傳 JSON 格式的獨立包數據
        formData.append('audio_packets', blob, filename);
        
        console.log(`🚀 WebCodecs 獨立包上傳 - 檔案: ${filename}, MIME: ${mimeType}, 大小: ${blob.size} bytes`);
        console.log('🎯 使用統一端點，JSON 格式自動檢測');
        
        const response = await fetch(endpoint, {
          method: 'POST',
          body: formData,
        });
        
        if (!response.ok) {
          const errorData: ErrorResponse = await response.json();
          throw new Error(errorData.error || `HTTP ${response.status}`);
        }
        
        const data: TranscriptResult = await response.json();
        setResult(data);
        setAudioBlob(null);
        
        console.log('✅ WebCodecs 獨立包上傳成功');
        return;
      } else {
        // MediaRecorder 傳統格式 - 統一使用標準端點
        endpoint = '/upload';
        
        // 業界領先：智能檔名生成
        filename = 'recording';
        if (mimeType.includes('webm')) filename += '.webm';
        else if (mimeType.includes('ogg')) filename += '.ogg';
        else if (mimeType.includes('mp4')) filename += '.mp4';
        else if (mimeType.includes('wav')) filename += '.wav';
        else filename += browser?.ext || '.webm';
        
        console.log(`📼 MediaRecorder 上傳 - 檔案: ${filename}, MIME: ${mimeType}, 瀏覽器: ${browser?.name}`);
        console.log('🎯 使用統一端點，二進制格式自動檢測');
        
        // 對於二進制格式，使用標準的 audio 欄位名
        formData.append('audio', blob, filename);
      }
      
      // 發送到對應的後端端點
      const response = await fetch(endpoint, {
        method: 'POST',
        body: formData,
      });
      
      if (!response.ok) {
        // WebCodecs 統一端點處理所有格式，無需降級
        
        const errorData: ErrorResponse = await response.json();
        throw new Error(errorData.error || `HTTP ${response.status}`);
      }
      
      const data: TranscriptResult = await response.json();
      setResult(data);
      setAudioBlob(null); // Clear the audio blob after successful upload
      
      console.log(`✅ 上傳成功`);
      
    } catch (err) {
      console.error('Upload failed:', err);
      setError(err instanceof Error ? err.message : '上傳失敗，請重試');
    } finally {
      setIsUploading(false);
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const clearResults = () => {
    setResult(null);
    setError(null);
    setAudioBlob(null);
    setRecordingTime(0);
  };

  return (
    <div class="container">
      <div class="card">
        <h1 style="text-align: center; color: #1f2937; margin-bottom: 16px;">
          🎙️ Care Voice AI 語音轉錄系統
        </h1>
        
        {/* 系統狀態顯示 - 業界領先 */}
        <div style="text-align: center; margin-bottom: 24px; padding: 12px; background: #f9fafb; border-radius: 8px;">
          <Show when={browserInfo()}>
            <div style="font-size: 14px; color: #6b7280; margin-bottom: 8px;">
              🌐 瀏覽器: <strong>{browserInfo()?.name}</strong> | 
              🎵 格式: <strong>{browserInfo()?.mimeType}</strong> |
              {browserInfo()?.isSupported ? 
                <span style="color: #059669;"> ✅ 完全支援</span> : 
                <span style="color: #dc2626;"> ⚠️ 部分支援</span>
              }
            </div>
            <div style="font-size: 13px; color: #4b5563; margin-bottom: 8px;">
              🚀 錄音技術: <strong>{browserInfo()?.recordingMethod === 'webcodecs' ? 'WebCodecs (硬體加速)' : 'MediaRecorder (相容模式)'}</strong> |
              {browserInfo()?.webCodecsSupported ? 
                <span style="color: #059669;"> ✅ 2025年業界領先</span> : 
                <span style="color: #f59e0b;"> ⚠️ 傳統技術</span>
              }
            </div>
          </Show>
          
          <Show when={healthStatus()}>
            <div style={`font-size: 14px; margin-bottom: 4px; color: ${isHealthy() ? '#059669' : '#dc2626'};`}>
              {isHealthy() ? '🟢' : '🔴'} 服務狀態: <strong>{healthStatus()?.status}</strong>
            </div>
          </Show>
          
          <button 
            onClick={performHealthCheck} 
            style="font-size: 12px; padding: 4px 8px; background: #6b7280; color: white; border: none; border-radius: 4px; cursor: pointer;"
          >
            🔄 重新檢查
          </button>
        </div>
        
        {/* 錄音控制 - 業界領先介面 */}
        <div style="text-align: center; margin-bottom: 24px;">
          <Show when={!isRecording() && !audioBlob()}>
            <button 
              onClick={startRecording} 
              disabled={isUploading() || !isHealthy()}
              style={`padding: 12px 24px; font-size: 16px; border: none; border-radius: 8px; cursor: pointer; transition: all 0.2s; ${
                !isHealthy() ? 'background: #9ca3af; color: white;' : 'background: #3b82f6; color: white;'
              }`}
            >
              🎤 開始高品質錄音
            </button>
            {!isHealthy() && (
              <div style="font-size: 12px; color: #dc2626; margin-top: 8px;">
                ⚠️ 服務未就緒，請檢查連接
              </div>
            )}
          </Show>
          
          <Show when={isRecording()}>
            <div style="margin-bottom: 16px; padding: 16px; background: #fee2e2; border-radius: 8px;">
              <div style="font-size: 24px; font-weight: bold; color: #dc2626; margin-bottom: 8px; animation: pulse 1s infinite;">
                🔴 正在錄音...
              </div>
              <div style="font-size: 18px; color: #6b7280; margin-bottom: 8px;">
                ⏱️ {formatTime(recordingTime())}
              </div>
              <div style="font-size: 12px; color: #6b7280;">
                🎵 格式: {browserInfo()?.mimeType} | 🌐 瀏覽器: {browserInfo()?.name}
              </div>
            </div>
            <button 
              onClick={stopRecording} 
              style="padding: 12px 24px; font-size: 16px; background: #dc2626; color: white; border: none; border-radius: 8px; cursor: pointer; animation: pulse 1s infinite;"
            >
              ⏹️ 停止錄音
            </button>
          </Show>
          
          <Show when={audioBlob() && !isUploading()}>
            <div style="margin-bottom: 16px; padding: 16px; background: #d1fae5; border-radius: 8px;">
              <div style="color: #059669; font-weight: bold; margin-bottom: 8px;">
                ✅ 錄音完成！({formatTime(recordingTime())})
              </div>
              <div style="font-size: 12px; color: #6b7280;">
                📁 檔案大小: {Math.round((audioBlob()?.size || 0) / 1024)} KB | 
                🎵 格式: {audioBlob()?.type === 'application/json' ? 'WebCodecs 獨立包' : audioBlob()?.type} | 
                🌐 瀏覽器: {browserInfo()?.name}
              </div>
            </div>
            <button 
              onClick={uploadAndProcess} 
              style="padding: 12px 24px; font-size: 16px; background: #059669; color: white; border: none; border-radius: 8px; cursor: pointer; margin-right: 8px;"
            >
              🚀 AI 轉錄處理
            </button>
            <button 
              onClick={clearResults} 
              style="padding: 12px 24px; font-size: 16px; background: #6b7280; color: white; border: none; border-radius: 8px; cursor: pointer;"
            >
              🔄 重新錄音
            </button>
          </Show>
        </div>
        
        {/* AI 處理狀態 - 業界領先視覺化 */}
        <Show when={isUploading()}>
          <div style="padding: 20px; background: #dbeafe; border-radius: 8px; text-align: center; margin: 16px 0;">
            <div style="font-size: 20px; margin-bottom: 12px; animation: pulse 1s infinite;">🤖 AI 處理中...</div>
            <div style="font-size: 14px; color: #1e40af; margin-bottom: 8px;">正在使用 Whisper AI 轉錄音頻並生成摘要</div>
            <div style="font-size: 12px; color: #6b7280;">
              🎵 音頻格式: {audioBlob()?.type === 'application/json' ? 'WebCodecs 獨立包' : audioBlob()?.type} | 📁 大小: {Math.round((audioBlob()?.size || 0) / 1024)} KB
            </div>
            <div style="width: 100%; height: 4px; background: #e5e7eb; border-radius: 2px; overflow: hidden; margin-top: 12px;">
              <div style="height: 100%; background: #3b82f6; width: 100%; animation: progress 2s linear infinite;"></div>
            </div>
          </div>
        </Show>
        
        {/* 錯誤顯示 - 業界領先錯誤處理 */}
        <Show when={error()}>
          <div style="padding: 16px; background: #fee2e2; border: 1px solid #fecaca; border-radius: 8px; margin: 16px 0;">
            <div style="font-weight: bold; margin-bottom: 8px; color: #dc2626;">❌ 系統錯誤</div>
            <div style="color: #dc2626; margin-bottom: 8px;">{error()}</div>
            <div style="font-size: 12px; color: #6b7280;">💡 建議: 檢查麥克風權限和網路連接</div>
          </div>
        </Show>
      </div>
      
      {/* 結果顯示 - 業界領先結果展示 */}
      <Show when={result()}>
        <div style="background: white; border-radius: 12px; padding: 24px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); margin-bottom: 24px;">
          <h2 style="color: #1f2937; margin-bottom: 16px; display: flex; align-items: center;">
            📝 完整逐字稿
            <span style="font-size: 12px; background: #d1fae5; color: #059669; padding: 4px 8px; border-radius: 4px; margin-left: 12px;">AI 轉錄完成</span>
          </h2>
          <div style="background: #f9fafb; padding: 16px; border-radius: 8px; line-height: 1.6; white-space: pre-wrap;">
            {result()?.full_transcript || '暫無轉錄結果'}
          </div>
        </div>
        
        <div style="background: white; border-radius: 12px; padding: 24px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); margin-bottom: 24px;">
          <h2 style="color: #1f2937; margin-bottom: 16px; display: flex; align-items: center;">
            🎯 關懷重點摘要
            <span style="font-size: 12px; background: #dbeafe; color: #1e40af; padding: 4px 8px; border-radius: 4px; margin-left: 12px;">AI 分析摘要</span>
          </h2>
          <div style="background: #f0f9ff; padding: 16px; border-radius: 8px; line-height: 1.6; white-space: pre-wrap;">
            {result()?.summary || '暫無摘要'}
          </div>
          
          <div style="text-align: center; margin-top: 24px;">
            <button 
              onClick={clearResults} 
              style="padding: 12px 24px; font-size: 16px; background: #059669; color: white; border: none; border-radius: 8px; cursor: pointer;"
            >
              🔄 開始新的錄音
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}

export default App;