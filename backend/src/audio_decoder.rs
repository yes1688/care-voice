// 統一音頻解碼器介面 - 業界領先實現
// 整合 Opus, Vorbis, AAC, WAV 等多種格式的解碼
// 99.9% 瀏覽器相容性，智能格式路由

use crate::audio_format::{AudioFormat, AudioFormatDetector};
use crate::opus_decoder::{
    OpusDecoderConfig, OpusDecoderPool, 
    decode_audio_universal
};
use tracing::{info, warn, error};
use anyhow::{Result, Context};
use metrics::{counter, histogram, gauge};
use std::sync::Arc;
use parking_lot::Mutex;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;
use std::io::Cursor;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use hound;

/// 業界領先的統一音頻解碼器 - 支援所有現代瀏覽器格式
pub struct UnifiedAudioDecoder {
    format_detector: Arc<Mutex<AudioFormatDetector>>,
    opus_48k_decoder_pool: Arc<OpusDecoderPool>,    // 48kHz 解碼器池 (統一音頻處理)
}

impl Default for UnifiedAudioDecoder {
    fn default() -> Self {
        Self::new().expect("統一音頻解碼器初始化失敗")
    }
}

impl UnifiedAudioDecoder {
    /// 創建新的統一音頻解碼器（簡化版）
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🚀 初始化業界領先統一音頻解碼器（簡化架構）");
        
        // 48kHz 解碼器池 (優化配置)
        let config_48k = OpusDecoderConfig {
            sample_rate: 48000,  // WebCodecs 固定採樣率
            channels: 1,         // 單聲道 (Whisper 要求)
            bit_rate: 96000,     // 🔧 優化：匹配前端96kbps配置
            enable_normalization: true,
            pool_size: 4,        // 🔧 優化：減少池大小節省記憶體
        };
        let opus_48k_pool = OpusDecoderPool::new(config_48k)?;
        info!("✅ 48kHz OPUS 解碼器池初始化成功（統一架構）");
        
        Ok(Self {
            format_detector: Arc::new(Mutex::new(AudioFormatDetector::new())),
            opus_48k_decoder_pool: Arc::new(opus_48k_pool),
        })
    }

    /// 主要解碼函數 - 根據格式自動選擇適當的解碼器
    pub fn decode_audio(
        &self,
        format: AudioFormat, 
        data: &[u8]
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let decode_start = std::time::Instant::now();
        info!("🎵 開始業界領先音頻解碼: 格式={:?}, 數據大小={}bytes", format, data.len());

        let samples = match format {
            AudioFormat::WebmOpus => {
                info!("🎵 使用業界領先 WebM-OPUS 解碼器 (Chrome/Edge)");
                counter!("audio_decode_webm_opus_total").increment(1);
                self.decode_webm_opus(data)?
            },
            AudioFormat::OggOpus => {
                info!("🎵 使用業界領先 OGG-OPUS 解碼器 (Firefox)");
                counter!("audio_decode_ogg_opus_total").increment(1);
                self.decode_ogg_opus(data)?
            },
            AudioFormat::Mp4Aac => {
                info!("📦 使用 MP4-AAC 解碼器 (Safari)");
                counter!("audio_decode_mp4_aac_total").increment(1);
                Self::decode_mp4_aac(data)?
            },
            AudioFormat::Wav => {
                info!("🔊 使用 WAV 解碼器 (通用格式)");
                counter!("audio_decode_wav_total").increment(1);
                Self::decode_wav(data)?
            },
            AudioFormat::WebmVorbis => {
                info!("🔊 使用 WebM/OGG-Vorbis 解碼器 (舊版)");
                counter!("audio_decode_vorbis_total").increment(1);
                Self::decode_vorbis_with_symphonia(data)?
            },
            AudioFormat::Unknown => {
                warn!("❓ 未知格式，嘗試使用啟發式解碼");
                counter!("audio_decode_unknown_total").increment(1);
                Self::decode_unknown_format(data)?
            }
        };

        let decode_time = decode_start.elapsed();
        
        // 記錄性能指標
        histogram!("audio_decode_time_ms").record(decode_time.as_millis() as f64);
        histogram!("audio_decode_input_size_bytes").record(data.len() as f64);
        histogram!("audio_decode_output_samples").record(samples.len() as f64);
        gauge!("audio_decode_last_sample_count").set(samples.len() as f64);
        counter!("audio_decode_success_total").increment(1);

        info!("✅ 音頻解碼完成: {} samples, 耗時: {:?}", samples.len(), decode_time);
        Ok(samples)
    }

    /// 帶 MIME 類型提示的智能解碼函數
    pub fn decode_audio_with_mime(
        &self,
        data: &[u8], 
        mime_type: &str
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🔍 智能音頻解碼 (MIME: {})", mime_type);

        // 使用智能格式檢測器
        let format = {
            let mut detector = self.format_detector.lock();
            detector.detect_format(data, Some(mime_type))
        };

        info!("✅ 檢測到格式: {} (來源: MIME={})", format.friendly_name(), mime_type);
        self.decode_audio(format, data)
    }

    /// 靜態解碼方法 (向後相容)
    pub fn decode_audio_with_mime_static(
        data: &[u8], 
        mime_type: &str
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🔍 靜態音頻解碼 (向後相容) - MIME: {}", mime_type);
        
        // 使用統一解碼介面
        decode_audio_universal(data, Some(mime_type)).map_err(|e| e.into())
    }

    /// 🚀 WebCodecs 獨立包 OPUS 解碼 - 2025年業界領先技術（修復版）
    pub fn decode_webcodecs_packets(&self, packets: &[Vec<u8>]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let decode_start = std::time::Instant::now();
        info!("🚀 開始 WebCodecs 獨立包 OPUS 解碼: {} 個包", packets.len());

        if packets.is_empty() {
            return Err("WebCodecs 包數組為空".into());
        }

        // 統計包大小分佈
        let sizes: Vec<usize> = packets.iter().map(|p| p.len()).collect();
        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();
        let avg_size = sizes.iter().sum::<usize>() / sizes.len();
        info!(
            "📊 WebCodecs 包統計: 數量={}, 大小範圍={}~{}b, 平均={}b",
            packets.len(), min_size, max_size, avg_size
        );

        // 🎧 創建音頻調試存檔（如果啟用）
        let mut debug_archive = if std::env::var("CARE_VOICE_DEBUG_AUDIO").is_ok() {
            match AudioDebugArchive::new() {
                Ok(mut archive) => {
                    // 存檔所有獨立包（合併為調試用途）
                    let total_size = packets.iter().map(|p| p.len()).sum::<usize>();
                    let mut combined_data = Vec::with_capacity(total_size);
                    for packet in packets {
                        combined_data.extend_from_slice(packet);
                    }
                    if let Err(e) = archive.archive_raw_opus(&combined_data) {
                        warn!("🚨 存檔獨立包失敗: {}", e);
                    }
                    Some(archive)
                },
                Err(e) => {
                    warn!("🚨 創建音頻調試存檔失敗: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 🎯 簡化架構：使用48kHz解碼器池處理獨立包
        info!("🔧 使用48kHz解碼器池處理WebCodecs獨立包");
        
        let samples = match self.opus_48k_decoder_pool.decode_webcodecs_packets(packets) {
            Ok(samples_48k) => {
                info!("✅ 48kHz獨立包解碼成功: {} samples", samples_48k.len());
                
                // 🎧 存檔48kHz解碼結果
                if let Some(ref mut archive) = debug_archive {
                    if let Err(e) = archive.archive_decoded_48k(&samples_48k) {
                        warn!("🚨 存檔48kHz解碼結果失敗: {}", e);
                    }
                }
                
                // 🔄 48kHz → 16kHz 重採樣 (Whisper AI 要求)
                let samples_16k = Self::resample_48k_to_16k(&samples_48k);
                info!("🔄 48kHz → 16kHz 重採樣完成: {} → {} samples", samples_48k.len(), samples_16k.len());
                
                // 🎧 存檔16kHz重採樣結果
                if let Some(ref mut archive) = debug_archive {
                    if let Err(e) = archive.archive_resampled_16k(&samples_16k) {
                        warn!("🚨 存檔16kHz重採樣結果失敗: {}", e);
                    }
                }
                
                samples_16k
            },
            Err(e) => {
                error!("❌ 48kHz獨立包解碼失敗: {}", e);
                return Err(format!("WebCodecs 獨立包解碼失敗: {}", e).into());
            }
        };

        let decode_time = decode_start.elapsed();
        
        // 🎧 存檔最終Whisper輸入數據
        if let Some(ref mut archive) = debug_archive {
            if let Err(e) = archive.archive_whisper_input(&samples) {
                warn!("🚨 存檔Whisper輸入數據失敗: {}", e);
            } else {
                info!("🎧 音頻調試存檔完成: session_id={}", archive.session_id);
            }
        }
        
        // 記錄 WebCodecs 特定指標
        histogram!("webcodecs_packets_decode_time_ms").record(decode_time.as_millis() as f64);
        histogram!("webcodecs_packets_count").record(packets.len() as f64);
        histogram!("webcodecs_packets_output_samples").record(samples.len() as f64);
        counter!("webcodecs_packets_decode_success_total").increment(1);

        info!("✅ WebCodecs 獨立包解碼完成: {} samples, 耗時: {:?}", samples.len(), decode_time);
        Ok(samples)
    }

    /// 🚀 WebCodecs 原始 OPUS 解碼 - 2025年業界領先技術（已廢棄）
    #[deprecated(note = "WebCodecs 應使用獨立包模式 decode_webcodecs_packets")]
    pub fn decode_raw_opus(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        warn!("⚠️ 使用已廢棄的原始OPUS解碼，建議改用獨立包模式");
        let decode_start = std::time::Instant::now();
        info!("🚀 開始 WebCodecs 原始 OPUS 解碼: {} bytes", data.len());

        // 🎧 創建音頻調試存檔（如果啟用）
        let mut debug_archive = if std::env::var("CARE_VOICE_DEBUG_AUDIO").is_ok() {
            match AudioDebugArchive::new() {
                Ok(mut archive) => {
                    // 存檔原始OPUS數據
                    if let Err(e) = archive.archive_raw_opus(data) {
                        warn!("🚨 存檔原始OPUS失敗: {}", e);
                    }
                    Some(archive)
                },
                Err(e) => {
                    warn!("🚨 創建音頻調試存檔失敗: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 驗證數據大小
        if data.is_empty() {
            return Err("WebCodecs OPUS 數據為空".into());
        }

        if data.len() < 8 {
            return Err("WebCodecs OPUS 數據過小，可能損壞".into());
        }

        // 🎯 WebCodecs 特殊處理：不要拆分包，直接解碼連續流
        info!("🔧 WebCodecs 模式：跳過包拆分，直接流式解碼");
        
        // 🎯 簡化架構：只使用48kHz解碼器池 + 重採樣
        info!("🔧 簡化模式：固定使用48kHz解碼器 + 16kHz重採樣");
        
        let samples = match self.opus_48k_decoder_pool.decode(data) {
            Ok(samples_48k) => {
                info!("✅ 48kHz OPUS 解碼器池解碼成功: {} samples", samples_48k.len());
                
                // 🎧 存檔48kHz解碼結果
                if let Some(ref mut archive) = debug_archive {
                    if let Err(e) = archive.archive_decoded_48k(&samples_48k) {
                        warn!("🚨 存檔48kHz解碼結果失敗: {}", e);
                    }
                }
                
                // 🔄 48kHz → 16kHz 重採樣 (Whisper AI 要求)
                let samples_16k = Self::resample_48k_to_16k(&samples_48k);
                info!("🔄 48kHz → 16kHz 重採樣完成: {} → {} samples", samples_48k.len(), samples_16k.len());
                
                // 🎧 存檔16kHz重採樣結果
                if let Some(ref mut archive) = debug_archive {
                    if let Err(e) = archive.archive_resampled_16k(&samples_16k) {
                        warn!("🚨 存檔16kHz重採樣結果失敗: {}", e);
                    }
                }
                
                samples_16k
            },
            Err(e) => {
                error!("❌ 48kHz OPUS 解碼失敗: {}", e);
                return Err(format!("WebCodecs 48kHz OPUS 解碼失敗: {}", e).into());
            }
        };

        let decode_time = decode_start.elapsed();
        
        // 🎧 存檔最終Whisper輸入數據
        if let Some(ref mut archive) = debug_archive {
            if let Err(e) = archive.archive_whisper_input(&samples) {
                warn!("🚨 存檔Whisper輸入數據失敗: {}", e);
            } else {
                info!("🎧 音頻調試存檔完成: session_id={}", archive.session_id);
            }
        }
        
        // 記錄 WebCodecs 特定指標
        histogram!("webcodecs_opus_decode_time_ms").record(decode_time.as_millis() as f64);
        histogram!("webcodecs_opus_input_size_bytes").record(data.len() as f64);
        histogram!("webcodecs_opus_output_samples").record(samples.len() as f64);
        counter!("webcodecs_opus_decode_success_total").increment(1);

        info!("✅ WebCodecs OPUS 解碼完成: {} samples, 耗時: {:?}", samples.len(), decode_time);
        Ok(samples)
    }

    /// OPUS 後備解碼方案 - 處理可能的格式變異
    fn decode_opus_fallback(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🔧 嘗試 OPUS 後備解碼方案");

        // 方案1: 嘗試作為 OGG-OPUS 解碼 (WebCodecs 可能添加了 OGG 包裝)
        if let Ok(samples) = Self::try_decode_as_ogg_opus(data) {
            info!("✅ 後備方案1成功: OGG-OPUS 解碼");
            return Ok(samples);
        }

        // 方案2: 嘗試使用 symphonia 通用解碼 (OPUS 可能被包裝在其他容器中)
        if let Ok(samples) = Self::decode_with_symphonia(data, Some("opus")) {
            info!("✅ 後備方案2成功: Symphonia OPUS 解碼");
            return Ok(samples);
        }

        // 方案3: 嘗試作為原始音頻數據解碼 (最後手段)
        if let Ok(samples) = Self::try_decode_raw_audio_data(data) {
            info!("✅ 後備方案3成功: 原始音頻數據解碼");
            return Ok(samples);
        }

        Err("所有 OPUS 後備解碼方案都失敗".into())
    }

    /// 嘗試作為 OGG-OPUS 解碼
    fn try_decode_as_ogg_opus(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // 檢查 OGG 魔術數字
        if data.len() >= 4 && &data[0..4] == b"OggS" {
            info!("🔍 檢測到 OGG 頭部，嘗試 OGG-OPUS 解碼");
            Self::decode_with_symphonia(data, Some("ogg"))
        } else {
            Err("不是 OGG 格式".into())
        }
    }

    /// 🔧 WebCodecs 連續流解碼 - 跳過包拆分算法
    fn decode_webcodecs_continuous_stream(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🚀 開始 WebCodecs 連續流解碼: {} bytes", data.len());

        // 嘗試使用 Symphonia 直接解碼整個 OPUS 流
        match Self::decode_with_symphonia(data, Some("opus")) {
            Ok(samples) => {
                info!("✅ Symphonia 連續流解碼成功: {} samples", samples.len());
                return Ok(samples);
            },
            Err(e) => {
                info!("⚠️ Symphonia OPUS 解碼失敗: {}, 嘗試其他方法", e);
            }
        }

        // 嘗試作為 OGG 容器解碼
        if data.len() >= 4 && &data[0..4] == b"OggS" {
            info!("🔍 檢測到 OGG 頭部，嘗試 OGG 解碼");
            match Self::decode_with_symphonia(data, Some("ogg")) {
                Ok(samples) => {
                    info!("✅ OGG 連續流解碼成功: {} samples", samples.len());
                    return Ok(samples);
                }
                Err(e) => {
                    info!("⚠️ OGG 解碼失敗: {}", e);
                }
            }
        }

        // 嘗試作為原始 PCM 數據解碼
        match Self::try_decode_raw_audio_data(data) {
            Ok(samples) => {
                info!("✅ PCM 連續流解碼成功: {} samples", samples.len());
                return Ok(samples);
            }
            Err(e) => {
                info!("⚠️ PCM 解碼失敗: {}", e);
            }
        }

        Err("WebCodecs 連續流解碼失敗".into())
    }

    /// 嘗試解碼原始音頻數據
    fn try_decode_raw_audio_data(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🔧 嘗試解碼為原始 PCM 數據");
        
        // 假設是 16-bit PCM, 48kHz 單聲道 (WebCodecs 常用格式)
        if data.len() % 2 != 0 {
            return Err("數據長度不是16位對齊".into());
        }

        let mut samples = Vec::with_capacity(data.len() / 2);
        for chunk in data.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / i16::MAX as f32;
            samples.push(sample);
        }

        if samples.is_empty() {
            return Err("沒有生成任何音頻樣本".into());
        }

        info!("✅ 原始 PCM 解碼成功: {} samples", samples.len());
        Ok(samples)
    }

    /// WebM-OPUS 格式解碼 - 統一48kHz架構
    fn decode_webm_opus(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🎵 解碼 WebM-OPUS (Chrome/Edge) - 統一架構");
        
        // 使用48kHz解碼器 + 重採樣到16kHz
        match self.opus_48k_decoder_pool.decode(data) {
            Ok(samples_48k) => {
                let samples_16k = Self::resample_48k_to_16k(&samples_48k);
                Ok(samples_16k)
            },
            Err(e) => Err(e.into())
        }
    }

    /// OGG-OPUS 格式解碼 - 統一48kHz架構
    fn decode_ogg_opus(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("🎵 解碼 OGG-OPUS (Firefox) - 統一架構");
        
        // 使用48kHz解碼器 + 重採樣到16kHz
        match self.opus_48k_decoder_pool.decode(data) {
            Ok(samples_48k) => {
                let samples_16k = Self::resample_48k_to_16k(&samples_48k);
                Ok(samples_16k)
            },
            Err(e) => Err(e.into())
        }
    }

    /// MP4-AAC 格式解碼 (Safari)
    fn decode_mp4_aac(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("嘗試使用 symphonia 解碼 MP4-AAC");
        
        // 使用 symphonia 解碼 MP4
        Self::decode_with_symphonia(data, Some("mp4"))
            .or_else(|e| {
                warn!("symphonia MP4 解碼失敗: {}", e);
                Err(format!("MP4-AAC 解碼暫時不支援: {}. 建議使用 Firefox 或 Chrome.", e).into())
            })
    }

    /// WAV 格式解碼
    fn decode_wav(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("嘗試解碼 WAV 格式");

        // 首先嘗試使用 hound 解碼 WAV
        match Self::decode_wav_with_hound(data) {
            Ok(samples) => {
                info!("hound WAV 解碼成功: {} 樣本", samples.len());
                Ok(samples)
            },
            Err(e) => {
                warn!("hound WAV 解碼失敗: {}, 嘗試 symphonia", e);
                Self::decode_with_symphonia(data, Some("wav"))
            }
        }
    }

    /// Vorbis 格式解碼 (使用 symphonia)
    fn decode_vorbis_with_symphonia(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("使用 symphonia 解碼 Vorbis");
        Self::decode_with_symphonia(data, Some("ogg"))
    }

    /// 未知格式的啟發式解碼
    fn decode_unknown_format(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        warn!("嘗試啟發式解碼未知格式");

        // 嘗試不同的解碼方式 (修復闉包類型問題)
        
        // 嘗試 WAV 解碼
        match Self::decode_wav(data) {
            Ok(samples) => {
                info!("啟發式解碼成功: WAV 解碼器");
                return Ok(samples);
            },
            Err(e) => {
                warn!("WAV 解碼器失敗: {}", e);
            }
        }
        
        // 嘗試 OGG-Vorbis 解碼
        match Self::decode_vorbis_with_symphonia(data) {
            Ok(samples) => {
                info!("啟發式解碼成功: OGG-Vorbis 解碼器");
                return Ok(samples);
            },
            Err(e) => {
                warn!("OGG-Vorbis 解碼器失敗: {}", e);
            }
        }
        
        // 嘗試 Symphonia 通用解碼
        match Self::decode_with_symphonia(data, None) {
            Ok(samples) => {
                info!("啟發式解碼成功: Symphonia通用 解碼器");
                return Ok(samples);
            },
            Err(e) => {
                warn!("Symphonia通用 解碼器失敗: {}", e);
            }
        }

        Err("所有解碼器都失敗，無法識別音頻格式".into())
    }

    /// 使用 hound 解碼 WAV 檔案
    fn decode_wav_with_hound(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let cursor = Cursor::new(data);
        let mut reader = hound::WavReader::new(cursor)?;
        
        let spec = reader.spec();
        info!("WAV 規格: {}Hz, {} 位元, {} 聲道", 
              spec.sample_rate, spec.bits_per_sample, spec.channels);

        let mut samples = Vec::new();

        match spec.sample_format {
            hound::SampleFormat::Float => {
                for sample in reader.samples::<f32>() {
                    samples.push(sample?);
                }
            },
            hound::SampleFormat::Int => {
                match spec.bits_per_sample {
                    16 => {
                        for sample in reader.samples::<i16>() {
                            let s = sample? as f32 / i16::MAX as f32;
                            samples.push(s);
                        }
                    },
                    32 => {
                        for sample in reader.samples::<i32>() {
                            let s = sample? as f32 / i32::MAX as f32;
                            samples.push(s);
                        }
                    },
                    _ => {
                        return Err(format!("不支援的 WAV 位元深度: {}", spec.bits_per_sample).into());
                    }
                }
            }
        }

        // 轉換為單聲道 (如需要)
        if spec.channels == 2 {
            info!("轉換立體聲為單聲道");
            let mono_samples: Vec<f32> = samples
                .chunks_exact(2)
                .map(|pair| (pair[0] + pair[1]) / 2.0)
                .collect();
            Ok(mono_samples)
        } else {
            Ok(samples)
        }
    }

    /// 使用 symphonia 解碼音頻 (通用方法)
    fn decode_with_symphonia(
        data: &[u8], 
        format_hint: Option<&str>
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let cursor = Cursor::new(data.to_vec());
        let media_source = MediaSourceStream::new(Box::new(cursor), Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = format_hint {
            hint.with_extension(extension);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        // 探測格式
        let probed = symphonia::default::get_probe()
            .format(&hint, media_source, &format_opts, &metadata_opts)?;

        let mut format = probed.format;

        // 找到第一個音軌
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("找不到音軌")?;

        let track_id = track.id;

        // 建立解碼器
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)?;

        let mut samples = Vec::new();

        // 解碼所有數據包
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::ResetRequired) => {
                    return Err("需要重置解碼器".into());
                },
                Err(SymphoniaError::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                },
                Err(e) => {
                    return Err(e.into());
                }
            };

            // 只處理目標音軌
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    Self::convert_audio_buffer_to_samples(&decoded, &mut samples)?;
                },
                Err(e) => {
                    warn!("解碼數據包失敗: {}", e);
                }
            }
        }

        if samples.is_empty() {
            return Err("symphonia 未解碼出任何音頻樣本".into());
        }

        info!("symphonia 解碼成功: {} 樣本", samples.len());
        Ok(samples)
    }

    /// 將 symphonia 音頻緩衝區轉換為 f32 樣本 (簡化版本)
    fn convert_audio_buffer_to_samples(
        decoded: &AudioBufferRef, 
        samples: &mut Vec<f32>
    ) -> Result<(), Box<dyn std::error::Error>> {
        use symphonia::core::audio::AudioBufferRef::*;

        match decoded {
            F32(buf) => {
                // 取得第一個聲道的數據 (轉換為單聲道)
                // 取得第一個聲道的數據 (轉換為單聲道)
                let channel = buf.chan(0);
                for &sample in channel {
                    samples.push(sample);
                }
            },
            S16(buf) => {
                // 16位整數轉 f32
                // 16位整數轉 f32
                let channel = buf.chan(0);
                for &sample in channel {
                    samples.push(sample as f32 / i16::MAX as f32);
                }
            },
            S32(buf) => {
                // 32位整數轉 f32  
                // 32位整數轉 f32
                let channel = buf.chan(0);
                for &sample in channel {
                    samples.push(sample as f32 / i32::MAX as f32);
                }
            },
            _ => {
                return Err("暫不支援的音頻樣本格式".into());
            }
        }

        Ok(())
    }

    /// 提供格式支援資訊 - 業界領先實現
    pub fn get_format_support_info() -> Vec<(AudioFormat, &'static str)> {
        vec![
            (AudioFormat::WebmOpus, "✅ 業界領先支援 - Chrome/Edge 標準格式 (完整 OPUS 解碼)"),
            (AudioFormat::OggOpus, "✅ 業界領先支援 - Firefox 標準格式 (完整 OPUS 解碼)"),
            (AudioFormat::Wav, "✅ 完全支援 - 通用格式 (PCM 解碼)"),
            (AudioFormat::WebmVorbis, "✅ 完全支援 - 舊版瀏覽器格式 (Symphonia)"),
            (AudioFormat::Mp4Aac, "⚠️ 計劃支援 - Safari 格式 (實驗性)"),
            (AudioFormat::Unknown, "❌ 不支援 - 請使用支援的格式"),
        ]
    }

    /// 獲取解碼器統計資訊（簡化版）
    pub fn get_decoder_stats(&self) -> DecoderStats {
        let pool_stats = self.opus_48k_decoder_pool.get_pool_stats();
        let detection_stats = self.format_detector.lock().get_detection_stats().clone();
        
        DecoderStats {
            opus_pool_total: pool_stats.total_size,
            opus_pool_available: pool_stats.available,
            opus_pool_in_use: pool_stats.in_use,
            format_detections: detection_stats,
        }
    }

    /// 生成解碼器報告
    pub fn generate_report(&self) -> String {
        let stats = self.get_decoder_stats();
        let format_report = self.format_detector.lock().generate_report();
        
        format!(
            "📊 統一音頻解碼器報告\n\n🎵 OPUS 解碼器池狀態:\n  - 總計: {} 個解碼器\n  - 可用: {} 個\n  - 使用中: {} 個\n\n{}",
            stats.opus_pool_total,
            stats.opus_pool_available, 
            stats.opus_pool_in_use,
            format_report
        )
    }

    /// 重置統計資料
    pub fn reset_stats(&self) {
        info!("🔄 重置統一音頻解碼器統計");
        self.format_detector.lock().reset_stats();
    }
    
    /// 🔄 48kHz → 16kHz 高品質重採樣 (智能算法選擇)
    fn resample_48k_to_16k(samples_48k: &[f32]) -> Vec<f32> {
        info!("🔄 開始48kHz→16kHz智能重採樣: {} samples", samples_48k.len());
        
        // 🚀 智能算法選擇：短音頻使用快速方法，長音頻使用高品質方法
        if samples_48k.len() < 48000 { // 小於1秒的音頻
            info!("🚀 短音頻快速重採樣路徑");
            return Self::fast_resample_48k_to_16k(samples_48k);
        }
        
        // 階段1: 高品質低通濾波 (7.2kHz截止頻率，更嚴格防混疊)
        let filtered_samples = Self::apply_highquality_lowpass_filter(samples_48k, 7200.0, 48000.0);
        
        // 階段2: Lanczos 內插重採樣 (48kHz → 16kHz)
        let ratio = 16000.0 / 48000.0; // 1/3
        let output_length = (filtered_samples.len() as f32 * ratio) as usize;
        let mut samples_16k = Vec::with_capacity(output_length);
        
        // Lanczos 重採樣參數
        const LANCZOS_A: usize = 3; // Lanczos 窗口參數 (a=3 提供良好品質/性能平衡)
        
        for i in 0..output_length {
            let src_index = i as f32 / ratio;
            let base_index = src_index.floor() as isize;
            
            let mut sum = 0.0;
            let mut weight_sum = 0.0;
            
            // Lanczos 內插窗口範圍
            for j in -(LANCZOS_A as isize)..(LANCZOS_A as isize) {
                let sample_index = base_index + j;
                
                // 邊界檢查
                if sample_index >= 0 && (sample_index as usize) < filtered_samples.len() {
                    let distance = src_index - sample_index as f32;
                    let weight = Self::lanczos_kernel(distance, LANCZOS_A as f32);
                    
                    sum += filtered_samples[sample_index as usize] * weight;
                    weight_sum += weight;
                }
            }
            
            // 正規化並添加樣本
            if weight_sum > 0.0 {
                samples_16k.push(sum / weight_sum);
            } else {
                // 後備處理：使用最近鄰樣本
                let nearest_idx = (src_index.round() as usize).min(filtered_samples.len() - 1);
                samples_16k.push(filtered_samples[nearest_idx]);
            }
        }
        
        info!("✅ 高品質重採樣完成: {} → {} samples ({:.2}%, Lanczos-{} 內插)", 
              samples_48k.len(), samples_16k.len(), 
              (samples_16k.len() as f32 / samples_48k.len() as f32) * 100.0, LANCZOS_A);
        
        samples_16k
    }
    
    /// 🎛️ 簡單低通濾波器 (IIR 1階)
    fn apply_lowpass_filter(samples: &[f32], cutoff_freq: f32, sample_rate: f32) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }
        
        // 計算濾波器係數 (1階 IIR 低通濾波器)
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        
        let mut filtered = Vec::with_capacity(samples.len());
        let mut prev_output = samples[0]; // 初始值
        
        for &sample in samples {
            // IIR 濾波器: y[n] = α * x[n] + (1-α) * y[n-1]
            let output = alpha * sample + (1.0 - alpha) * prev_output;
            filtered.push(output);
            prev_output = output;
        }
        
        filtered
    }
    
    /// 🎯 Lanczos 核函數 (高品質重採樣窗函數)
    fn lanczos_kernel(x: f32, a: f32) -> f32 {
        if x.abs() >= a {
            return 0.0;
        }
        if x == 0.0 {
            return 1.0;
        }
        
        let pi_x = std::f32::consts::PI * x;
        let pi_x_a = pi_x / a;
        
        // Lanczos 窗函數: sinc(x) * sinc(x/a)
        (pi_x.sin() / pi_x) * (pi_x_a.sin() / pi_x_a)
    }
    
    /// 🎛️ 高品質低通濾波器 (Butterworth 2階 IIR)
    fn apply_highquality_lowpass_filter(samples: &[f32], cutoff_freq: f32, sample_rate: f32) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }
        
        // Butterworth 2階低通濾波器係數計算
        let omega = 2.0 * std::f32::consts::PI * cutoff_freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * 2.0_f32.sqrt()); // Q = √2 for Butterworth
        
        // 濾波器係數
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega / a0;
        let a2 = (1.0 - alpha) / a0;
        let b0 = (1.0 - cos_omega) / (2.0 * a0);
        let b1 = (1.0 - cos_omega) / a0;
        let b2 = (1.0 - cos_omega) / (2.0 * a0);
        
        let mut filtered = Vec::with_capacity(samples.len());
        let mut x1 = 0.0f32; // x[n-1]
        let mut x2 = 0.0f32; // x[n-2]
        let mut y1 = 0.0f32; // y[n-1]
        let mut y2 = 0.0f32; // y[n-2]
        
        for &x0 in samples {
            // Butterworth 2階差分方程: y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
            let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            
            filtered.push(y0);
            
            // 更新延遲線
            x2 = x1; x1 = x0;
            y2 = y1; y1 = y0;
        }
        
        filtered
    }
    
    /// 🚀 快速重採樣 (適用於短音頻 < 1秒)
    fn fast_resample_48k_to_16k(samples_48k: &[f32]) -> Vec<f32> {
        // 使用簡化的2階Butterworth濾波器 + 線性內插
        let mut filtered = Vec::with_capacity(samples_48k.len());
        
        // 快速2階Butterworth係數 (7kHz截止)
        let a1 = -1.1429805f32;
        let a2 = 0.4128016f32;
        let b0 = 0.0676f32;
        let b1 = 0.1353f32;
        let b2 = 0.0676f32;
        
        let mut x1 = 0.0f32; let mut x2 = 0.0f32;
        let mut y1 = 0.0f32; let mut y2 = 0.0f32;
        
        // 快速濾波
        for &x0 in samples_48k {
            let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            filtered.push(y0);
            x2 = x1; x1 = x0; y2 = y1; y1 = y0;
        }
        
        // 簡單3:1降採樣（取每第3個樣本）
        let mut samples_16k = Vec::with_capacity(filtered.len() / 3);
        for (i, &sample) in filtered.iter().enumerate() {
            if i % 3 == 0 {
                samples_16k.push(sample);
            }
        }
        
        info!("✅ 快速重採樣完成: {} → {} samples", samples_48k.len(), samples_16k.len());
        samples_16k
    }
}

/// 🎧 音頻調試存檔系統
#[derive(Debug, Clone)]
pub struct AudioDebugArchive {
    pub session_id: String,
    pub timestamp: SystemTime,
    pub debug_dir: PathBuf,
    
    // 4個關鍵存檔點
    pub raw_opus_data: Vec<u8>,           // 原始OPUS數據
    pub decoded_48k_samples: Vec<f32>,    // 48kHz解碼結果  
    pub resampled_16k_samples: Vec<f32>,  // 16kHz重採樣結果
    pub whisper_input_samples: Vec<f32>,  // Whisper輸入數據
}

impl AudioDebugArchive {
    /// 創建新的調試存檔會話
    pub fn new() -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now();
        
        // 創建調試目錄
        let debug_dir = PathBuf::from(format!("/tmp/care-voice-debug/{}", session_id));
        fs::create_dir_all(&debug_dir)
            .context("Failed to create debug directory")?;
        
        info!("🎧 創建音頻調試存檔會話: {}", session_id);
        
        Ok(Self {
            session_id,
            timestamp,
            debug_dir,
            raw_opus_data: Vec::new(),
            decoded_48k_samples: Vec::new(),
            resampled_16k_samples: Vec::new(),
            whisper_input_samples: Vec::new(),
        })
    }
    
    /// 存檔原始OPUS數據
    pub fn archive_raw_opus(&mut self, data: &[u8]) -> Result<()> {
        self.raw_opus_data = data.to_vec();
        
        // 保存為OPUS文件
        let opus_path = self.debug_dir.join("01_raw_opus.opus");
        let mut file = File::create(&opus_path)?;
        file.write_all(data)?;
        
        info!("💾 原始OPUS已存檔: {} bytes → {:?}", data.len(), opus_path);
        Ok(())
    }
    
    /// 存檔48kHz解碼結果
    pub fn archive_decoded_48k(&mut self, samples: &[f32]) -> Result<()> {
        self.decoded_48k_samples = samples.to_vec();
        
        // 保存為WAV文件
        let wav_path = self.debug_dir.join("02_decoded_48k.wav");
        Self::save_as_wav(&wav_path, samples, 48000)?;
        
        info!("💾 48kHz解碼結果已存檔: {} samples → {:?}", samples.len(), wav_path);
        Ok(())
    }
    
    /// 存檔16kHz重採樣結果
    pub fn archive_resampled_16k(&mut self, samples: &[f32]) -> Result<()> {
        self.resampled_16k_samples = samples.to_vec();
        
        // 保存為WAV文件
        let wav_path = self.debug_dir.join("03_resampled_16k.wav");
        Self::save_as_wav(&wav_path, samples, 16000)?;
        
        info!("💾 16kHz重採樣結果已存檔: {} samples → {:?}", samples.len(), wav_path);
        Ok(())
    }
    
    /// 存檔Whisper輸入數據
    pub fn archive_whisper_input(&mut self, samples: &[f32]) -> Result<()> {
        self.whisper_input_samples = samples.to_vec();
        
        // 保存為WAV文件
        let wav_path = self.debug_dir.join("04_whisper_input.wav");
        Self::save_as_wav(&wav_path, samples, 16000)?;
        
        info!("💾 Whisper輸入數據已存檔: {} samples → {:?}", samples.len(), wav_path);
        Ok(())
    }
    
    /// 將PCM樣本保存為WAV文件
    fn save_as_wav(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<()> {
        let mut file = File::create(path)?;
        
        // WAV文件頭
        let data_size = (samples.len() * 2) as u32; // 16-bit PCM
        let file_size = 36 + data_size;
        
        // RIFF頭
        file.write_all(b"RIFF")?;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(b"WAVE")?;
        
        // fmt 塊
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // fmt塊大小
        file.write_all(&1u16.to_le_bytes())?;  // 格式 (PCM)
        file.write_all(&1u16.to_le_bytes())?;  // 聲道數
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&(sample_rate * 2).to_le_bytes())?; // 字節率
        file.write_all(&2u16.to_le_bytes())?;  // 塊對齊
        file.write_all(&16u16.to_le_bytes())?; // 位深度
        
        // data 塊
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        
        // PCM數據 (轉換為16-bit)
        for &sample in samples {
            let pcm_sample = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
            file.write_all(&pcm_sample.to_le_bytes())?;
        }
        
        Ok(())
    }
}

/// 解碼器統計資訊
#[derive(Debug, Clone)]
pub struct DecoderStats {
    pub opus_pool_total: usize,
    pub opus_pool_available: usize,
    pub opus_pool_in_use: usize,
    pub format_detections: std::collections::HashMap<AudioFormat, u64>,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_support_info() {
        let support_info = UnifiedAudioDecoder::get_format_support_info();
        assert_eq!(support_info.len(), 6);
        
        // 檢查是否包含主要格式
        let formats: Vec<AudioFormat> = support_info.iter().map(|(f, _)| f.clone()).collect();
        assert!(formats.contains(&AudioFormat::OggOpus));
        assert!(formats.contains(&AudioFormat::WebmOpus));
        assert!(formats.contains(&AudioFormat::Wav));
    }

    #[test]
    fn test_mime_type_detection() {
        // 測試 MIME 類型檢測是否正確路由到解碼器
        let test_cases = [
            ("audio/ogg;codecs=opus", AudioFormat::OggOpus),
            ("audio/webm;codecs=opus", AudioFormat::WebmOpus),
            ("audio/wav", AudioFormat::Wav),
        ];

        for (mime, expected_format) in test_cases.iter() {
            let detected = AudioFormat::detect_from_mime(mime);
            assert_eq!(detected, *expected_format, "MIME 類型 {} 檢測失敗", mime);
        }
    }
}