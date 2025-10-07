use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, Database};

use crate::domain::entities::llm_api_key::{LlmApiKey, LlmProvider};
use crate::domain::repositories::llm_api_key_repository::LlmApiKeyRepository;
use crate::shared::error::AppError;

pub struct MongoLlmApiKeyRepository {
    db: Database,
}

impl MongoLlmApiKeyRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl LlmApiKeyRepository for MongoLlmApiKeyRepository {
    async fn find_by_id(&self, key_id: &str) -> Result<LlmApiKey, AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        collection
            .find_one(doc! { "key_id": key_id })
            .await?
            .ok_or_else(|| AppError::NotFound(format!("LLM API key {} not found", key_id)))
    }

    async fn find_default_for_provider(
        &self,
        project_id: &str,
        provider: &LlmProvider,
    ) -> Result<Option<LlmApiKey>, AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        Ok(collection
            .find_one(doc! {
                "project_id": project_id,
                "provider": bson::to_bson(provider)?,
                "is_active": true,
                "is_default": true
            })
            .await?)
    }

    async fn find_by_project_and_provider(
        &self,
        project_id: &str,
        provider: &LlmProvider,
    ) -> Result<Vec<LlmApiKey>, AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        let mut cursor = collection
            .find(doc! {
                "project_id": project_id,
                "provider": bson::to_bson(provider)?,
                "is_active": true
            })
            .await?;

        let mut keys = Vec::new();
        while let Ok(Some(key)) = cursor.try_next().await {
            keys.push(key);
        }

        Ok(keys)
    }

    async fn create(&self, key: &LlmApiKey) -> Result<LlmApiKey, AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        collection.insert_one(key).await?;
        Ok(key.clone())
    }

    async fn update(&self, key: &LlmApiKey) -> Result<(), AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        collection
            .update_one(
                doc! { "key_id": &key.key_id },
                doc! { "$set": bson::to_document(key)? },
            )
            .await?;

        Ok(())
    }

    async fn mark_used(&self, key_id: &str) -> Result<(), AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        collection
            .update_one(
                doc! { "key_id": key_id },
                doc! { "$set": { "last_used_at": chrono::Utc::now() } },
            )
            .await?;

        Ok(())
    }

    async fn deactivate(&self, key_id: &str) -> Result<(), AppError> {
        let collection = self.db.collection::<LlmApiKey>("llm_api_keys");

        collection
            .update_one(
                doc! { "key_id": key_id },
                doc! { "$set": {
                    "is_active": false,
                    "updated_at": chrono::Utc::now()
                } },
            )
            .await?;

        Ok(())
    }
}
