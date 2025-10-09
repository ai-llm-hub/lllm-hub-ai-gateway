use async_trait::async_trait;

use crate::domain::entities::LlmApiKey;
use crate::domain::entities::LlmProvider;
use crate::shared::error::AppError;

/// Repository trait for LLM API key data access
#[async_trait]
pub trait LlmApiKeyRepository: Send + Sync {
    /// Find LLM API key by ID
    async fn find_by_id(&self, key_id: &str) -> Result<LlmApiKey, AppError>;

    /// Find default key for project and provider
    async fn find_default_for_provider(
        &self,
        project_id: &str,
        provider: &LlmProvider,
    ) -> Result<Option<LlmApiKey>, AppError>;

    /// Find all keys for project and provider
    async fn find_by_project_and_provider(
        &self,
        project_id: &str,
        provider: &LlmProvider,
    ) -> Result<Vec<LlmApiKey>, AppError>;

    /// Create new LLM API key
    async fn create(&self, key: &LlmApiKey) -> Result<LlmApiKey, AppError>;

    /// Update LLM API key
    async fn update(&self, key: &LlmApiKey) -> Result<(), AppError>;

    /// Mark key as used
    async fn mark_used(&self, key_id: &str) -> Result<(), AppError>;

    /// Deactivate key
    async fn deactivate(&self, key_id: &str) -> Result<(), AppError>;
}