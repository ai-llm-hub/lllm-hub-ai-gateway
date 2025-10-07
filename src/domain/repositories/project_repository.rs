use async_trait::async_trait;

use crate::domain::entities::project::Project;
use crate::shared::error::AppError;

/// Repository trait for project data access
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Find project by API key (via project_api_keys table)
    async fn find_by_api_key(&self, api_key: &str) -> Result<Project, AppError>;

    /// Find project by API key ID
    async fn find_by_api_key_id(&self, key_id: &str) -> Result<Project, AppError>;

    /// Find project by ID
    async fn find_by_id(&self, project_id: &str) -> Result<Project, AppError>;

    /// Create new project
    async fn create(&self, project: &Project) -> Result<Project, AppError>;

    /// Update project
    async fn update(&self, project: &Project) -> Result<(), AppError>;

    /// Delete project
    async fn delete(&self, project_id: &str) -> Result<(), AppError>;

    /// Check if project is active
    async fn is_active(&self, project: &Project) -> bool {
        project.is_active()
    }
}