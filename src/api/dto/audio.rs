use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::entities::transcription::{
    ResponseFormat, TimestampGranularity, TranscriptionResponse, TranscriptionSegment,
    TranscriptionUsage, TranscriptionWord,
};

/// Audio transcription request DTO
#[derive(Debug, Deserialize)]
pub struct TranscribeRequestDto {
    pub model: Option<String>,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub response_format: Option<ResponseFormatDto>,
    pub temperature: Option<f32>,
    pub timestamp_granularities: Option<String>,
    pub llm_api_key_id: Option<String>,
}

impl TranscribeRequestDto {
    pub fn parse_timestamp_granularities(&self) -> Option<Vec<TimestampGranularity>> {
        self.timestamp_granularities.as_ref().map(|s| {
            s.split(',')
                .filter_map(|g| match g.trim() {
                    "word" => Some(TimestampGranularity::Word),
                    "segment" => Some(TimestampGranularity::Segment),
                    _ => None,
                })
                .collect()
        })
    }
}

/// Response format DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatDto {
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

impl From<ResponseFormatDto> for ResponseFormat {
    fn from(dto: ResponseFormatDto) -> Self {
        match dto {
            ResponseFormatDto::Json => ResponseFormat::Json,
            ResponseFormatDto::Text => ResponseFormat::Text,
            ResponseFormatDto::Srt => ResponseFormat::Srt,
            ResponseFormatDto::VerboseJson => ResponseFormat::VerboseJson,
            ResponseFormatDto::Vtt => ResponseFormat::Vtt,
        }
    }
}

/// Timestamp granularity DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimestampGranularityDto {
    Word,
    Segment,
}

/// Transcription response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranscribeResponseDto {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub segments: Option<Vec<TranscriptionSegmentDto>>,
    pub words: Option<Vec<TranscriptionWordDto>>,
    pub usage: Option<TranscriptionUsageDto>,
}

impl From<TranscriptionResponse> for TranscribeResponseDto {
    fn from(response: TranscriptionResponse) -> Self {
        Self {
            text: response.text,
            language: response.language,
            duration: response.duration,
            segments: response.segments.map(|segs| {
                segs.into_iter().map(TranscriptionSegmentDto::from).collect()
            }),
            words: response.words.map(|words| {
                words.into_iter().map(TranscriptionWordDto::from).collect()
            }),
            usage: response.usage.map(TranscriptionUsageDto::from),
        }
    }
}

/// Transcription segment DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranscriptionSegmentDto {
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

impl From<TranscriptionSegment> for TranscriptionSegmentDto {
    fn from(segment: TranscriptionSegment) -> Self {
        Self {
            id: segment.id,
            start: segment.start,
            end: segment.end,
            text: segment.text,
            tokens: segment.tokens,
            temperature: segment.temperature,
            avg_logprob: segment.avg_logprob,
            compression_ratio: segment.compression_ratio,
            no_speech_prob: segment.no_speech_prob,
        }
    }
}

/// Transcription word DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranscriptionWordDto {
    pub word: String,
    pub start: f32,
    pub end: f32,
}

impl From<TranscriptionWord> for TranscriptionWordDto {
    fn from(word: TranscriptionWord) -> Self {
        Self {
            word: word.word,
            start: word.start,
            end: word.end,
        }
    }
}

/// Usage information DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranscriptionUsageDto {
    pub audio_duration_seconds: f32,
    pub tokens_used: Option<i32>,
    pub estimated_cost_usd: Option<f64>,
}

impl From<TranscriptionUsage> for TranscriptionUsageDto {
    fn from(usage: TranscriptionUsage) -> Self {
        Self {
            audio_duration_seconds: usage.audio_duration_seconds,
            tokens_used: usage.tokens_used,
            estimated_cost_usd: usage.estimated_cost_usd,
        }
    }
}