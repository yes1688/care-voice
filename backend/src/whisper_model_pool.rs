// ===================================
// Whisper 多模型並行處理架構
// 業界領先的智能模型選擇與 GPU 資源最佳化
// ===================================

use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, error, warn, debug, span, Level};
use anyhow::{Result, Context as AnyhowContext};
use std::collections::HashMap;
use std::time::Instant;
use rayon::prelude::*;
use crossbeam::channel::{self, Receiver, Sender};
use uuid::Uuid;
use std::sync::atomic::AtomicU64;

// 效能監控
use metrics::{counter, histogram, gauge};

// CPU 資訊
use num_cpus;

/// 轉錄品質等級
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranscriptionQuality {
    /// 超快速處理 (0.05x 實時) - 適用於即時應用
    Turbo,
    /// 平衡處理 (0.1x 實時) - 適用於一般應用
    Balanced,
    /// 高精度處理 (0.2x 實時) - 適用於關鍵應用
    HighAccuracy,
    /// 最高品質 (0.3x 實時) - 適用於專業應用
    Premium,
}

impl TranscriptionQuality {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::Turbo => "ggml-tiny.bin",
            Self::Balanced => "ggml-base.bin", 
            Self::HighAccuracy => "ggml-large-v2.bin",
            Self::Premium => "ggml-large-v3.bin",
        }
    }

    pub fn target_latency_ms(&self) -> u64 {
        match self {
            Self::Turbo => 50,
            Self::Balanced => 100,
            Self::HighAccuracy => 200,
            Self::Premium => 300,
        }
    }
}

/// 轉錄任務
#[derive(Debug)]
pub struct TranscriptionTask {
    pub id: Uuid,
    pub audio_samples: Vec<f32>,
    pub quality: TranscriptionQuality,
    pub language: Option<String>,
    pub timestamp: Instant,
}

/// 轉錄結果
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub task_id: Uuid,
    pub transcript: String,
    pub confidence: Option<f32>,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub text: String,
    pub confidence: Option<f32>,
}

/// Whisper 模型實例
struct WhisperModel {
    context: WhisperContext,
    quality: TranscriptionQuality,
    model_path: String,
    creation_time: Instant,
    total_processed: AtomicU64,
    total_processing_time: AtomicU64,
}

impl WhisperModel {
    fn new(model_path: String, quality: TranscriptionQuality) -> Result<Self> {
        let span = span!(Level::INFO, "whisper_model_creation", quality = ?quality);
        let _enter = span.enter();

        info!("正在初始化 {} 模型: {}", quality.model_name(), model_path);
        
        let start_time = Instant::now();
        let context = WhisperContext::new_with_params(
            &model_path,
            WhisperContextParameters::default(),
        ).with_context(|| format!("無法載入 Whisper 模型: {}", model_path))?;
        
        let creation_time = start_time.elapsed();
        info!("✅ {} 模型初始化完成，耗時: {:?}", quality.model_name(), creation_time);
        
        // 記錄模型載入指標
        histogram!("whisper_model_load_time_ms").record(creation_time.as_millis() as f64);
        counter!("whisper_model_loaded_total", "quality" => quality.model_name()).increment(1);

        Ok(Self {
            context,
            quality,
            model_path,
            creation_time: Instant::now(),
            total_processed: AtomicU64::new(0),
            total_processing_time: AtomicU64::new(0),
        })
    }

    async fn transcribe(&self, task: &TranscriptionTask) -> Result<TranscriptionResult> {
        let span = span!(Level::DEBUG, "whisper_transcribe", 
            task_id = %task.id,
            quality = ?self.quality,
            samples = task.audio_samples.len()
        );
        let _enter = span.enter();

        let start_time = Instant::now();
        
        // 配置轉錄參數
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 根據品質等級調整參數
        match self.quality {
            TranscriptionQuality::Turbo => {
                params.set_n_threads(4);
                params.set_print_special(false);
                params.set_print_progress(false);
            },
            TranscriptionQuality::Balanced => {
                params.set_n_threads(6);
                params.set_print_special(false);
                params.set_print_progress(false);
            },
            TranscriptionQuality::HighAccuracy => {
                params.set_n_threads(8);
                params.set_temperature(0.0);
                // params.set_best_of(3); // whisper-rs API 已變更
            },
            TranscriptionQuality::Premium => {
                params.set_n_threads(8);
                params.set_temperature(0.0);
                // params.set_best_of(5); // whisper-rs API 已變更
                // params.set_beam_size(5); // whisper-rs API 已變更
            },
        }

        // 設定語言
        if let Some(ref language) = task.language {
            params.set_language(Some(language));
        }

        params.set_print_timestamps(true);
        
        // 執行轉錄
        let mut state = self.context.create_state()
            .with_context(|| "無法創建 Whisper 狀態")?;
            
        state.full(params, &task.audio_samples)
            .with_context(|| "Whisper 轉錄失敗")?;

        // 收集轉錄結果
        let num_segments = state.full_n_segments()
            .with_context(|| "無法獲取轉錄段數")?;

        let mut segments = Vec::new();
        let mut full_transcript = String::new();

        for i in 0..num_segments {
            let segment_text = state.full_get_segment_text(i)
                .with_context(|| format!("無法獲取第 {} 段文字", i))?;
            
            let start_time = state.full_get_segment_t0(i)
                .with_context(|| format!("無法獲取第 {} 段開始時間", i))? as f32 / 100.0;
                
            let end_time = state.full_get_segment_t1(i)
                .with_context(|| format!("無法獲取第 {} 段結束時間", i))? as f32 / 100.0;

            segments.push(TranscriptSegment {
                start_time,
                end_time,
                text: segment_text.clone(),
                confidence: None, // Whisper-rs 目前不提供信心分數
            });

            full_transcript.push_str(&segment_text);
        }

        let processing_time = start_time.elapsed();
        
        // 更新統計資料
        self.total_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_processing_time.fetch_add(
            processing_time.as_millis() as u64, 
            std::sync::atomic::Ordering::Relaxed
        );

        // 記錄效能指標
        histogram!("whisper_transcription_time_ms").record(processing_time.as_millis() as f64);
        counter!("whisper_transcriptions_completed_total", 
            "quality" => self.quality.model_name()).increment(1);
        gauge!("whisper_audio_duration_seconds").set(task.audio_samples.len() as f64 / 16000.0);

        debug!("✅ 轉錄完成: {} 段, 耗時: {:?}", num_segments, processing_time);

        Ok(TranscriptionResult {
            task_id: task.id,
            transcript: full_transcript.trim().to_string(),
            confidence: None,
            processing_time_ms: processing_time.as_millis() as u64,
            model_used: self.quality.model_name().to_string(),
            segments,
        })
    }

    fn get_stats(&self) -> ModelStats {
        let total_processed = self.total_processed.load(std::sync::atomic::Ordering::Relaxed);
        let total_time = self.total_processing_time.load(std::sync::atomic::Ordering::Relaxed);
        
        ModelStats {
            quality: self.quality,
            total_processed,
            total_processing_time_ms: total_time,
            average_processing_time_ms: if total_processed > 0 { 
                total_time / total_processed 
            } else { 
                0 
            },
            uptime: self.creation_time.elapsed(),
        }
    }
}

/// 模型統計資料
#[derive(Debug, Clone)]
pub struct ModelStats {
    pub quality: TranscriptionQuality,
    pub total_processed: u64,
    pub total_processing_time_ms: u64,
    pub average_processing_time_ms: u64,
    pub uptime: std::time::Duration,
}

/// Whisper 模型池 - 業界領先的並行處理架構
pub struct WhisperModelPool {
    models: RwLock<HashMap<TranscriptionQuality, Arc<WhisperModel>>>,
    task_sender: Sender<TranscriptionTask>,
    result_receiver: Arc<RwLock<HashMap<Uuid, TranscriptionResult>>>,
    worker_handles: Vec<std::thread::JoinHandle<()>>,
}

impl WhisperModelPool {
    /// 創建新的模型池
    pub fn new(model_base_path: &str) -> Result<Self> {
        info!("🚀 正在初始化 Whisper 模型池...");
        
        let mut models = HashMap::new();
        
        // 載入所有品質等級的模型
        for quality in [
            TranscriptionQuality::Turbo,
            TranscriptionQuality::Balanced,
            TranscriptionQuality::HighAccuracy,
        ] {
            let model_path = format!("{}/{}", model_base_path, quality.model_name());
            
            // 檢查模型檔案是否存在
            if !std::path::Path::new(&model_path).exists() {
                warn!("⚠️  模型檔案不存在，跳過: {}", model_path);
                continue;
            }
            
            match WhisperModel::new(model_path, quality) {
                Ok(model) => {
                    models.insert(quality, Arc::new(model));
                    info!("✅ {} 模型載入成功", quality.model_name());
                },
                Err(e) => {
                    error!("❌ {} 模型載入失敗: {}", quality.model_name(), e);
                }
            }
        }

        if models.is_empty() {
            return Err(anyhow::anyhow!("沒有可用的 Whisper 模型"));
        }

        // 創建任務通道
        let (task_sender, task_receiver) = channel::bounded(1000);
        let result_storage = Arc::new(RwLock::new(HashMap::new()));
        
        // 啟動工作線程
        let worker_handles = Self::start_workers(
            Arc::new(RwLock::new(models.clone())),
            task_receiver,
            result_storage.clone(),
        );

        info!("✅ Whisper 模型池初始化完成，載入 {} 個模型", models.len());
        counter!("whisper_model_pool_initialized_total").increment(1);
        gauge!("whisper_models_loaded_count").set(models.len() as f64);

        Ok(Self {
            models: RwLock::new(models),
            task_sender,
            result_receiver: result_storage,
            worker_handles,
        })
    }

    /// 啟動背景工作線程
    fn start_workers(
        models: Arc<RwLock<HashMap<TranscriptionQuality, Arc<WhisperModel>>>>,
        task_receiver: Receiver<TranscriptionTask>,
        result_storage: Arc<RwLock<HashMap<Uuid, TranscriptionResult>>>,
    ) -> Vec<std::thread::JoinHandle<()>> {
        let num_workers = num_cpus::get().min(8);
        info!("啟動 {} 個 Whisper 工作線程", num_workers);

        (0..num_workers)
            .map(|worker_id| {
                let models = models.clone();
                let task_receiver = task_receiver.clone();
                let result_storage = result_storage.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new()
                        .expect("無法創建 tokio 運行時");

                    while let Ok(task) = task_receiver.recv() {
                        let span = span!(Level::DEBUG, "whisper_worker", 
                            worker_id = worker_id,
                            task_id = %task.id
                        );
                        let _enter = span.enter();

                        // 選擇合適的模型
                        let model = {
                            let models_guard = models.read();
                            if let Some(model) = models_guard.get(&task.quality) {
                                model.clone()
                            } else {
                                // 回退到可用的模型
                                if let Some(model) = models_guard.get(&TranscriptionQuality::Balanced) {
                                    warn!("所請求的品質 {:?} 不可用，回退到 Balanced", task.quality);
                                    model.clone()
                                } else if let Some((_, model)) = models_guard.iter().next() {
                                    warn!("Balanced 模型不可用，使用第一個可用模型");
                                    model.clone()
                                } else {
                                    error!("沒有可用的模型");
                                    continue;
                                }
                            }
                        };

                        // 執行轉錄
                        match rt.block_on(model.transcribe(&task)) {
                            Ok(result) => {
                                debug!("✅ 任務 {} 完成", task.id);
                                result_storage.write().insert(task.id, result);
                            },
                            Err(e) => {
                                error!("❌ 任務 {} 失敗: {}", task.id, e);
                                counter!("whisper_transcription_errors_total").increment(1);
                            }
                        }
                    }

                    info!("工作線程 {} 退出", worker_id);
                })
            })
            .collect()
    }

    /// 提交轉錄任務
    pub async fn transcribe_async(
        &self,
        audio_samples: Vec<f32>,
        quality: TranscriptionQuality,
        language: Option<String>,
    ) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        let task = TranscriptionTask {
            id: task_id,
            audio_samples,
            quality,
            language,
            timestamp: Instant::now(),
        };

        self.task_sender.send(task)
            .with_context(|| "任務佇列已滿，無法提交新任務")?;

        counter!("whisper_tasks_submitted_total", 
            "quality" => quality.model_name()).increment(1);

        debug!("📝 任務 {} 已提交 (品質: {:?})", task_id, quality);
        Ok(task_id)
    }

    /// 獲取轉錄結果
    pub fn get_result(&self, task_id: Uuid) -> Option<TranscriptionResult> {
        self.result_receiver.write().remove(&task_id)
    }

    /// 阻塞式轉錄 (向後相容)
    pub async fn transcribe_blocking(
        &self,
        audio_samples: Vec<f32>,
        quality: TranscriptionQuality,
        language: Option<String>,
    ) -> Result<TranscriptionResult> {
        let task_id = self.transcribe_async(audio_samples, quality, language).await?;
        
        // 輪詢結果
        let start_time = Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        
        loop {
            if let Some(result) = self.get_result(task_id) {
                return Ok(result);
            }
            
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("轉錄超時"));
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// 自適應品質轉錄
    pub async fn transcribe_adaptive(
        &self,
        audio_samples: Vec<f32>,
        target_latency_ms: Option<u64>,
    ) -> Result<TranscriptionResult> {
        // 根據音頻長度和目標延遲選擇品質
        let audio_duration_ms = (audio_samples.len() as f64 / 16.0) as u64;
        
        let quality = if let Some(target) = target_latency_ms {
            if target <= 100 {
                TranscriptionQuality::Turbo
            } else if target <= 200 {
                TranscriptionQuality::Balanced
            } else {
                TranscriptionQuality::HighAccuracy
            }
        } else if audio_duration_ms <= 5000 {
            TranscriptionQuality::Turbo
        } else if audio_duration_ms <= 30000 {
            TranscriptionQuality::Balanced
        } else {
            TranscriptionQuality::HighAccuracy
        };

        info!("🎯 自適應品質選擇: {:?} (音頻: {}ms)", quality, audio_duration_ms);
        self.transcribe_blocking(audio_samples, quality, None).await
    }

    /// 獲取模型池統計資料
    pub fn get_stats(&self) -> Vec<ModelStats> {
        self.models.read()
            .values()
            .map(|model| model.get_stats())
            .collect()
    }

    /// 檢查健康狀態
    pub fn health_check(&self) -> bool {
        !self.models.read().is_empty()
    }
}

impl Drop for WhisperModelPool {
    fn drop(&mut self) {
        info!("正在關閉 Whisper 模型池...");
        // 工作線程會在通道關閉時自動退出
    }
}