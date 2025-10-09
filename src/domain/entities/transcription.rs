use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::LlmProvider;

/// Transcription request entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    pub file_data: Vec<u8>,
    pub file_name: String,
    pub model: Option<String>,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub temperature: Option<f32>,
    pub timestamp_granularities: Option<Vec<TimestampGranularity>>,
    pub llm_api_key_id: Option<String>,
}

/// Response format enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

/// Timestamp granularity enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampGranularity {
    Word,
    Segment,
}

/// Transcription response entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub segments: Option<Vec<TranscriptionSegment>>,
    pub words: Option<Vec<TranscriptionWord>>,
    pub usage: Option<TranscriptionUsage>,
}

/// Transcription segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: i32,
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub tokens: Option<Vec<i32>>,
    pub temperature: Option<f32>,
    pub avg_logprob: Option<f32>,
    pub compression_ratio: Option<f32>,
    pub no_speech_prob: Option<f32>,
}

/// Transcription word with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub word: String,
    pub start: f32,
    pub end: f32,
}

/// Usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionUsage {
    pub audio_duration_seconds: f32,
    pub tokens_used: Option<i32>,
    pub estimated_cost_usd: Option<f64>,
}

/// Transcription history entity for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionHistory {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub transcription_id: String,
    #[serde(with = "crate::shared::utils::string_or_objectid")]
    pub project_id: String,  // Deserializes ObjectId from MongoDB to String
    pub provider: LlmProvider,
    pub file_hash: String,
    pub file_name: String,
    pub file_size_bytes: usize,
    pub duration_seconds: Option<f32>,
    pub model: String,
    pub language: Option<String>,
    pub text: String,
    pub cost_usd: f64,
    pub response_time_ms: u64,
    pub from_cache: bool,
    pub created_at: DateTime<Utc>,
}

impl TranscriptionHistory {
    pub fn new(
        project_id: String,
        provider: LlmProvider,
        file_hash: String,
        file_name: String,
        file_size_bytes: usize,
        duration_seconds: Option<f32>,
        model: String,
        language: Option<String>,
        text: String,
        cost_usd: f64,
        response_time_ms: u64,
        from_cache: bool,
    ) -> Self {
        Self {
            id: None,
            transcription_id: format!("trans_{}", uuid::Uuid::new_v4()),
            project_id,
            provider,
            file_hash,
            file_name,
            file_size_bytes,
            duration_seconds,
            model,
            language,
            text,
            cost_usd,
            response_time_ms,
            from_cache,
            created_at: Utc::now(),
        }
    }
}