// 啟用 jemalloc 記憶體分配器以提升 musl 環境性能
#[cfg(feature = "jemalloc")]
use jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, error, warn};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

// 新增音頻處理模組
mod audio_format;
mod opus_decoder;
mod audio_decoder;

use audio_format::AudioFormat;
use audio_decoder::UnifiedAudioDecoder;

// 全域統計計數器
static WAV_COUNT: AtomicU64 = AtomicU64::new(0);
static WEBM_OPUS_COUNT: AtomicU64 = AtomicU64::new(0);
static WEBM_VORBIS_COUNT: AtomicU64 = AtomicU64::new(0);
static CONVERSION_SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static CONVERSION_FAILURE_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct TranscriptResponse {
    full_transcript: String,
    summary: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Whisper 服務結構
struct WhisperService {
    context: WhisperContext,
}

impl WhisperService {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("📋 WhisperService::new() - Starting initialization");
        info!("Initializing Whisper service...");
        
        // 載入模型 (使用相對路徑)
        let model_path = "./models/ggml-base.bin";
        println!("📁 Loading Whisper model from: {}", model_path);
        info!("Loading Whisper model from: {}", model_path);
        
        // 檢查文件是否存在
        if !std::path::Path::new(model_path).exists() {
            println!("❌ Model file not found: {}", model_path);
            return Err(format!("Model file not found: {}", model_path).into());
        }
        
        println!("🔄 Creating WhisperContext...");
        let ctx = match WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        ) {
            Ok(ctx) => {
                println!("✅ WhisperContext created successfully");
                ctx
            },
            Err(e) => {
                println!("❌ WhisperContext creation failed: {}", e);
                return Err(e.into());
            }
        };
        
        println!("✅ WhisperService initialized successfully!");
        info!("Whisper service initialized successfully!");
        Ok(Self { context: ctx })
    }
    
    async fn transcribe(&self, audio_samples: &[f32]) -> Result<String, Box<dyn std::error::Error>> {
        info!("Starting transcription for {} samples", audio_samples.len());
        
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 設置中文語言 (可選)
        params.set_language(Some("zh"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        
        let mut state = self.context.create_state()?;
        state.full(params, audio_samples)?;
        
        // 收集所有文字片段
        let mut full_text = String::new();
        let num_segments = state.full_n_segments()?;
        
        info!("Transcription completed with {} segments", num_segments);
        
        for i in 0..num_segments {
            let segment_text = state.full_get_segment_text(i)?;
            full_text.push_str(&segment_text);
        }
        
        info!("Full transcript: {}", full_text);
        Ok(full_text)
    }
}

// 主函數 - 包含 Whisper 服務初始化
#[tokio::main]
async fn main() {
    // 初始化日誌 - 強制輸出到 stdout，確保 supervisord 能捕獲
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "care_voice=info,whisper_rs=info".to_string())
        )
        .init();

    println!("🚀 Starting Care Voice backend with whisper-rs...");
    info!("Starting Care Voice backend with whisper-rs...");
    
    // 初始化 Whisper 服務
    println!("🔧 Initializing Whisper service...");
    let whisper_service = match WhisperService::new() {
        Ok(service) => {
            println!("✅ Whisper service initialized successfully!");
            Arc::new(service)
        },
        Err(e) => {
            println!("❌ Failed to initialize Whisper service: {}", e);
            eprintln!("❌ CRITICAL ERROR: {}", e);
            error!("Failed to initialize Whisper service: {}", e);
            std::process::exit(1);
        }
    };
    
    // CORS 配置
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(tower_http::cors::Any);
    
    let app = Router::new()
        .route("/upload", post(upload_audio))
        .route("/api/upload", post(upload_audio))  // 添加前端期望的路由
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(whisper_service);
    
    // 支援環境變數配置端口，默認 8000，在統一容器中使用 8080
    let port = std::env::var("BACKEND_PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    info!("Server running on http://{}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}

// 上傳處理 - 使用 whisper-rs
async fn upload_audio(
    State(whisper_service): State<Arc<WhisperService>>,
    mut multipart: Multipart,
) -> Result<Json<TranscriptResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Received audio upload request");
    
    // 處理 multipart 資料
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Error reading multipart field: {}", e);
        (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid multipart data".to_string() }))
    })? {
        
        if field.name() == Some("audio") {
            info!("Processing audio field");
            
            // 獲取 MIME 類型以改進格式檢測
            let content_type = field.content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            info!("音頻 MIME 類型: {}", content_type);
            
            let data = field.bytes().await.map_err(|e| {
                error!("Error reading audio bytes: {}", e);
                (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Failed to read audio data".to_string() }))
            })?;
            
            info!("Received audio data: {} bytes", data.len());
            
            // 使用新的統一音頻解碼器 (支援 Opus)
            let audio_samples = convert_to_wav_samples_with_mime(&data, &content_type).map_err(|e| {
                error!("Audio conversion failed: {}", e);
                CONVERSION_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
                
                // 根據新的解碼器提供友善的錯誤信息
                let user_message = if e.to_string().contains("Opus") {
                    "✅ Opus 格式支援已啟用！如果仍有問題，請檢查音頻品質或容器格式。"
                } else if e.to_string().contains("WebM") {
                    "WebM 容器解析問題。請嘗試：\n✅ Firefox (OGG-Opus 格式)\n🔄 重新錄製音頻"
                } else if e.to_string().contains("MP4") || e.to_string().contains("AAC") {
                    "Safari MP4-AAC 格式支援有限。建議使用：\n✅ Chrome (WebM-Opus)\n✅ Firefox (OGG-Opus)"
                } else if e.to_string().contains("無法識別") || e.to_string().contains("未知") {
                    "無法識別音頻格式。支援格式：\n✅ WebM-Opus (Chrome/Edge)\n✅ OGG-Opus (Firefox)\n✅ WAV (通用)"
                } else {
                    "音頻格式轉換失敗。請確認音頻檔案完整且使用支援的格式。"
                };
                
                (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse { 
                    error: user_message.to_string() 
                }))
            })?;
            
            // 轉換成功統計
            CONVERSION_SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
            
            info!("Audio converted to {} samples", audio_samples.len());
            
            // 使用 Whisper 轉錄
            let full_transcript = whisper_service.transcribe(&audio_samples).await.map_err(|e| {
                error!("Transcription failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Transcription failed".to_string() }))
            })?;
            
            // 生成簡化摘要
            let summary = generate_simple_summary(&full_transcript);
            
            info!("Transcription completed successfully");
            
            return Ok(Json(TranscriptResponse {
                full_transcript,
                summary,
            }));
        }
    }
    
    warn!("No audio field found in multipart data");
    Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "No audio field found".to_string() })))
}

// 新的音頻格式轉換函數 - 支援 Opus 和 MIME 類型檢測
fn convert_to_wav_samples_with_mime<'a>(
    audio_data: &'a [u8], 
    mime_type: &'a str
) -> Result<Vec<f32>, Box<dyn std::error::Error + 'a>> {
    info!("開始音頻格式轉換，數據大小: {} bytes，MIME: {}", audio_data.len(), mime_type);
    
    // 使用統一音頻解碼器處理
    match UnifiedAudioDecoder::decode_audio_with_mime(audio_data, mime_type) {
        Ok(samples) => {
            info!("音頻解碼成功：{} 樣本", samples.len());
            
            // 更新統計計數器
            match AudioFormat::detect_from_mime(mime_type) {
                AudioFormat::WebmOpus => WEBM_OPUS_COUNT.fetch_add(1, Ordering::Relaxed),
                AudioFormat::OggOpus => WEBM_OPUS_COUNT.fetch_add(1, Ordering::Relaxed), // 統一計算 Opus
                AudioFormat::Wav => WAV_COUNT.fetch_add(1, Ordering::Relaxed),
                AudioFormat::WebmVorbis => WEBM_VORBIS_COUNT.fetch_add(1, Ordering::Relaxed),
                _ => 0, // 其他格式不計數
            };
            
            Ok(samples)
        },
        Err(e) => {
            error!("統一音頻解碼器失敗: {}", e);
            
            // 如果新解碼器失敗，回退到舊方法
            warn!("回退到舊的音頻解碼方法");
            convert_to_wav_samples_legacy(audio_data)
        }
    }
}

// 舊版音頻轉換函數 (向後相容)
fn convert_to_wav_samples_legacy(audio_data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    info!("使用舊版音頻轉換方法");
    
    // 首先嘗試作為 WAV 文件讀取
    if let Ok(samples) = try_read_as_wav(audio_data) {
        info!("Successfully read as WAV format");
        WAV_COUNT.fetch_add(1, Ordering::Relaxed);
        return Ok(samples);
    }
    
    // 如果不是 WAV，嘗試使用 symphonia 解碼
    match try_decode_with_symphonia(audio_data) {
        Ok(samples) => {
            info!("Successfully decoded with symphonia");
            WEBM_VORBIS_COUNT.fetch_add(1, Ordering::Relaxed);
            Ok(samples)
        },
        Err(e) => {
            error!("Failed to decode audio: {}", e);
            Err(e)
        }
    }
}

fn try_read_as_wav(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let cursor = std::io::Cursor::new(data);
    let mut reader = hound::WavReader::new(cursor)?;
    
    let samples: Result<Vec<f32>, _> = match reader.spec().sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect()
        },
        hound::SampleFormat::Int => {
            reader.samples::<i16>()
                .map(|s| s.map(|sample| sample as f32 / 32768.0))
                .collect()
        }
    };
    
    let samples = samples?;
    
    // 確保單聲道，如果是立體聲則取平均
    let channels = reader.spec().channels as usize;
    if channels == 1 {
        Ok(samples)
    } else {
        Ok(samples.chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect())
    }
}

fn try_decode_with_symphonia(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::get_probe;
    use symphonia::core::audio::SampleBuffer;
    
    info!("開始使用 symphonia 解碼音頻數據，大小: {} bytes", data.len());
    
    // 複製數據到擁有所有權的 Vec 以避免生命週期問題
    let owned_data = data.to_vec();
    let cursor = std::io::Cursor::new(owned_data);
    let media_source = MediaSourceStream::new(Box::new(cursor), Default::default());
    
    // 創建格式提示 - 告訴 symphonia 這可能是 WebM 或 OGG 格式
    let mut hint = Hint::new();
    hint.with_extension("webm");
    hint.with_extension("ogg");
    
    // 探測格式
    let probe = get_probe();
    let probed = probe
        .format(&hint, media_source, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| {
            error!("格式探測失敗: {}", e);
            
            // 提供更詳細的錯誤信息
            let data_preview = if data.len() >= 16 {
                format!("{:02x?}", &data[..16])
            } else {
                format!("{:02x?}", data)
            };
            
            error!("音頻數據前16位元組: {}", data_preview);
            
            // 區分不同類型的格式錯誤
            match e {
                symphonia::core::errors::Error::IoError(ref io_err) 
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    "音頻文件可能已完全解析，但缺少尾部信息".to_string()
                },
                symphonia::core::errors::Error::Unsupported(_) => {
                    "不支援的音頻編解碼器，請確認已安裝所需的 symphonia 特性".to_string()
                },
                _ => format!("無法識別音頻格式: {}", e)
            }
        })?;
    
    let mut format = probed.format;
    info!("成功識別音頻格式");
    
    // 找到第一個音頻軌道
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("找不到音頻軌道")?;
    
    let track_id = track.id;
    info!("找到音頻軌道 ID: {}, 編解碼器: {:?}", track_id, track.codec_params.codec);
    
    // 創建解碼器
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| format!("無法創建解碼器: {}", e))?;
    
    let mut audio_samples: Vec<f32> = Vec::new();
    let mut sample_buffer: Option<SampleBuffer<f32>> = None;
    
    // 解碼音頻包
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::ResetRequired) => {
                // 重置解碼器
                decoder.reset();
                continue;
            },
            Err(symphonia::core::errors::Error::IoError(ref e)) 
                if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // 正常的文件結束 - 這是正常解碼完成的信號
                info!("音頻解碼正常完成 - 到達流末尾");
                break;
            },
            Err(e) => {
                // 區分真正的錯誤和正常結束
                match e {
                    symphonia::core::errors::Error::IoError(ref io_err) => {
                        if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                            info!("音頻解碼正常完成 - IO 流結束");
                            break;
                        } else {
                            error!("真正的 IO 錯誤: {}", e);
                            return Err(format!("音頻解碼 IO 錯誤: {}", e).into());
                        }
                    },
                    _ => {
                        warn!("解碼結束或遇到其他錯誤: {}", e);
                        break;
                    }
                }
            }
        };
        
        // 只處理我們感興趣的軌道
        if packet.track_id() != track_id {
            continue;
        }
        
        // 解碼音頻包
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                // 初始化樣本緩衝區（僅在第一次時）
                if sample_buffer.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;
                    sample_buffer = Some(SampleBuffer::<f32>::new(duration, spec));
                }
                
                if let Some(ref mut buf) = sample_buffer {
                    // 獲取聲道數（在 copy 之前）
                    let channels = audio_buf.spec().channels.count();
                    
                    // 將音頻數據複製到樣本緩衝區
                    buf.copy_interleaved_ref(audio_buf);
                    
                    // 獲取樣本數據
                    let samples = buf.samples();
                    
                    // 處理多聲道到單聲道的轉換
                    if channels == 1 {
                        // 單聲道，直接添加
                        audio_samples.extend_from_slice(samples);
                    } else {
                        // 多聲道，轉換為單聲道（取平均值）
                        for chunk in samples.chunks(channels) {
                            let mono_sample: f32 = chunk.iter().sum::<f32>() / channels as f32;
                            audio_samples.push(mono_sample);
                        }
                    }
                }
            },
            Err(e) => {
                warn!("解碼音頻包時出錯: {}", e);
                continue;
            }
        }
    }
    
    if audio_samples.is_empty() {
        return Err("沒有解碼出任何音頻數據".into());
    }
    
    info!("成功解碼 {} 個音頻樣本", audio_samples.len());
    
    // 確保樣本在合理範圍內 (-1.0 到 1.0)
    let max_abs = audio_samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_abs > 1.0 {
        info!("音頻樣本超出範圍，進行標準化，最大值: {}", max_abs);
        for sample in &mut audio_samples {
            *sample /= max_abs;
        }
    }
    
    // Whisper 通常期望 16kHz 採樣率
    // 注意：這裡簡化處理，實際情況可能需要重採樣
    info!("音頻解碼完成，輸出 {} 個 PCM 樣本", audio_samples.len());
    
    Ok(audio_samples)
}

// 簡單摘要生成 (可替換為更智能的方案)
fn generate_simple_summary(transcript: &str) -> String {
    if transcript.trim().is_empty() {
        return "無法生成摘要：轉錄文字為空".to_string();
    }
    
    // 簡化版摘要 - 取前200字
    let summary = if transcript.len() > 200 {
        format!("{}...", &transcript[..200])
    } else {
        transcript.to_string()
    };
    
    // 添加關懷重點提示
    format!("關懷摘要：{}", summary.trim())
}

async fn health_check() -> Json<serde_json::Value> {
    // 取得音頻格式支援資訊
    let format_support = UnifiedAudioDecoder::get_format_support_info();
    let support_info: serde_json::Value = format_support
        .into_iter()
        .map(|(format, status)| {
            serde_json::json!({
                "format": format.friendly_name(),
                "status": status
            })
        })
        .collect::<Vec<_>>()
        .into();

    // 統計計數器
    let stats = serde_json::json!({
        "wav_processed": WAV_COUNT.load(Ordering::Relaxed),
        "webm_opus_processed": WEBM_OPUS_COUNT.load(Ordering::Relaxed),
        "webm_vorbis_processed": WEBM_VORBIS_COUNT.load(Ordering::Relaxed),
        "conversion_success": CONVERSION_SUCCESS_COUNT.load(Ordering::Relaxed),
        "conversion_failure": CONVERSION_FAILURE_COUNT.load(Ordering::Relaxed),
    });

    Json(serde_json::json!({
        "status": "healthy",
        "service": "Care Voice with whisper-rs + Opus Support",
        "version": "2.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "audio_formats": support_info,
        "processing_stats": stats,
        "enhancements": [
            "✅ Opus 音頻解碼支援 (Chrome/Firefox/Edge)",
            "✅ 智能格式檢測 (MIME + 二進制)",
            "✅ 統一音頻解碼器架構",
            "✅ 向後相容性保證",
            "🔄 WebM 容器支援 (基礎版本)"
        ]
    }))
}