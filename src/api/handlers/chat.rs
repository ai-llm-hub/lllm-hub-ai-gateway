/// Chat completions handler
/// Based on CID specification: cid/rest-api/gateway/chat.yaml

use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;
use tracing::{error, info};

use crate::api::dto::{ChatCompletionRequest, ChatCompletionResponse, ChatErrorResponse, ChatError};
use crate::domain::services::providers::OpenAIProvider;
use crate::shared::error::AppError;
use crate::AppState;

/// Create chat completion
///
/// OpenAI-compatible chat completions API with intelligent routing and optimization
#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    tag = "Chat Completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Chat completion successful", body = ChatCompletionResponse),
        (status = 400, description = "Bad request - invalid parameters", body = ChatErrorResponse),
        (status = 401, description = "Unauthorized - invalid API key", body = ChatErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ChatErrorResponse),
        (status = 500, description = "Internal server error", body = ChatErrorResponse),
        (status = 503, description = "Service unavailable - all providers down", body = ChatErrorResponse)
    ),
    security(
        ("projectApiKey" = [])
    )
)]
pub async fn create_chat_completion(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, Json<ChatErrorResponse>)> {
    info!("Chat completion request: model={}, messages={}", request.model, request.messages.len());

    // Validate request
    if let Err(e) = request.validate() {
        error!("Invalid chat completion request: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatErrorResponse {
                error: ChatError {
                    r#type: "invalid_request_error".to_string(),
                    message: e,
                    code: "invalid_request".to_string(),
                },
            }),
        ));
    }

    // TODO: Get project from authentication context
    // For now, we'll use a default OpenAI API key from environment
    let openai_api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
        error!("OPENAI_API_KEY not set");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatErrorResponse {
                error: ChatError {
                    r#type: "configuration_error".to_string(),
                    message: "OpenAI API key not configured".to_string(),
                    code: "missing_api_key".to_string(),
                },
            }),
        )
    })?;

    // TODO: Implement intelligent routing based on model
    // For now, route all requests to OpenAI
    let provider = OpenAIProvider::new();

    // Call provider
    match provider.chat_completion(&openai_api_key, &request).await {
        Ok(response) => {
            info!("Chat completion successful: id={}, usage={} tokens",
                response.id, response.usage.total_tokens);

            // TODO: Log usage to database for cost tracking

            Ok(Json(response))
        }
        Err(e) => {
            error!("Chat completion failed: {}", e);

            let (status, error_type, code) = match &e {
                AppError::ExternalApiError(msg) if msg.contains("401") || msg.contains("authentication") => {
                    (StatusCode::UNAUTHORIZED, "authentication_error", "invalid_api_key")
                }
                AppError::ExternalApiError(msg) if msg.contains("429") || msg.contains("rate_limit") => {
                    (StatusCode::TOO_MANY_REQUESTS, "rate_limit_error", "rate_limit_exceeded")
                }
                AppError::ExternalApiError(msg) if msg.contains("400") => {
                    (StatusCode::BAD_REQUEST, "invalid_request_error", "invalid_request")
                }
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "api_error", "provider_error")
                }
            };

            Err((
                status,
                Json(ChatErrorResponse {
                    error: ChatError {
                        r#type: error_type.to_string(),
                        message: e.to_string(),
                        code: code.to_string(),
                    },
                }),
            ))
        }
    }
}
