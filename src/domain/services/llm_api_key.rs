use std::sync::Arc;

use crate::domain::entities::LlmApiKey;
use crate::domain::entities::LlmProvider;
use crate::domain::repositories::llm_api_key_repository::LlmApiKeyRepository;
use crate::shared::error::AppError;
use crate::shared::utils::EncryptionService;

/// LLM API Key service for managing encrypted provider API keys
pub struct LlmApiKeyService {
    repository: Arc<dyn LlmApiKeyRepository>,
    encryption: EncryptionService,
}

impl LlmApiKeyService {
    pub fn new(
        repository: Arc<dyn LlmApiKeyRepository>,
        encryption: EncryptionService,
    ) -> Self {
        Self {
            repository,
            encryption,
        }
    }

    /// Get decrypted LLM API key by ID
    pub async fn get_decrypted_key(&self, key_id: &str) -> Result<String, AppError> {
        // Get from database
        let llm_key = self.repository.find_by_id(key_id).await?;

        if !llm_key.is_active {
            return Err(AppError::AuthorizationError(
                "LLM API key is inactive".to_string(),
            ));
        }

        // Decrypt
        let decrypted = self.encryption.decrypt(&llm_key.encrypted_key)?;

        // Mark as used (fire and forget)
        let repo = self.repository.clone();
        let key_id = key_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = repo.mark_used(&key_id).await {
                tracing::warn!("Failed to mark LLM API key as used: {}", e);
            }
        });

        Ok(decrypted)
    }

    /// Get default LLM API key for a provider
    pub async fn get_default_key_for_provider(
        &self,
        project_id: &str,
        provider: &LlmProvider,
    ) -> Result<Option<String>, AppError> {
        let key = self
            .repository
            .find_default_for_provider(project_id, provider)
            .await?;

        match key {
            Some(llm_key) => {
                let decrypted = self.get_decrypted_key(&llm_key.key_id).await?;
                Ok(Some(decrypted))
            }
            None => Ok(None),
        }
    }

    /// Create new LLM API key
    pub async fn create_key(
        &self,
        project_id: String,
        provider: LlmProvider,
        name: String,
        api_key: String,
    ) -> Result<LlmApiKey, AppError> {
        // Encrypt the API key
        let encrypted = self.encryption.encrypt(&api_key)?;

        // Extract key prefix (first 8 characters)
        let key_prefix = if api_key.len() >= 8 {
            api_key[..8].to_string()
        } else {
            api_key.clone()
        };

        // For AI gateway, organization_id is not tracked at this level
        // and created_by is handled by the gateway itself
        let llm_key = LlmApiKey::new(
            String::new(),           // organization_id - not used in AI gateway
            provider,
            name,
            encrypted,
            key_prefix,
            String::from("system"),  // created_by - placeholder for AI gateway
        );

        self.repository.create(&llm_key).await
    }

    /// Deactivate LLM API key
    pub async fn deactivate_key(&self, key_id: &str) -> Result<(), AppError> {
        self.repository.deactivate(key_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require mock repository and Redis
    // Omitted for brevity
}