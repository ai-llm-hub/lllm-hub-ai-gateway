use axum::Json;

use crate::api::dto::HealthResponse;
use crate::shared::error::AppError;

/// Health check handler
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}