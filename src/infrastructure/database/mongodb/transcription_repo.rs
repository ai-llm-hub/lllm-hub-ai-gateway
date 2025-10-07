use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, Database};

use crate::domain::entities::transcription::TranscriptionHistory;
use crate::domain::repositories::transcription_repository::TranscriptionRepository;
use crate::shared::error::AppError;

pub struct MongoTranscriptionRepository {
    db: Database,
}

impl MongoTranscriptionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TranscriptionRepository for MongoTranscriptionRepository {
    async fn create(&self, history: &TranscriptionHistory) -> Result<(), AppError> {
        let collection = self.db.collection::<TranscriptionHistory>("transcription_history");

        collection.insert_one(history).await?;
        Ok(())
    }

    async fn find_by_id(&self, transcription_id: &str) -> Result<TranscriptionHistory, AppError> {
        let collection = self.db.collection::<TranscriptionHistory>("transcription_history");

        collection
            .find_one(doc! { "transcription_id": transcription_id })
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Transcription {} not found", transcription_id)))
    }

    async fn find_by_file_hash(&self, file_hash: &str) -> Result<Option<TranscriptionHistory>, AppError> {
        let collection = self.db.collection::<TranscriptionHistory>("transcription_history");

        Ok(collection
            .find_one(doc! { "file_hash": file_hash })
            .await?)
    }

    async fn find_by_project(&self, project_id: &str, limit: i64) -> Result<Vec<TranscriptionHistory>, AppError> {
        let collection = self.db.collection::<TranscriptionHistory>("transcription_history");

        let mut cursor = collection
            .find(doc! { "project_id": project_id })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;

        let mut histories = Vec::new();
        while let Ok(Some(history)) = cursor.try_next().await {
            histories.push(history);
        }

        Ok(histories)
    }

    async fn count_by_project(&self, project_id: &str) -> Result<i64, AppError> {
        let collection = self.db.collection::<TranscriptionHistory>("transcription_history");

        let count = collection
            .count_documents(doc! { "project_id": project_id })
            .await?;

        Ok(count as i64)
    }
}
