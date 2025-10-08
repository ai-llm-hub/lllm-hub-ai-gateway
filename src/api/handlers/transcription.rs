use axum::{extract::State, Extension, Json};
use axum::extract::Multipart;
use std::sync::Arc;

use crate::api::dto::{TranscribeRequestDto, TranscribeResponseDto};
use crate::domain::entities::project::Project;
use crate::domain::entities::transcription::TranscriptionRequest;
use crate::shared::error::AppError;
use crate::AppState;

/// Audio transcription handler
#[utoipa::path(
    post,
    path = "/v1/audio/transcribe",
    tag = "Audio",
    request_body(content = TranscribeResponseDto, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Transcription successful", body = TranscribeResponseDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 413, description = "File too large"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("ApiKey" = [])
    )
)]
pub async fn transcribe_audio(
    State(state): State<Arc<AppState>>,
    Extension(project): Extension<Project>,
    mut multipart: Multipart,
) -> Result<Json<TranscribeResponseDto>, AppError> {
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut request_dto = TranscribeRequestDto {
        model: None,
        language: None,
        prompt: None,
        response_format: None,
        temperature: None,
        timestamp_granularities: None,
        llm_api_key_id: None,
    };

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let field_name = field
            .name()
            .ok_or_else(|| AppError::BadRequest("Missing field name".to_string()))?
            .to_string();

        match field_name.as_str() {
            "file" => {
                file_name = field
                    .file_name()
                    .unwrap_or("audio.wav")
                    .to_string();
                file_data = field.bytes().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?
                    .to_vec();
            }
            "model" => {
                request_dto.model = Some(field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read model: {}", e)))?);
            }
            "language" => {
                request_dto.language = Some(field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read language: {}", e)))?);
            }
            "prompt" => {
                request_dto.prompt = Some(field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read prompt: {}", e)))?);
            }
            "response_format" => {
                let format_str = field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read response_format: {}", e)))?;
                request_dto.response_format = serde_json::from_str(&format!("\"{}\"", format_str))
                    .ok();
            }
            "temperature" => {
                let temp_str = field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read temperature: {}", e)))?;
                request_dto.temperature = temp_str.parse().ok();
            }
            "timestamp_granularities" => {
                request_dto.timestamp_granularities = Some(field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read timestamp_granularities: {}", e)))?);
            }
            "llm_api_key_id" => {
                request_dto.llm_api_key_id = Some(field.text().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read llm_api_key_id: {}", e)))?);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate file data
    if file_data.is_empty() {
        return Err(AppError::BadRequest("No file provided".to_string()));
    }

    // Check file size against project limits
    let file_size_mb = file_data.len() as f32 / (1024.0 * 1024.0);
    if file_size_mb > project.rate_limits.max_file_size_mb as f32 {
        return Err(AppError::BadRequest(format!(
            "File size {:.2}MB exceeds limit of {}MB",
            file_size_mb, project.rate_limits.max_file_size_mb
        )));
    }

    // Parse timestamp granularities before moving request_dto
    let timestamp_granularities = request_dto.parse_timestamp_granularities();

    // Create transcription request
    let transcription_request = TranscriptionRequest {
        file_data,
        file_name,
        model: request_dto.model,
        language: request_dto.language,
        prompt: request_dto.prompt,
        response_format: request_dto.response_format.map(|f| f.into()),
        temperature: request_dto.temperature,
        timestamp_granularities,
        llm_api_key_id: request_dto.llm_api_key_id,
    };

    // Perform transcription
    let response = state.transcription_service
        .transcribe(project.project_id, transcription_request)
        .await?;

    Ok(Json(TranscribeResponseDto::from(response)))
}