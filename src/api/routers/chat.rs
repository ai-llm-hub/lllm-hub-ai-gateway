use axum::{routing::post, Router};
use std::sync::Arc;

use crate::api::handlers::create_chat_completion;
use crate::AppState;

/// Create the chat completions router
///
/// Provides OpenAI-compatible chat completions API
pub fn chat_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/completions", post(create_chat_completion))
}
