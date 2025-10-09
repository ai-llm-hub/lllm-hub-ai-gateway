/// Chat Completions API DTOs
/// Based on CID specification: cid/rest-api/gateway/chat.yaml
/// OpenAI-compatible chat completions API

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Chat completion request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatCompletionRequest {
    /// Model identifier (e.g., "gpt-4", "claude-3-opus", "gemini-pro")
    pub model: String,

    /// Array of message objects
    pub messages: Vec<ChatMessage>,

    /// Sampling temperature (0-2, default: 1)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Whether to stream responses (default: false)
    #[serde(default)]
    #[allow(dead_code)]
    pub stream: bool,

    /// Nucleus sampling parameter (0-1, default: 1)
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Frequency penalty (-2 to 2, default: 0)
    #[serde(default)]
    pub frequency_penalty: f32,

    /// Presence penalty (-2 to 2, default: 0)
    #[serde(default)]
    pub presence_penalty: f32,
}

fn default_temperature() -> f32 {
    1.0
}

fn default_top_p() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FinishReason {
    Stop,
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM Hub-specific metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMetadata {
    /// Actual provider used (e.g., "openai", "anthropic")
    pub provider: String,

    /// Whether response was served from cache
    #[serde(default)]
    pub cached: bool,

    /// Cost in USD
    pub cost: f64,

    /// Response time in milliseconds
    pub response_time: u64,
}

/// Chat completion response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_llmhub: Option<ChatMetadata>,
}

// Streaming response structures

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ChatRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatChoiceChunk {
    pub index: u32,
    pub delta: ChatDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Streaming chat completion chunk
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoiceChunk>,
}

/// Error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatError {
    pub r#type: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatErrorResponse {
    pub error: ChatError,
}

impl ChatCompletionRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), String> {
        if self.messages.is_empty() {
            return Err("messages array cannot be empty".to_string());
        }

        if self.model.is_empty() {
            return Err("model cannot be empty".to_string());
        }

        if !(0.0..=2.0).contains(&self.temperature) {
            return Err("temperature must be between 0 and 2".to_string());
        }

        if !(0.0..=1.0).contains(&self.top_p) {
            return Err("top_p must be between 0 and 1".to_string());
        }

        if !(-2.0..=2.0).contains(&self.frequency_penalty) {
            return Err("frequency_penalty must be between -2 and 2".to_string());
        }

        if !(-2.0..=2.0).contains(&self.presence_penalty) {
            return Err("presence_penalty must be between -2 and 2".to_string());
        }

        Ok(())
    }
}
