use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

use crate::api::dto::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatRole, ChatUsage,
    FinishReason,
};
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

    /// Create chat completion using OpenAI API
    pub async fn chat_completion(
        &self,
        api_key: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError> {
        let url = format!("{}/chat/completions", self.base_url);

        // Convert our request to OpenAI format
        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAIChatMessage {
                    role: match m.role {
                        ChatRole::System => "system".to_string(),
                        ChatRole::User => "user".to_string(),
                        ChatRole::Assistant => "assistant".to_string(),
                    },
                    content: m.content.clone(),
                })
                .collect(),
            temperature: Some(request.temperature),
            max_tokens: request.max_tokens,
            stream: Some(false), // Non-streaming for now
            top_p: Some(request.top_p),
            frequency_penalty: Some(request.frequency_penalty),
            presence_penalty: Some(request.presence_penalty),
        };

        // Make API request
        let start_time = std::time::Instant::now();
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        let response_time = start_time.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::ExternalApiError(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let openai_response: OpenAIChatResponse = response.json().await?;

        // Calculate cost (simplified - should use actual pricing)
        let cost = calculate_openai_cost(
            &request.model,
            openai_response.usage.prompt_tokens,
            openai_response.usage.completion_tokens,
        );

        // Convert to our response format
        Ok(ChatCompletionResponse {
            id: openai_response.id,
            object: "chat.completion".to_string(),
            created: openai_response.created,
            model: openai_response.model,
            choices: openai_response
                .choices
                .into_iter()
                .map(|c| ChatChoice {
                    index: c.index,
                    message: ChatMessage {
                        role: match c.message.role.as_str() {
                            "system" => ChatRole::System,
                            "user" => ChatRole::User,
                            "assistant" => ChatRole::Assistant,
                            _ => ChatRole::Assistant,
                        },
                        content: c.message.content,
                    },
                    finish_reason: c.finish_reason.and_then(|r| match r.as_str() {
                        "stop" => Some(FinishReason::Stop),
                        "length" => Some(FinishReason::Length),
                        "content_filter" => Some(FinishReason::ContentFilter),
                        _ => None,
                    }),
                })
                .collect(),
            usage: ChatUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            x_llmhub: Some(crate::api::dto::ChatMetadata {
                provider: "openai".to_string(),
                cached: false,
                cost,
                response_time,
            }),
        })
    }
}

// Helper function to calculate OpenAI costs
fn calculate_openai_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    // Simplified pricing (as of 2024) - should be maintained separately
    let (input_price, output_price) = match model {
        m if m.starts_with("gpt-4-turbo") || m.starts_with("gpt-4-1106") => (0.01, 0.03),
        m if m.starts_with("gpt-4") => (0.03, 0.06),
        m if m.starts_with("gpt-3.5-turbo") => (0.0005, 0.0015),
        _ => (0.0005, 0.0015), // Default to GPT-3.5 pricing
    };

    let prompt_cost = (prompt_tokens as f64 / 1000.0) * input_price;
    let completion_cost = (completion_tokens as f64 / 1000.0) * output_price;

    prompt_cost + completion_cost
}

// OpenAI API request structures for chat
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

// OpenAI API response structures for chat
#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    created: i64,
    model: String,
    choices: Vec<OpenAIChatChoice>,
    usage: OpenAIChatUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatChoice {
    index: u32,
    message: OpenAIChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// OpenAI API response structures for transcription
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