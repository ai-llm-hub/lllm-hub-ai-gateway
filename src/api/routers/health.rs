use axum::{routing::get, Router};

use crate::api::handlers::health::health_check;

/// Health check router
pub fn health_router() -> Router {
    Router::new().route("/health", get(health_check))
}