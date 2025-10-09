// This file was auto-generated from OpenAPI schemas
// Do not make direct changes to this file.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use super::shared_types::*;

/// Unified Project entity representing a customer project.
/// Used by both Backend (Control Plane) and AI Gateway (Data Plane).
/// Backend uses full features, AI Gateway uses subset for routing/rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// MongoDB ObjectId (optional, set by database)
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Organization ID (can be ObjectId or string)
    pub organization_id: String,

    /// Machine-readable project name (lowercase, alphanumeric, underscore, hyphen)
    pub name: String,

    /// Human-readable project display name
    pub display_name: String,

    /// Optional project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub visibility: ProjectVisibility,

    pub status: ProjectStatus,

    /// Budget allocation in USD (Backend-specific, optional for AI Gateway)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_allocation: Option<f64>,

    /// Amount spent in USD
    #[serde(default)]
    pub spent_amount: f64,

    /// Rate limiting configuration (AI Gateway-specific, optional for Backend)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimits>,

    /// Legacy API key (deprecated, use ProjectApiKey collection instead).
    /// Maintained for backwards compatibility only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// User ID who created the project (can be ObjectId or string)
    pub created_by: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    /// Timestamp when project was archived
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<DateTime<Utc>>,

    /// Timestamp when project was soft-deleted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,

}

impl Project {
    /// Check if project is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProjectStatus::Active)
    }
}

/// Project membership linking users to projects with roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    /// MongoDB ObjectId (optional, set by database)
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Project ID (can be ObjectId or string)
    pub project_id: String,

    /// User ID (can be ObjectId or string)
    pub user_id: String,

    pub role: ProjectRole,

    /// User ID who added this member (can be ObjectId or string)
    pub added_by: String,

    pub added_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

}

