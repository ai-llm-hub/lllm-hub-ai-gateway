use axum::{routing::post, Router};
use std::sync::Arc;

use crate::api::handlers::transcription::transcribe_audio;
use crate::domain::services::transcription::TranscriptionService;

/// Audio API router
pub fn audio_router(service: Arc<TranscriptionService>) -> Router {
    Router::new()
        .route("/v1/audio/transcribe", post(transcribe_audio))
        .with_state(service)
}