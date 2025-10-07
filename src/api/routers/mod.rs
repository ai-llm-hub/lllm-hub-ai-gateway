pub mod audio;
pub mod health;

use axum::{middleware, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::dto::{
    HealthResponse, ResponseFormatDto, TimestampGranularityDto, TranscribeRequestDto,
    TranscribeResponseDto, TranscriptionSegmentDto, TranscriptionUsageDto, TranscriptionWordDto,
};
use crate::api::middleware::{authenticate, cors_layer};
use crate::domain::repositories::project_repository::ProjectRepository;
use crate::domain::services::transcription::TranscriptionService;

pub use audio::audio_router;
pub use health::health_router;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::health::health_check,
        crate::api::handlers::transcription::transcribe_audio,
    ),
    components(
        schemas(
            HealthResponse,
            TranscribeRequestDto,
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
        title = "AI Gateway API",
        version = "0.1.0",
        description = "LLM Hub Data Plane - Unified LLM API Gateway",
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

/// Create application router with all routes and middleware
pub fn create_app(
    project_repo: Arc<dyn ProjectRepository>,
    transcription_service: Arc<TranscriptionService>,
) -> Router {
    // Health routes (no authentication)
    let health_routes = health_router();

    // Audio routes (requires authentication)
    let audio_routes = audio_router(transcription_service)
        .route_layer(middleware::from_fn_with_state(
            project_repo.clone(),
            authenticate,
        ));

    // Swagger UI
    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(audio_routes)
        .merge(swagger)
        .layer(cors_layer())
}