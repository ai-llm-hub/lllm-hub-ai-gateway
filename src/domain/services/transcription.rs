use std::sync::Arc;
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::domain::entities::LlmProvider;
use crate::domain::entities::transcription::{
    TranscriptionHistory, TranscriptionRequest, TranscriptionResponse,
};
use crate::domain::repositories::transcription_repository::TranscriptionRepository;
use crate::domain::services::llm_api_key::LlmApiKeyService;
use crate::domain::services::providers::OpenAIProvider;
use crate::shared::error::AppError;

/// Transcription service orchestrating transcription workflow
pub struct TranscriptionService {
    repository: Arc<dyn TranscriptionRepository>,
    llm_key_service: Arc<LlmApiKeyService>,
    openai_provider: OpenAIProvider,
}

impl TranscriptionService {
    pub fn new(
        repository: Arc<dyn TranscriptionRepository>,
        llm_key_service: Arc<LlmApiKeyService>,
    ) -> Self {
        Self {
            repository,
            llm_key_service,
            openai_provider: OpenAIProvider::new(),
        }
    }

    /// Transcribe audio file
    pub async fn transcribe(
        &self,
        project_id: String,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResponse, AppError> {
        let start_time = Instant::now();

        // Calculate file hash for deduplication
        let file_hash = self.calculate_file_hash(&request.file_data);

        // Determine provider (default to OpenAI for now)
        let provider = LlmProvider::Openai;

        // Get LLM API key
        let api_key = if let Some(key_id) = &request.llm_api_key_id {
            // Use specified key
            self.llm_key_service.get_decrypted_key(key_id).await?
        } else {
            // Use default key for provider
            self.llm_key_service
                .get_default_key_for_provider(&project_id, &provider)
                .await?
                .ok_or_else(|| {
                    AppError::ConfigError(format!(
                        "No LLM API key configured for provider: {:?}",
                        provider
                    ))
                })?
        };

        // Call provider API
        let response = self
            .openai_provider
            .transcribe(
                &api_key,
                request.file_data.clone(),
                request.file_name.clone(),
                request.model.clone(),
                request.language.clone(),
                request.prompt.clone(),
                request.response_format.clone(),
                request.temperature,
                request.timestamp_granularities.clone(),
            )
            .await?;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        // Log usage
        self.log_usage(
            project_id,
            &request,
            &response,
            &file_hash,
            response_time_ms,
            false,
        )
        .await?;

        Ok(response)
    }

    /// Calculate SHA-256 hash of file data
    fn calculate_file_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Log transcription usage
    async fn log_usage(
        &self,
        project_id: String,
        request: &TranscriptionRequest,
        response: &TranscriptionResponse,
        file_hash: &str,
        response_time_ms: u64,
        from_cache: bool,
    ) -> Result<(), AppError> {
        let cost_usd = if from_cache {
            0.0
        } else {
            response
                .usage
                .as_ref()
                .and_then(|u| u.estimated_cost_usd)
                .unwrap_or(0.0)
        };

        let history = TranscriptionHistory::new(
            project_id,
            LlmProvider::Openai,
            file_hash.to_string(),
            request.file_name.clone(),
            request.file_data.len(),
            response.duration,
            request
                .model
                .clone()
                .unwrap_or_else(|| "whisper-1".to_string()),
            response.language.clone(),
            response.text.clone(),
            cost_usd,
            response_time_ms,
            from_cache,
        );

        // Log in background
        let repo = self.repository.clone();
        tokio::spawn(async move {
            if let Err(e) = repo.create(&history).await {
                tracing::error!("Failed to log transcription usage: {}", e);
            }
        });

        Ok(())
    }
}