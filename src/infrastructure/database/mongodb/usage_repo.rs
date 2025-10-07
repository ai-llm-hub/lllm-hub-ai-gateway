use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::{bson::doc, Database};

use crate::domain::entities::usage::UsageLog;
use crate::domain::repositories::usage_repository::UsageRepository;
use crate::shared::error::AppError;

pub struct MongoUsageRepository {
    db: Database,
}

impl MongoUsageRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UsageRepository for MongoUsageRepository {
    async fn create(&self, log: &UsageLog) -> Result<(), AppError> {
        let collection = self.db.collection::<UsageLog>("usage_logs");

        collection.insert_one(log).await?;
        Ok(())
    }

    async fn find_by_project(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<UsageLog>, AppError> {
        let collection = self.db.collection::<UsageLog>("usage_logs");

        let mut cursor = collection
            .find(doc! { "project_id": project_id })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;

        let mut logs = Vec::new();
        while let Ok(Some(log)) = cursor.try_next().await {
            logs.push(log);
        }

        Ok(logs)
    }

    async fn calculate_total_cost(
        &self,
        project_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<f64, AppError> {
        let collection = self.db.collection::<UsageLog>("usage_logs");

        let mut filter = doc! { "project_id": project_id };

        if let Some(start) = start_date {
            filter.insert("created_at", doc! { "$gte": start });
        }

        if let Some(end) = end_date {
            if filter.contains_key("created_at") {
                filter.get_document_mut("created_at")?
                    .insert("$lte", end);
            } else {
                filter.insert("created_at", doc! { "$lte": end });
            }
        }

        let pipeline = vec![
            doc! { "$match": filter },
            doc! { "$group": {
                "_id": null,
                "total_cost": { "$sum": "$cost_data.total_cost_usd" }
            } }
        ];

        let mut cursor = collection.aggregate(pipeline).await?;

        if let Ok(Some(result)) = cursor.try_next().await {
            let total = result
                .get_f64("total_cost")
                .unwrap_or_default();
            Ok(total)
        } else {
            Ok(0.0)
        }
    }
}
