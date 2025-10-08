pub mod audio;
pub mod health;

use utoipa::OpenApi;

use crate::api::dto::{
    DetailedHealthResponse, HealthResponse, ResponseFormatDto, TimestampGranularityDto,
    TranscribeResponseDto, TranscriptionSegmentDto, TranscriptionUsageDto, TranscriptionWordDto,
};

pub use audio::audio_router;
pub use health::health_router;

/// OpenAPI documentation
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::health::health_check,
        crate::api::handlers::health::detailed_health_check,
        crate::api::handlers::transcription::transcribe_audio,
    ),
    components(
        schemas(
            HealthResponse,
            DetailedHealthResponse,
            TranscribeResponseDto,
            ResponseFormatDto,
            TimestampGranularityDto,
            TranscriptionSegmentDto,
            TranscriptionUsageDto,
            TranscriptionWordDto,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Audio", description = "Audio transcription endpoints")
    ),
    info(
        title = "AI Gateway - LLM Hub Data Plane",
        version = "0.1.0",
        description = "High-performance unified LLM API gateway",
        contact(
            name = "LLM Hub Team",
            email = "support@example.com"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development server"),
        (url = "https://gateway.example.com", description = "Production server")
    )
)]
pub struct ApiDoc;

