use axum::{routing::get, Router};
use std::sync::Arc;

use crate::api::handlers::health::{detailed_health_check, health_check};
use crate::AppState;

/// Health check router
pub fn health_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/detailed", get(detailed_health_check))
}