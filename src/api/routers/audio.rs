use axum::{routing::post, Router};

use crate::api::handlers::transcription::transcribe_audio;

/// Audio API router
pub fn audio_router() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/transcribe", post(transcribe_audio))
}