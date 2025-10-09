pub mod audio;
pub mod chat;
pub mod health;

#[allow(unused_imports)]
use utoipa::OpenApi;

use crate::api::dto::{
    ChatChoice, ChatChoiceChunk, ChatCompletionChunk, ChatCompletionRequest,
    ChatCompletionResponse, ChatDelta, ChatError, ChatErrorResponse, ChatMessage, ChatMetadata,
    ChatRole, ChatUsage, DetailedHealthResponse, FinishReason, HealthResponse, ResponseFormatDto,
    TimestampGranularityDto, TranscribeResponseDto, TranscriptionSegmentDto, TranscriptionUsageDto,
    TranscriptionWordDto,
};

pub use audio::audio_router;
pub use chat::chat_router;
pub use health::health_router;

/// OpenAPI documentation
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::health::health_check,
        crate::api::handlers::health::detailed_health_check,
        crate::api::handlers::transcription::transcribe_audio,
        crate::api::handlers::chat::create_chat_completion,
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
            ChatCompletionRequest,
            ChatCompletionResponse,
            ChatChoice,
            ChatMessage,
            ChatRole,
            ChatUsage,
            ChatMetadata,
            FinishReason,
            ChatErrorResponse,
            ChatError,
            ChatCompletionChunk,
            ChatChoiceChunk,
            ChatDelta,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Audio", description = "Audio transcription endpoints"),
        (name = "Chat Completions", description = "OpenAI-compatible chat completions API")
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

