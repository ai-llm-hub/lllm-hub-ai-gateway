use async_trait::async_trait;

use crate::domain::entities::transcription::TranscriptionHistory;
use crate::shared::error::AppError;

/// Repository trait for transcription data access
#[async_trait]
pub trait TranscriptionRepository: Send + Sync {
    /// Create new transcription history record
    async fn create(&self, history: &TranscriptionHistory) -> Result<(), AppError>;

    /// Find transcription by ID
    async fn find_by_id(&self, transcription_id: &str) -> Result<TranscriptionHistory, AppError>;

    /// Find transcriptions by file hash
    async fn find_by_file_hash(&self, file_hash: &str) -> Result<Option<TranscriptionHistory>, AppError>;

    /// Find transcriptions by project
    async fn find_by_project(&self, project_id: &str, limit: i64) -> Result<Vec<TranscriptionHistory>, AppError>;

    /// Count transcriptions by project
    async fn count_by_project(&self, project_id: &str) -> Result<i64, AppError>;
}