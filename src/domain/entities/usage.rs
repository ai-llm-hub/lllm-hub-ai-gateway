use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::shared_types::LlmProvider;

/// Usage log entity for tracking API usage and costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub usage_id: String,
    #[serde(with = "crate::shared::utils::string_or_objectid")]
    pub project_id: String,  // Deserializes ObjectId from MongoDB to String
    pub api_endpoint: ApiEndpoint,
    pub provider: LlmProvider,
    pub model: String,
    pub request_metadata: RequestMetadata,
    pub response_metadata: ResponseMetadata,
    pub cost_data: CostData,
    pub cache_info: Option<CacheInfo>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// API endpoint enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiEndpoint {
    ChatCompletions,
    AudioTranscribe,
    AudioTranslate,
    Realtime,
    Embeddings,
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub audio_duration_seconds: Option<f32>,
    pub file_size_bytes: Option<i64>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub stream: bool,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub status_code: u16,
    pub latency_ms: u64,
    pub provider_latency_ms: Option<u64>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    pub finish_reason: Option<String>,
}

/// Cache information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub cache_type: CacheType,
    pub cache_hit: bool,
    pub similarity_score: Option<f32>,
}

/// Cache type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheType {
    Exact,
    Semantic,
}

/// Cost data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostData {
    pub prompt_cost_usd: Option<f64>,
    pub completion_cost_usd: Option<f64>,
    pub audio_cost_usd: Option<f64>,
    pub total_cost_usd: f64,
    pub cached_savings_usd: Option<f64>,
}

impl UsageLog {
    pub fn new(
        project_id: String,
        api_endpoint: ApiEndpoint,
        provider: LlmProvider,
        model: String,
        request_metadata: RequestMetadata,
        response_metadata: ResponseMetadata,
        cost_data: CostData,
        cache_info: Option<CacheInfo>,
        error: Option<String>,
    ) -> Self {
        Self {
            id: None,
            usage_id: format!("usage_{}", uuid::Uuid::new_v4()),
            project_id,
            api_endpoint,
            provider,
            model,
            request_metadata,
            response_metadata,
            cost_data,
            cache_info,
            error,
            created_at: Utc::now(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none() && (200..300).contains(&self.response_metadata.status_code)
    }

    pub fn is_cached(&self) -> bool {
        self.cache_info.as_ref().map_or(false, |info| info.cache_hit)
    }

    pub fn get_actual_cost(&self) -> f64 {
        if self.is_cached() {
            0.0
        } else {
            self.cost_data.total_cost_usd
        }
    }

    pub fn get_full_cost(&self) -> f64 {
        self.cost_data.total_cost_usd
    }
}