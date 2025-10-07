use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

use crate::domain::entities::transcription::{
    ResponseFormat, TimestampGranularity, TranscriptionResponse, TranscriptionSegment,
    TranscriptionUsage, TranscriptionWord,
};
use crate::shared::error::AppError;

/// OpenAI provider service for API interactions
pub struct OpenAIProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Transcribe audio using OpenAI Whisper API
    pub async fn transcribe(
        &self,
        api_key: &str,
        file_data: Vec<u8>,
        file_name: String,
        model: Option<String>,
        language: Option<String>,
        prompt: Option<String>,
        response_format: Option<ResponseFormat>,
        temperature: Option<f32>,
        timestamp_granularities: Option<Vec<TimestampGranularity>>,
    ) -> Result<TranscriptionResponse, AppError> {
        let url = format!("{}/audio/transcriptions", self.base_url);

        // Build multipart form
        let mut form = Form::new()
            .part("file", Part::bytes(file_data).file_name(file_name))
            .text("model", model.unwrap_or_else(|| "whisper-1".to_string()));

        if let Some(lang) = language {
            form = form.text("language", lang);
        }

        if let Some(p) = prompt {
            form = form.text("prompt", p);
        }

        if let Some(format) = response_format {
            let format_str = match format {
                ResponseFormat::Json => "json",
                ResponseFormat::Text => "text",
                ResponseFormat::Srt => "srt",
                ResponseFormat::VerboseJson => "verbose_json",
                ResponseFormat::Vtt => "vtt",
            };
            form = form.text("response_format", format_str);
        }

        if let Some(temp) = temperature {
            form = form.text("temperature", temp.to_string());
        }

        if let Some(granularities) = timestamp_granularities {
            let granularities_str = granularities
                .iter()
                .map(|g| match g {
                    TimestampGranularity::Word => "word",
                    TimestampGranularity::Segment => "segment",
                })
                .collect::<Vec<_>>()
                .join(",");
            form = form.text("timestamp_granularities", granularities_str);
        }

        // Make API request
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ExternalApiError(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        // Parse response
        let openai_response: OpenAITranscriptionResponse = response.json().await?;

        // Convert to our domain model
        Ok(TranscriptionResponse {
            text: openai_response.text,
            language: openai_response.language,
            duration: openai_response.duration,
            segments: openai_response.segments.map(|segs| {
                segs.into_iter()
                    .map(|s| TranscriptionSegment {
                        id: s.id,
                        start: s.start,
                        end: s.end,
                        text: s.text,
                        tokens: s.tokens,
                        temperature: s.temperature,
                        avg_logprob: s.avg_logprob,
                        compression_ratio: s.compression_ratio,
                        no_speech_prob: s.no_speech_prob,
                    })
                    .collect()
            }),
            words: openai_response.words.map(|words| {
                words
                    .into_iter()
                    .map(|w| TranscriptionWord {
                        word: w.word,
                        start: w.start,
                        end: w.end,
                    })
                    .collect()
            }),
            usage: openai_response.duration.map(|dur| TranscriptionUsage {
                audio_duration_seconds: dur,
                tokens_used: None,
                estimated_cost_usd: Some(dur as f64 * 0.006 / 60.0), // $0.006 per minute
            }),
        })
    }
}

// OpenAI API response structures
#[derive(Debug, Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
    language: Option<String>,
    duration: Option<f32>,
    segments: Option<Vec<OpenAISegment>>,
    words: Option<Vec<OpenAIWord>>,
}

#[derive(Debug, Deserialize)]
struct OpenAISegment {
    id: i32,
    start: f32,
    end: f32,
    text: String,
    tokens: Option<Vec<i32>>,
    temperature: Option<f32>,
    avg_logprob: Option<f32>,
    compression_ratio: Option<f32>,
    no_speech_prob: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct OpenAIWord {
    word: String,
    start: f32,
    end: f32,
}