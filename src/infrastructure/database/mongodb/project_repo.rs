use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, Database};

use crate::domain::entities::llm_api_key::ProjectApiKey;
use crate::domain::entities::project::{Project, ProjectStatus};
use crate::domain::repositories::project_repository::ProjectRepository;
use crate::shared::error::AppError;
use crate::shared::utils::HashService;

pub struct MongoProjectRepository {
    db: Database,
}

impl MongoProjectRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepository for MongoProjectRepository {
    async fn find_by_api_key(&self, api_key: &str) -> Result<Project, AppError> {
        let api_keys_collection = self.db.collection::<ProjectApiKey>("project_api_keys");

        // Find matching API key with bcrypt verification
        let mut cursor = api_keys_collection
            .find(doc! { "is_active": true })
            .await?;

        while let Ok(Some(key_doc)) = cursor.try_next().await {
            if HashService::verify_api_key(api_key, &key_doc.key_hash).unwrap_or(false) {
                // Mark as used
                api_keys_collection
                    .update_one(
                        doc! { "_id": &key_doc.id },
                        doc! { "$set": { "last_used_at": chrono::Utc::now() } },
                    )
                    .await?;

                // Find associated project
                return self.find_by_id(&key_doc.project_id).await;
            }
        }

        Err(AppError::AuthenticationError("Invalid API key".to_string()))
    }

    async fn find_by_api_key_id(&self, key_id: &str) -> Result<Project, AppError> {
        let api_keys_collection = self.db.collection::<ProjectApiKey>("project_api_keys");

        let key_doc = api_keys_collection
            .find_one(doc! { "key_id": key_id, "is_active": true })
            .await?
            .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;

        self.find_by_id(&key_doc.project_id).await
    }

    async fn find_by_id(&self, project_id: &str) -> Result<Project, AppError> {
        let projects_collection = self.db.collection::<Project>("projects");

        projects_collection
            .find_one(doc! { "project_id": project_id })
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project {} not found", project_id)))
    }

    async fn create(&self, project: &Project) -> Result<Project, AppError> {
        let projects_collection = self.db.collection::<Project>("projects");

        projects_collection.insert_one(project).await?;
        Ok(project.clone())
    }

    async fn update(&self, project: &Project) -> Result<(), AppError> {
        let projects_collection = self.db.collection::<Project>("projects");

        projects_collection
            .update_one(
                doc! { "project_id": &project.project_id },
                doc! { "$set": bson::to_document(project)? },
            )
            .await?;

        Ok(())
    }

    async fn delete(&self, project_id: &str) -> Result<(), AppError> {
        let projects_collection = self.db.collection::<Project>("projects");

        projects_collection
            .update_one(
                doc! { "project_id": project_id },
                doc! { "$set": { "status": "inactive" } },
            )
            .await?;

        Ok(())
    }
}
