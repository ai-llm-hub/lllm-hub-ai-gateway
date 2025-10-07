use async_trait::async_trait;

use crate::domain::entities::usage::UsageLog;
use crate::shared::error::AppError;

/// Repository trait for usage log data access
#[async_trait]
pub trait UsageRepository: Send + Sync {
    /// Create new usage log
    async fn create(&self, log: &UsageLog) -> Result<(), AppError>;

    /// Find usage logs by project
    async fn find_by_project(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<UsageLog>, AppError>;

    /// Calculate total cost for project
    async fn calculate_total_cost(
        &self,
        project_id: &str,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<f64, AppError>;
}