pub mod audio;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use audio::{
    ResponseFormatDto, TimestampGranularityDto, TranscribeRequestDto, TranscribeResponseDto,
    TranscriptionSegmentDto, TranscriptionUsageDto, TranscriptionWordDto,
};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}