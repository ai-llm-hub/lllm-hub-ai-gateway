use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project entity representing a customer project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub project_id: String,
    pub name: String,
    pub organization_id: String,
    pub status: ProjectStatus,
    pub rate_limits: RateLimits,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub tokens_per_minute: Option<u32>,
    pub max_file_size_mb: u32,
    pub max_concurrent_requests: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: Some(90000),
            max_file_size_mb: 25,
            max_concurrent_requests: 10,
        }
    }
}

impl Project {
    pub fn new(
        project_id: String,
        name: String,
        organization_id: String,
    ) -> Self {
        Self {
            id: None,
            project_id,
            name,
            organization_id,
            status: ProjectStatus::Active,
            rate_limits: RateLimits::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == ProjectStatus::Active
    }

    pub fn set_status(&mut self, status: ProjectStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_rate_limits(&mut self, limits: RateLimits) {
        self.rate_limits = limits;
        self.updated_at = Utc::now();
    }
}