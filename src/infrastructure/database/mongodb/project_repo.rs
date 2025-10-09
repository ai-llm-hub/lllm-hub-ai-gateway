use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, Collection, Database};

use crate::domain::entities::ProjectApiKey;
use crate::domain::entities::Project;
use crate::domain::repositories::project_repository::ProjectRepository;
use crate::shared::error::AppError;
use crate::shared::utils::EncryptionService;

pub struct MongoProjectRepository {
    encryption: EncryptionService,
    projects: Collection<Project>,
    project_api_keys: Collection<ProjectApiKey>,
}

impl MongoProjectRepository {
    pub fn new(db: Database, encryption: EncryptionService) -> Self {
        Self {
            projects: db.collection::<Project>("projects"),
            project_api_keys: db.collection::<ProjectApiKey>("project_api_keys"),
            encryption,
        }
    }
}

#[async_trait]
impl ProjectRepository for MongoProjectRepository {
    async fn find_by_api_key(&self, api_key: &str) -> Result<Project, AppError> {

        // Extract key prefix for optimization (first 9 characters: "pk_" + 6 chars)
        let key_prefix = if api_key.len() >= 9 {
            &api_key[0..9]
        } else {
            api_key
        };

        // Find matching API key with AES-256-GCM decryption
        // Query by prefix to reduce number of keys to decrypt
        let mut cursor = self.project_api_keys
            .find(doc! {
                "is_active": true,
                "key_prefix": key_prefix
            })
            .await
            .map_err(|e| {
                tracing::error!(
                    "Database query failed while looking up API key with prefix '{}': {}",
                    key_prefix,
                    e
                );
                AppError::InternalError(format!(
                    "Failed to query project API keys: {}",
                    e
                ))
            })?;

        let mut keys_checked = 0;
        while let Some(key_doc) = cursor.try_next().await.map_err(|e| {
            tracing::error!("Failed to iterate through API key cursor: {}", e);
            AppError::InternalError(format!("Database cursor error: {}", e))
        })? {
            keys_checked += 1;
            let key_id_str = key_doc.id.as_ref().map(|id| id.to_hex()).unwrap_or_else(|| "unknown".to_string());

            // Check if key has expired
            if let Some(expires_at) = key_doc.expires_at {
                if chrono::Utc::now() > expires_at {
                    continue; // Skip expired keys
                }
            }

            // Decrypt stored key and compare with provided key
            match self.encryption.decrypt(&key_doc.key_hash) {
                Ok(decrypted_key) => {
                    if decrypted_key == api_key {

                        // Mark as used
                        if let Err(e) = self.project_api_keys
                            .update_one(
                                doc! { "_id": &key_doc.id },
                                doc! { "$set": { "last_used_at": chrono::Utc::now() } },
                            )
                            .await
                        {
                            tracing::warn!(
                                "Failed to update last_used_at for key {}: {}",
                                key_id_str,
                                e
                            );
                            // Don't fail the request, just log the warning
                        }

                        // Find associated project
                        return self.find_by_id(&key_doc.project_id).await;
                    } else {
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to decrypt API key {} (hash length: {}): {}",
                        key_id_str,
                        key_doc.key_hash.len(),
                        e
                    );
                    // Continue checking other keys instead of failing
                    continue;
                }
            }
        }

        Err(AppError::AuthenticationError("Invalid API key".to_string()))
    }

    async fn find_by_api_key_id(&self, key_id: &str) -> Result<Project, AppError> {

        let key_doc = self.project_api_keys
            .find_one(doc! { "key_id": key_id, "is_active": true })
            .await
            .map_err(|e| {
                tracing::error!("Database error while looking up API key ID {}: {}", key_id, e);
                AppError::InternalError(format!("Failed to query API key: {}", e))
            })?
            .ok_or_else(|| {
                AppError::NotFound("API key not found".to_string())
            })?;

        self.find_by_id(&key_doc.project_id).await
    }

    async fn find_by_id(&self, project_id: &str) -> Result<Project, AppError> {

        // Parse string ID to ObjectId
        let object_id = mongodb::bson::oid::ObjectId::parse_str(project_id)
            .map_err(|_| AppError::BadRequest(format!("Invalid project ID format: {}", project_id)))?;

        self.projects
            .find_one(doc! { "_id": object_id })
            .await
            .map_err(|e| {
                tracing::error!("Database error while looking up project {}: {}", project_id, e);
                AppError::InternalError(format!("Failed to query project: {}", e))
            })?
            .ok_or_else(|| {
                AppError::NotFound(format!("Project {} not found", project_id))
            })
    }

    async fn create(&self, project: &Project) -> Result<Project, AppError> {
        let project_id = project.id.as_ref().map(|id| id.to_hex()).unwrap_or_else(|| project.name.clone());

        self.projects
            .insert_one(project)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create project {}: {}", project_id, e);
                AppError::InternalError(format!("Failed to create project: {}", e))
            })?;

        Ok(project.clone())
    }

    async fn update(&self, project: &Project) -> Result<(), AppError> {
        let project_id = project.id.as_ref().map(|id| id.to_hex()).unwrap_or_else(|| project.name.clone());

        let object_id = project.id.ok_or_else(|| {
            AppError::BadRequest("Project ID is required for update".to_string())
        })?;

        let doc = bson::to_document(project).map_err(|e| {
            tracing::error!("Failed to serialize project {} to BSON: {}", project_id, e);
            AppError::InternalError(format!("Failed to serialize project: {}", e))
        })?;

        let result = self.projects
            .update_one(
                doc! { "_id": object_id },
                doc! { "$set": doc },
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to update project {}: {}", project_id, e);
                AppError::InternalError(format!("Failed to update project: {}", e))
            })?;

        if result.matched_count == 0 {
            tracing::warn!("Project {} not found during update", project_id);
            return Err(AppError::NotFound(format!(
                "Project {} not found",
                project_id
            )));
        }

        Ok(())
    }

    async fn delete(&self, project_id: &str) -> Result<(), AppError> {
        // Parse string ID to ObjectId
        let object_id = mongodb::bson::oid::ObjectId::parse_str(project_id)
            .map_err(|_| AppError::BadRequest(format!("Invalid project ID format: {}", project_id)))?;

        let result = self.projects
            .update_one(
                doc! { "_id": object_id },
                doc! { "$set": { "status": "inactive" } },
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete project {}: {}", project_id, e);
                AppError::InternalError(format!("Failed to delete project: {}", e))
            })?;

        if result.matched_count == 0 {
            tracing::warn!("Project {} not found during deletion", project_id);
            return Err(AppError::NotFound(format!(
                "Project {} not found",
                project_id
            )));
        }

        Ok(())
    }
}
