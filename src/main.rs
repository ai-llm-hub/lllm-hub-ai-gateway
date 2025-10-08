mod api;
mod domain;
mod infrastructure;
mod shared;

use axum::{extract::DefaultBodyLimit, http::StatusCode, Router};
use std::{sync::Arc, time::Instant};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use domain::services::{LlmApiKeyService, TranscriptionService};
use infrastructure::{
    connect_mongodb, MongoLlmApiKeyRepository, MongoProjectRepository,
    MongoTranscriptionRepository, MongoUsageRepository,
};
use shared::{Config, EncryptionService};

#[derive(Clone)]
pub struct AppState {
    pub start_time: Instant,
    pub version: String,
    pub db: mongodb::Database,
    pub config: Config,
    pub project_repo: Arc<dyn domain::repositories::ProjectRepository>,
    pub llm_key_repo: Arc<dyn domain::repositories::LlmApiKeyRepository>,
    pub transcription_repo: Arc<dyn domain::repositories::TranscriptionRepository>,
    pub usage_repo: Arc<dyn domain::repositories::UsageRepository>,
    pub llm_key_service: Arc<LlmApiKeyService>,
    pub transcription_service: Arc<TranscriptionService>,
}

fn create_trace_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::load()?;

    // Enable Rust backtrace for better error debugging
    if config.server.environment == "development" {
        std::env::set_var("RUST_BACKTRACE", "full");
        std::env::set_var("RUST_LIB_BACKTRACE", "full");
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_gateway=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .with_level(true)
                .with_ansi(config.server.environment == "development"),
        )
        .init();

    info!(
        "🔧 Loaded configuration for environment: {}",
        config.server.environment
    );

    // Validate configuration
    config.validate().map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;

    // Initialize database connection with retries
    info!("🔄 Initializing database connection...");
    let mongodb_connection_string = config.get_mongodb_connection_string();
    info!(
        "📦 MongoDB: {}:{}@{}:{}/{}",
        config.database.mongodb.username,
        "***", // Hide password in logs
        config.database.mongodb.url,
        config.database.mongodb.port,
        config.database.mongodb.database
    );

    info!("Connecting to MongoDB...{}", mongodb_connection_string);

    let db = match connect_mongodb(
        &mongodb_connection_string,
        &config.database.mongodb.database,
    )
    .await {
        Ok(database) => {
            info!("✅ Database connection established successfully");
            database
        }
        Err(e) => {
            tracing::error!("❌ Failed to connect to database: {}", e);
            tracing::error!(
                "Please ensure MongoDB is running and accessible at {}:{}",
                config.database.mongodb.url,
                config.database.mongodb.port
            );
            std::process::exit(1);
        }
    };

    // Initialize encryption service (needed by repositories)
    let encryption = match EncryptionService::new(&config.security.encryption_key) {
        Ok(service) => {
            info!("✅ Encryption service initialized");
            service
        }
        Err(e) => {
            tracing::error!("❌ Failed to initialize encryption service: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize repositories
    let project_repo = Arc::new(MongoProjectRepository::new(db.clone(), encryption.clone()));
    let llm_key_repo = Arc::new(MongoLlmApiKeyRepository::new(db.clone()));
    let transcription_repo = Arc::new(MongoTranscriptionRepository::new(db.clone()));
    let usage_repo = Arc::new(MongoUsageRepository::new(db.clone()));

    // Initialize services
    let llm_key_service = Arc::new(LlmApiKeyService::new(
        llm_key_repo.clone(),
        encryption,
    ));

    let transcription_service = Arc::new(TranscriptionService::new(
        transcription_repo.clone(),
        llm_key_service.clone(),
    ));

    // Create application state with all services
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        db: db.clone(),
        config: config.clone(),
        project_repo: project_repo.clone(),
        llm_key_repo: llm_key_repo.clone(),
        transcription_repo: transcription_repo.clone(),
        usage_repo: usage_repo.clone(),
        llm_key_service: llm_key_service.clone(),
        transcription_service: transcription_service.clone(),
    });

    // Create routers
    let health_routes = api::routers::health_router();
    let audio_routes = api::routers::audio_router()
        .route_layer(axum::middleware::from_fn_with_state(
            state.project_repo.clone(),
            api::middleware::authenticate,
        ));

    // Build our application with routes
    let app = Router::new()
        // Health check endpoints (no authentication)
        .merge(health_routes)
        // API v1 routes
        .nest("/v1/audio", audio_routes)
        // Add state
        .with_state(state.clone());

    // Add Swagger UI only if enabled AND in development mode
    let app = if config.server.environment == "development" {
        app.merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", api::ApiDoc::openapi())
        )
    } else {
        app
    };

    // Add middleware
    // Note: Order matters! CORS should be outermost, then tracing
    let app = app
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024)) // 25MB max body size for audio file uploads
        .layer(create_trace_layer())
        .layer(api::middleware::cors_layer())
        // Fallback handler
        .fallback(fallback);

    // Get address from config
    let addr = format!("{}:{}", config.server.host, config.server.port);

    info!("🚀 AI Gateway starting on http://{}", addr);
    if config.server.environment == "development" {
        info!("📚 Swagger UI available at http://{}/swagger-ui/", addr);
    }

    // Create the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new()
            .route("/health", get(api::handlers::health::health_check));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}