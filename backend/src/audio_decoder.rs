// 統一音頻解碼器介面
// 整合 Opus, Vorbis, AAC, WAV 等多種格式的解碼

use crate::audio_format::AudioFormat;
use crate::opus_decoder::{OpusDecoder, OpusDecoderConfig};
use tracing::{info, warn};
use std::io::Cursor;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use hound;

/// 統一音頻解碼器結構
pub struct UnifiedAudioDecoder;

impl UnifiedAudioDecoder {
    /// 主要解碼函數 - 根據格式自動選擇適當的解碼器
    pub fn decode_audio(
        format: AudioFormat, 
        data: &[u8]
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("開始解碼音頻: 格式={:?}, 數據大小={}bytes", format, data.len());

        match format {
            AudioFormat::WebmOpus => {
                info!("使用 WebM-Opus 解碼器");
                Self::decode_webm_opus(data)
            },
            AudioFormat::OggOpus => {
                info!("使用 OGG-Opus 解碼器");
                Self::decode_ogg_opus(data)
            },
            AudioFormat::Mp4Aac => {
                info!("使用 MP4-AAC 解碼器");
                Self::decode_mp4_aac(data)
            },
            AudioFormat::Wav => {
                info!("使用 WAV 解碼器");
                Self::decode_wav(data)
            },
            AudioFormat::WebmVorbis => {
                info!("使用 WebM/OGG-Vorbis 解碼器");
                Self::decode_vorbis_with_symphonia(data)
            },
            AudioFormat::Unknown => {
                warn!("未知格式，嘗試使用啟發式解碼");
                Self::decode_unknown_format(data)
            }
        }
    }

    /// 帶 MIME 類型提示的解碼函數
    pub fn decode_audio_with_mime(
        data: &[u8], 
        mime_type: &str
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("解碼音頻 (MIME: {})", mime_type);

        // 首先使用 MIME 類型檢測
        let format_from_mime = AudioFormat::detect_from_mime(mime_type);
        
        // 如果 MIME 檢測失敗，使用二進制檢測
        let format = if format_from_mime == AudioFormat::Unknown {
            info!("MIME 檢測失敗，使用二進制檢測");
            AudioFormat::detect_from_data(data)
        } else {
            format_from_mime
        };

        info!("檢測到格式: {} (來源: MIME={})", format.friendly_name(), mime_type);
        Self::decode_audio(format, data)
    }

    /// WebM-Opus 格式解碼 (階段性實現)
    fn decode_webm_opus(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let config = OpusDecoderConfig {
            sample_rate: 48000,
            channels: 1,  // Mono
            frame_size: 960,
        };

        let mut opus_decoder = OpusDecoder::new(config)?;
        opus_decoder.decode_webm_opus(data)
    }

    /// OGG-Opus 格式解碼 (階段性實現)
    fn decode_ogg_opus(data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let config = OpusDecoderConfig {
            sample_rate: 48000,
            channels: 1,  // Mono
            frame_size: 960,
        };

        let mut opus_decoder = OpusDecoder::new(config)?;
        opus_decoder.decode_ogg_opus(data)
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
        let cursor = Cursor::new(data);
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

    /// 提供格式支援資訊
    pub fn get_format_support_info() -> Vec<(AudioFormat, &'static str)> {
        vec![
            (AudioFormat::OggOpus, "✅ 完全支援 - Firefox 標準格式"),
            (AudioFormat::WebmOpus, "🔶 基礎支援 - Chrome/Edge 格式 (簡化實作)"),
            (AudioFormat::Wav, "✅ 完全支援 - 通用格式"),
            (AudioFormat::WebmVorbis, "✅ 完全支援 - 舊版瀏覽器格式"),
            (AudioFormat::Mp4Aac, "⚠️ 有限支援 - Safari 格式 (實驗性)"),
            (AudioFormat::Unknown, "❌ 不支援 - 請使用支援的格式"),
        ]
    }
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