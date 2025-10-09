use axum::{extract::State, Json};
use chrono::Utc;
use std::sync::Arc;

use crate::api::dto::{DetailedHealthResponse, HealthResponse};
use crate::shared::error::AppError;
use crate::AppState;

/// Health check handler
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unavailable")
    )
)]
pub async fn health_check() -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
    }))
}

/// Detailed health check handler
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Detailed health information", body = DetailedHealthResponse),
        (status = 503, description = "Service is unavailable")
    )
)]
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DetailedHealthResponse>, AppError> {
    let uptime = state.start_time.elapsed().as_secs();

    Ok(Json(DetailedHealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: state.version.clone(),
        service: "ai-gateway".to_string(),
        uptime_seconds: uptime,
        environment: state.config.server.environment.clone(),
    }))
}