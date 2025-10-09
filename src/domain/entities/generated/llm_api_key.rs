// This file was auto-generated from OpenAPI schemas
// Do not make direct changes to this file.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use super::shared_types::*;

/// LLM API key entity for encrypted provider keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmApiKey {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique key identifier
    pub key_id: String,

    /// Organization ID
    pub organization_id: String,

    pub provider: LlmProvider,

    /// Human-readable name for the key
    pub name: String,

    /// AES-256-GCM encrypted provider API key
    pub encrypted_key: String,

    /// First few characters of the key for display
    pub key_prefix: String,

    pub key_type: LlmApiKeyType,

    /// Optional list of permissions/scopes for the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,

    /// Optional scope description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Whether the key is currently active
    #[serde(default)]
    pub is_active: bool,

    /// Whether this is the default key for the provider
    #[serde(default)]
    pub is_default: bool,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<mongodb::bson::Document>,

    /// User ID who created the key
    pub created_by: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    /// Timestamp when key was last used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,

    /// Optional expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

}

impl LlmApiKey {
    /// Create a new LLM API key
    pub fn new(
        organization_id: String,
        provider: LlmProvider,
        name: String,
        encrypted_key: String,
        key_prefix: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            key_id: uuid::Uuid::new_v4().to_string(),
            organization_id,
            provider,
            name,
            encrypted_key,
            key_prefix,
            key_type: LlmApiKeyType::default(),
            permissions: None,
            scope: None,
            is_active: true,
            is_default: false,
            metadata: None,
            created_by,
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at: None,
        }
    }
}

/// Project API key entity for customer authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKey {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique key identifier
    pub key_id: String,

    /// Project ID
    pub project_id: String,

    /// Human-readable name for the API key
    pub name: String,

    /// AES-256-GCM encrypted key hash
    pub key_hash: String,

    /// First 9 chars for identification
    pub key_prefix: String,

    /// Last 4 chars for identification
    pub key_suffix: String,

    /// Whether the API key is currently active
    #[serde(default)]
    pub is_active: bool,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    /// Timestamp when key was last used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,

    /// Optional expiration timestamp for temporary keys
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

}

