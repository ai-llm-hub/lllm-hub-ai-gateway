use async_trait::async_trait;
use bson::oid::ObjectId;
use futures::TryStreamExt;
use mongodb::{bson::doc, Collection, Database};

use crate::domain::entities::llm_api_key::ProjectApiKey;
use crate::domain::entities::project::Project;
use crate::domain::repositories::project_repository::ProjectRepository;
use crate::shared::error::AppError;
use crate::shared::utils::EncryptionService;
use tracing::info;

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

        info!("Looking up API key in database");

        // Extract key prefix for optimization (first 9 characters: "pk_" + 6 chars)
        let key_prefix = if api_key.len() >= 9 {
            &api_key[0..9]
        } else {
            api_key
        };

        info!("Using key prefix: {}", key_prefix);

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

        info!("Successfully queried database for keys with prefix: {}", key_prefix);

        let mut keys_checked = 0;
        while let Some(key_doc) = cursor.try_next().await.map_err(|e| {
            tracing::error!("Failed to iterate through API key cursor: {}", e);
            AppError::InternalError(format!("Database cursor error: {}", e))
        })? {
            info!("Evaluating candidate API key from database");
            keys_checked += 1;
            let key_id_str = key_doc.id.as_ref().map(|id| id.to_hex()).unwrap_or_else(|| "unknown".to_string());
            info!("Checking key ID: {} (attempt #{})", key_id_str, keys_checked);

            // Check if key has expired
            if let Some(expires_at) = key_doc.expires_at {
                if chrono::Utc::now() > expires_at {
                    info!("Key {} has expired, skipping", key_id_str);
                    continue; // Skip expired keys
                }
            }

            // Decrypt stored key and compare with provided key
            match self.encryption.decrypt(&key_doc.key_hash) {
                Ok(decrypted_key) => {
                    if decrypted_key == api_key {
                        info!("API key matched! Marking as used and fetching project");

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
                        info!("Decrypted key does not match provided key");
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

        info!("No matching API key found after checking {} candidates", keys_checked);
        Err(AppError::AuthenticationError("Invalid API key".to_string()))
    }

    async fn find_by_api_key_id(&self, key_id: &str) -> Result<Project, AppError> {
        info!("Looking up project by API key ID: {}", key_id);

        let key_doc = self.project_api_keys
            .find_one(doc! { "key_id": key_id, "is_active": true })
            .await
            .map_err(|e| {
                tracing::error!("Database error while looking up API key ID {}: {}", key_id, e);
                AppError::InternalError(format!("Failed to query API key: {}", e))
            })?
            .ok_or_else(|| {
                info!("API key ID {} not found or inactive", key_id);
                AppError::NotFound("API key not found".to_string())
            })?;

        info!("Found API key {}, fetching project {}", key_id, key_doc.project_id);
        self.find_by_id(&key_doc.project_id).await
    }

    async fn find_by_id(&self, project_id: &str) -> Result<Project, AppError> {
        info!("Looking up project by ID: {}", project_id);

        // Parse project_id as ObjectId for querying the _id field
        let object_id = ObjectId::parse_str(project_id)
            .map_err(|e| AppError::BadRequest(format!("Invalid project ID format: {}", e)))?;

        self.projects
            .find_one(doc! { "_id": object_id })
            .await
            .map_err(|e| {
                tracing::error!("Database error while looking up project {}: {}", project_id, e);
                AppError::InternalError(format!("Failed to query project: {}", e))
            })?
            .ok_or_else(|| {
                info!("Project {} not found in database", project_id);
                AppError::NotFound(format!("Project {} not found", project_id))
            })
    }

    async fn create(&self, project: &Project) -> Result<Project, AppError> {
        info!("Creating new project: {}", project.project_id);

        self.projects
            .insert_one(project)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create project {}: {}", project.project_id, e);
                AppError::InternalError(format!("Failed to create project: {}", e))
            })?;

        info!("Successfully created project: {}", project.project_id);
        Ok(project.clone())
    }

    async fn update(&self, project: &Project) -> Result<(), AppError> {
        info!("Updating project: {}", project.project_id);

        let doc = bson::to_document(project).map_err(|e| {
            tracing::error!("Failed to serialize project {} to BSON: {}", project.project_id, e);
            AppError::InternalError(format!("Failed to serialize project: {}", e))
        })?;

        let result = self.projects
            .update_one(
                doc! { "project_id": &project.project_id },
                doc! { "$set": doc },
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to update project {}: {}", project.project_id, e);
                AppError::InternalError(format!("Failed to update project: {}", e))
            })?;

        if result.matched_count == 0 {
            tracing::warn!("Project {} not found during update", project.project_id);
            return Err(AppError::NotFound(format!(
                "Project {} not found",
                project.project_id
            )));
        }

        info!("Successfully updated project: {}", project.project_id);
        Ok(())
    }

    async fn delete(&self, project_id: &str) -> Result<(), AppError> {
        info!("Soft-deleting project: {}", project_id);

        let result = self.projects
            .update_one(
                doc! { "project_id": project_id },
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

        info!("Successfully deleted project: {}", project_id);
        Ok(())
    }
}
