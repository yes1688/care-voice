import { createSignal, Show } from 'solid-js';

interface TranscriptResult {
  full_transcript: string;
  summary: string;
}

interface ErrorResponse {
  error: string;
}

function App() {
  const [isRecording, setIsRecording] = createSignal(false);
  const [audioBlob, setAudioBlob] = createSignal<Blob | null>(null);
  const [isUploading, setIsUploading] = createSignal(false);
  const [result, setResult] = createSignal<TranscriptResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [recordingTime, setRecordingTime] = createSignal(0);
  
  let mediaRecorder: MediaRecorder | null = null;
  let recordingInterval: number | null = null;

  const startRecording = async () => {
    try {
      setError(null);
      setResult(null);
      
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
      // 強制使用 WAV 格式以支援語音識別（無需 ffmpeg）
      const options: MediaRecorderOptions = {};
      const wavFormats = [
        'audio/wav',
        'audio/wave', 
        'audio/x-wav'
      ];
      
      // 嘗試 WAV 格式
      for (const format of wavFormats) {
        if (MediaRecorder.isTypeSupported(format)) {
          options.mimeType = format;
          console.log(`✅ 使用 WAV 格式: ${format} (支援直接語音識別)`);
          break;
        }
      }
      
      // 如果不支援 WAV，使用 WebM 但會說明需要轉換
      if (!options.mimeType) {
        const fallbackFormats = ['audio/webm', 'audio/ogg'];
        for (const format of fallbackFormats) {
          if (MediaRecorder.isTypeSupported(format)) {
            options.mimeType = format;
            console.log(`⚠️ 使用 ${format} (需要服務器端轉換)`);
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
        
        console.log(`錄音完成，格式: ${finalMimeType}, 大小: ${blob.size} bytes`);
        
        // Stop all tracks to free up the microphone
        stream.getTracks().forEach(track => track.stop());
        
        if (recordingInterval) {
          clearInterval(recordingInterval);
          recordingInterval = null;
        }
      };
      
      mediaRecorder.start();
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
    if (mediaRecorder && mediaRecorder.state === 'recording') {
      mediaRecorder.stop();
      setIsRecording(false);
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
      // Use appropriate file extension based on MIME type (Gemini API compatible only)
      const mimeType = blob.type;
      let filename = 'recording';
      if (mimeType.includes('wav')) filename += '.wav';
      else if (mimeType.includes('mpeg') || mimeType.includes('mp3')) filename += '.mp3';
      else if (mimeType.includes('aac')) filename += '.aac';
      else if (mimeType.includes('ogg')) filename += '.ogg';
      else if (mimeType.includes('flac')) filename += '.flac';
      else filename += '.ogg'; // fallback to supported format
      
      console.log(`上傳音頻檔案: ${filename}, MIME類型: ${mimeType}`);
      formData.append('audio', blob, filename);
      
      // 使用相對路徑調用 API (nginx 代理到後端)
      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      });
      
      if (!response.ok) {
        const errorData: ErrorResponse = await response.json();
        throw new Error(errorData.error || `HTTP ${response.status}`);
      }
      
      const data: TranscriptResult = await response.json();
      setResult(data);
      setAudioBlob(null); // Clear the audio blob after successful upload
      
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
        <h1 style="text-align: center; color: #1f2937; margin-bottom: 32px;">
          錄音轉文字系統
        </h1>
        
        {/* Recording Controls */}
        <div style="text-align: center; margin-bottom: 24px;">
          <Show when={!isRecording() && !audioBlob()}>
            <button onClick={startRecording} disabled={isUploading()}>
              🎤 開始錄音
            </button>
          </Show>
          
          <Show when={isRecording()}>
            <div style="margin-bottom: 16px;">
              <div style="font-size: 24px; font-weight: bold; color: #dc2626; margin-bottom: 8px;">
                ● 錄音中...
              </div>
              <div style="font-size: 18px; color: #6b7280;">
                {formatTime(recordingTime())}
              </div>
            </div>
            <button onClick={stopRecording} class="recording">
              ⏹️ 停止錄音
            </button>
          </Show>
          
          <Show when={audioBlob() && !isUploading()}>
            <div style="margin-bottom: 16px;">
              <div style="color: #059669; font-weight: bold; margin-bottom: 8px;">
                ✅ 錄音完成 ({formatTime(recordingTime())})
              </div>
            </div>
            <button onClick={uploadAndProcess} style="background: #059669;">
              📤 轉換為文字
            </button>
            <button onClick={clearResults} style="background: #6b7280;">
              🗑️ 重新錄音
            </button>
          </Show>
        </div>
        
        {/* Processing State */}
        <Show when={isUploading()}>
          <div class="result loading">
            <div style="font-size: 18px; margin-bottom: 8px;">🤖 AI 處理中...</div>
            <div>正在轉錄音頻並生成摘要，請稍候...</div>
          </div>
        </Show>
        
        {/* Error Display */}
        <Show when={error()}>
          <div class="result error">
            <div style="font-weight: bold; margin-bottom: 8px;">❌ 錯誤</div>
            <div>{error()}</div>
          </div>
        </Show>
      </div>
      
      {/* Results Display */}
      <Show when={result()}>
        <div class="card">
          <h2 style="color: #1f2937; margin-bottom: 16px;">📝 完整逐字稿</h2>
          <div class="result">
            {result()?.full_transcript}
          </div>
        </div>
        
        <div class="card">
          <h2 style="color: #1f2937; margin-bottom: 16px;">🎯 關懷重點摘要</h2>
          <div class="result">
            {result()?.summary}
          </div>
          
          <div style="text-align: center; margin-top: 24px;">
            <button onClick={clearResults} style="background: #059669;">
              🔄 開始新的錄音
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}

export default App;