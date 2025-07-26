// 音頻格式檢測模組
// 支援 WebM, OGG, MP4, WAV 格式的自動檢測

use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    WebmOpus,   // Chrome/Edge: audio/webm;codecs=opus
    OggOpus,    // Firefox: audio/ogg;codecs=opus
    Mp4Aac,     // Safari: audio/mp4;codecs=mp4a.40.2
    Wav,        // 通用格式: audio/wav
    WebmVorbis, // 舊版 Firefox: audio/webm;codecs=vorbis
    Unknown,    // 無法識別的格式
}

impl AudioFormat {
    /// 基於二進制數據的魔術數字檢測音頻格式
    pub fn detect_from_data(data: &[u8]) -> Self {
        if data.len() < 16 {
            warn!("音頻數據太小，無法檢測格式: {} bytes", data.len());
            return AudioFormat::Unknown;
        }

        // WebM 魔術數字檢測 (EBML 頭)
        if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            info!("檢測到 WebM 容器格式");
            
            // 進一步檢測編碼器類型
            // 在 WebM 文件中尋找編碼器信息
            if Self::contains_opus_codec_info(data) {
                info!("WebM 容器包含 Opus 編碼");
                return AudioFormat::WebmOpus;
            } else if Self::contains_vorbis_codec_info(data) {
                info!("WebM 容器包含 Vorbis 編碼");
                return AudioFormat::WebmVorbis;
            }
            
            // 預設假設為 Opus (2025年標準)
            info!("WebM 格式未能確定編碼器，預設為 Opus");
            return AudioFormat::WebmOpus;
        }

        // OGG 魔術數字檢測
        if data.starts_with(b"OggS") {
            info!("檢測到 OGG 容器格式");
            
            // OGG 文件通常在前 100 bytes 內包含編碼器信息
            if data.len() >= 100 {
                let header_section = &data[0..100];
                if header_section.windows(4).any(|window| window == b"Opus") {
                    info!("OGG 容器包含 Opus 編碼");
                    return AudioFormat::OggOpus;
                } else if header_section.windows(6).any(|window| window == b"vorbis") {
                    info!("OGG 容器包含 Vorbis 編碼");
                    return AudioFormat::WebmVorbis; // 統一處理 Vorbis
                }
            }
            
            // 預設假設為 Opus (2025年標準)
            info!("OGG 格式未能確定編碼器，預設為 Opus");
            return AudioFormat::OggOpus;
        }

        // MP4 魔術數字檢測 (ftyp box)
        if data.len() >= 8 && &data[4..8] == b"ftyp" {
            info!("檢測到 MP4 容器格式");
            
            // 檢查 MP4 品牌
            if data.len() >= 12 {
                let brand = &data[8..12];
                if brand == b"M4A " || brand == b"mp41" || brand == b"mp42" {
                    info!("MP4 格式，假設為 AAC 編碼");
                    return AudioFormat::Mp4Aac;
                }
            }
            
            return AudioFormat::Mp4Aac;
        }

        // WAV 魔術數字檢測
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WAVE" {
            info!("檢測到 WAV 格式");
            return AudioFormat::Wav;
        }

        warn!("無法識別音頻格式，數據開頭: {:?}", &data[0..std::cmp::min(16, data.len())]);
        AudioFormat::Unknown
    }

    /// 基於 MIME 類型檢測音頻格式
    pub fn detect_from_mime(mime_type: &str) -> Self {
        let mime_lower = mime_type.to_lowercase();
        
        match mime_lower.as_str() {
            // WebM Opus 格式 (Chrome/Edge 標準)
            "audio/webm" | "audio/webm;codecs=opus" => {
                info!("MIME 檢測: WebM Opus 格式");
                AudioFormat::WebmOpus
            },
            
            // WebM Vorbis 格式 (舊版 Firefox)
            "audio/webm;codecs=vorbis" => {
                info!("MIME 檢測: WebM Vorbis 格式");
                AudioFormat::WebmVorbis
            },
            
            // OGG Opus 格式 (Firefox 標準)
            "audio/ogg" | "audio/ogg;codecs=opus" => {
                info!("MIME 檢測: OGG Opus 格式");
                AudioFormat::OggOpus
            },
            
            // OGG Vorbis 格式 (舊版)
            "audio/ogg;codecs=vorbis" => {
                info!("MIME 檢測: OGG Vorbis 格式");
                AudioFormat::WebmVorbis
            },
            
            // MP4 AAC 格式 (Safari)
            "audio/mp4" | "audio/mp4;codecs=mp4a.40.2" | "audio/m4a" => {
                info!("MIME 檢測: MP4 AAC 格式");
                AudioFormat::Mp4Aac
            },
            
            // WAV 格式
            "audio/wav" | "audio/wave" => {
                info!("MIME 檢測: WAV 格式");
                AudioFormat::Wav
            },
            
            _ => {
                warn!("未知 MIME 類型: {}", mime_type);
                AudioFormat::Unknown
            }
        }
    }

    /// 檢測數據中是否包含 Opus 編碼器信息
    fn contains_opus_codec_info(data: &[u8]) -> bool {
        // 在前 1KB 數據中尋找 "Opus" 字符串
        let search_len = std::cmp::min(1024, data.len());
        let search_data = &data[0..search_len];
        
        search_data.windows(4).any(|window| window == b"Opus")
    }

    /// 檢測數據中是否包含 Vorbis 編碼器信息  
    fn contains_vorbis_codec_info(data: &[u8]) -> bool {
        // 在前 1KB 數據中尋找 "vorbis" 字符串
        let search_len = std::cmp::min(1024, data.len());
        let search_data = &data[0..search_len];
        
        search_data.windows(6).any(|window| window == b"vorbis")
    }

    /// 取得格式的友善名稱
    pub fn friendly_name(&self) -> &'static str {
        match self {
            AudioFormat::WebmOpus => "WebM (Opus)",
            AudioFormat::OggOpus => "OGG (Opus)",
            AudioFormat::Mp4Aac => "MP4 (AAC)",
            AudioFormat::Wav => "WAV (PCM)",
            AudioFormat::WebmVorbis => "WebM/OGG (Vorbis)",
            AudioFormat::Unknown => "未知格式",
        }
    }

    /// 取得格式支援狀況
    pub fn support_status(&self) -> &'static str {
        match self {
            AudioFormat::WebmOpus => "✅ 已支援 (Chrome/Edge)",
            AudioFormat::OggOpus => "✅ 已支援 (Firefox)",
            AudioFormat::Mp4Aac => "⚠️ 計劃支援 (Safari)",
            AudioFormat::Wav => "✅ 已支援 (通用)",
            AudioFormat::WebmVorbis => "✅ 已支援 (舊版瀏覽器)",
            AudioFormat::Unknown => "❌ 不支援",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webm_detection() {
        let webm_header = [0x1A, 0x45, 0xDF, 0xA3, 0x00, 0x00, 0x00, 0x20];
        assert_eq!(AudioFormat::detect_from_data(&webm_header), AudioFormat::WebmOpus);
    }

    #[test]
    fn test_ogg_detection() {
        let ogg_header = b"OggS\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(AudioFormat::detect_from_data(ogg_header), AudioFormat::OggOpus);
    }

    #[test]
    fn test_wav_detection() {
        let wav_header = b"RIFF\x24\x08\x00\x00WAVE";
        assert_eq!(AudioFormat::detect_from_data(wav_header), AudioFormat::Wav);
    }

    #[test]
    fn test_mp4_detection() {
        let mp4_header = b"\x00\x00\x00\x20ftypM4A ";
        assert_eq!(AudioFormat::detect_from_data(mp4_header), AudioFormat::Mp4Aac);
    }

    #[test]
    fn test_mime_detection() {
        assert_eq!(AudioFormat::detect_from_mime("audio/webm;codecs=opus"), AudioFormat::WebmOpus);
        assert_eq!(AudioFormat::detect_from_mime("audio/ogg;codecs=opus"), AudioFormat::OggOpus);
        assert_eq!(AudioFormat::detect_from_mime("audio/mp4"), AudioFormat::Mp4Aac);
        assert_eq!(AudioFormat::detect_from_mime("audio/wav"), AudioFormat::Wav);
    }

    #[test]
    fn test_unknown_format() {
        let unknown_data = b"UNKNOWN_FORMAT";
        assert_eq!(AudioFormat::detect_from_data(unknown_data), AudioFormat::Unknown);
        assert_eq!(AudioFormat::detect_from_mime("audio/unknown"), AudioFormat::Unknown);
    }
}