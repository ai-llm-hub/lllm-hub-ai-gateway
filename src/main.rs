mod api;
mod domain;
mod infrastructure;
mod shared;

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::create_app;
use domain::services::{LlmApiKeyService, TranscriptionService};
use infrastructure::{
    connect_mongodb, MongoLlmApiKeyRepository, MongoProjectRepository,
    MongoTranscriptionRepository,
};
use shared::{Config, EncryptionService};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;

    tracing::info!("Starting AI Gateway v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Environment: {}", config.server.environment);

    // Validate configuration
    config.validate().map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;

    // Connect to MongoDB
    let db = connect_mongodb(
        &config.database.mongodb.url,
        &config.database.mongodb.database,
    )
    .await?;

    // Initialize repositories
    let project_repo = Arc::new(MongoProjectRepository::new(db.clone()));
    let llm_key_repo = Arc::new(MongoLlmApiKeyRepository::new(db.clone()));
    let transcription_repo = Arc::new(MongoTranscriptionRepository::new(db.clone()));

    // Initialize encryption service
    let encryption = EncryptionService::new(&config.security.encryption_key)?;

    // Initialize services
    let llm_key_service = Arc::new(LlmApiKeyService::new(
        llm_key_repo,
        encryption,
    ));

    let transcription_service = Arc::new(TranscriptionService::new(
        transcription_repo,
        llm_key_service,
    ));

    // Create application with routes
    let app = create_app(
        project_repo,
        transcription_service);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("🚀 AI Gateway listening on http://{}", addr);
    tracing::info!("📚 Swagger UI available at http://{}/swagger-ui/", addr);

    axum::serve(listener, app).await?;

    Ok(())
}