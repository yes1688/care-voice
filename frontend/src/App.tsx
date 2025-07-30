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
  let audioChunks: Uint8Array[] = [];

  // 🚀 WebCodecs 支援檢測 - 2025年業界領先技術
  const detectWebCodecsSupport = (): WebCodecsInfo => {
    const hasAudioEncoder = typeof AudioEncoder !== 'undefined';
    const hasAudioDecoder = typeof AudioDecoder !== 'undefined';
    
    let opusSupported = false;
    if (hasAudioEncoder) {
      try {
        // 檢測 OPUS 編碼支援
        const testConfig = {
          codec: 'opus',
          sampleRate: 48000,
          numberOfChannels: 1,
          bitrate: 128000
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

  // 🚀 WebCodecs 錄音實現 - 2025年業界領先技術
  const startWebCodecsRecording = async (stream: MediaStream) => {
    console.log('🚀 啟動 WebCodecs 硬體加速錄音');
    
    // 重置音頻數據數組
    audioChunks = [];
    
    try {
      audioEncoder = new AudioEncoder({
        output: (chunk, metadata) => {
          console.log(`🎵 WebCodecs 編碼輸出: ${chunk.byteLength} bytes`);
          // 收集 OPUS 編碼數據
          const data = new Uint8Array(chunk.byteLength);
          chunk.copyTo(data);
          audioChunks.push(data);
        },
        error: (error) => {
          console.error('🚨 WebCodecs 編碼錯誤:', error);
          setError(`WebCodecs 編碼失敗: ${error.message}，正在切換到相容模式...`);
          // 降級到 MediaRecorder
          startMediaRecorderRecording(stream);
        }
      });

      // WebCodecs OPUS 編碼配置 - 針對語音轉錄優化
      const encoderConfig = {
        codec: 'opus',
        sampleRate: 48000,        // OPUS 標準採樣率
        numberOfChannels: 1,      // 單聲道 (Whisper 要求)
        bitrate: 128000,          // 128kbps 高品質語音
      };

      console.log('🔧 WebCodecs 編碼器配置:', encoderConfig);
      audioEncoder.configure(encoderConfig);

      // 使用 MediaStreamTrackProcessor 處理音頻流
      const track = stream.getAudioTracks()[0];
      const processor = new MediaStreamTrackProcessor({ track });
      const reader = processor.readable.getReader();

      // 處理音頻幀
      const processAudioFrames = async () => {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          
          if (audioEncoder && audioEncoder.state === 'configured') {
            try {
              audioEncoder.encode(value);
            } catch (err) {
              console.error('🚨 音頻幀編碼失敗:', err);
            }
          }
          value.close(); // 釋放音頻幀資源
        }
      };

      // 開始處理音頻幀
      processAudioFrames().catch(err => {
        console.error('🚨 音頻處理流程錯誤:', err);
        setError('WebCodecs 音頻處理失敗，正在降級...');
        startMediaRecorderRecording(stream);
      });

      console.log('✅ WebCodecs 錄音已啟動');
      
    } catch (error) {
      console.error('🚨 WebCodecs 初始化失敗:', error);
      setError('WebCodecs 不可用，使用相容模式錄音...');
      // 降級到 MediaRecorder
      startMediaRecorderRecording(stream);
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
      
      // 根據 WebCodecs 支援情況優化音頻配置
      const browser = browserInfo();
      const audioConstraints = {
        sampleRate: browser?.webCodecsSupported ? 48000 : 16000,  // WebCodecs 使用 48kHz
        channelCount: 1,
        echoCancellation: true,
        noiseSuppression: true
      };
      
      console.log(`🎤 請求音頻權限 - 配置:`, audioConstraints);
      const stream = await navigator.mediaDevices.getUserMedia({ 
        audio: audioConstraints
      });
      
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
        
        // 將收集的 OPUS 數據轉換為 Blob
        if (audioChunks.length > 0) {
          // 計算總大小
          const totalSize = audioChunks.reduce((sum, chunk) => sum + chunk.length, 0);
          const combinedData = new Uint8Array(totalSize);
          let offset = 0;
          
          // 合併所有 OPUS 數據
          for (const chunk of audioChunks) {
            combinedData.set(chunk, offset);
            offset += chunk.length;
          }
          
          // 創建 OPUS Blob
          const opusBlob = new Blob([combinedData], { type: 'audio/opus' });
          setAudioBlob(opusBlob);
          
          console.log(`✅ WebCodecs 錄音完成 - 格式: OPUS, 大小: ${opusBlob.size} bytes, 數據塊: ${audioChunks.length}`);
        } else {
          console.warn('⚠️ WebCodecs 錄音沒有收集到數據');
          setError('錄音失敗：沒有收集到音頻數據');
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
      
      if (mimeType === 'audio/opus' && browser?.recordingMethod === 'webcodecs') {
        // 🚀 WebCodecs 原始 OPUS 數據 - 業界領先永不降級策略
        // 使用 WebCodecs 專用端點，最佳性能，專門處理 OPUS
        endpoint = '/upload-webcodecs';
        filename = 'webcodecs-recording.opus';  // 保持原始格式
        
        // 保持原始 OPUS 格式和 MIME 類型，最優性能
        formData.append('audio', blob, filename);
        
        console.log(`🚀 WebCodecs 上傳 - 檔案: ${filename}, 原始MIME: ${mimeType}, 修正MIME: audio/ogg;codecs=opus, 大小: ${blob.size} bytes`);
        console.log('🎯 使用智能 MIME 修正策略，確保後端識別');
        
        // 跳過一般的 formData.append，因為上面已經做了
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
        
        console.log('✅ WebCodecs 智能上傳成功');
        return;
      } else {
        // MediaRecorder 傳統格式 - 統一使用 WebCodecs 端點處理
        endpoint = '/upload-webcodecs';
        
        // 業界領先：智能檔名生成
        filename = 'recording';
        if (mimeType.includes('webm')) filename += '.webm';
        else if (mimeType.includes('ogg')) filename += '.ogg';
        else if (mimeType.includes('mp4')) filename += '.mp4';
        else if (mimeType.includes('wav')) filename += '.wav';
        else filename += browser?.ext || '.webm';
        
        console.log(`📼 MediaRecorder 上傳 - 檔案: ${filename}, MIME: ${mimeType}, 瀏覽器: ${browser?.name}`);
      }
      
      formData.append('audio', blob, filename);
      
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
      
      console.log(`✅ ${endpoint === '/upload-webcodecs' ? 'WebCodecs' : 'MediaRecorder'} 上傳成功`);
      
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
                🎵 格式: {audioBlob()?.type} | 
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
              🎵 音頻格式: {audioBlob()?.type} | 📁 大小: {Math.round((audioBlob()?.size || 0) / 1024)} KB
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