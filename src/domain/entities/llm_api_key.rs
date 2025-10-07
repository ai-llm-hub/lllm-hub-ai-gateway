use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// LLM provider enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    AwsBedrock,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::OpenAI => write!(f, "openai"),
            LlmProvider::Anthropic => write!(f, "anthropic"),
            LlmProvider::Google => write!(f, "google"),
            LlmProvider::Azure => write!(f, "azure"),
            LlmProvider::AwsBedrock => write!(f, "aws_bedrock"),
        }
    }
}

/// LLM API key entity for encrypted provider keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmApiKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub key_id: String,
    pub project_id: String,
    pub provider: LlmProvider,
    pub name: String,
    pub encrypted_key: String,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl LlmApiKey {
    pub fn new(
        project_id: String,
        provider: LlmProvider,
        name: String,
        encrypted_key: String,
    ) -> Self {
        Self {
            id: None,
            key_id: format!("llmk_{}", uuid::Uuid::new_v4()),
            project_id,
            provider,
            name,
            encrypted_key,
            is_active: true,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_used_at: None,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}

/// Project API key entity (customer-facing API keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKey {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub key_id: String,
    pub project_id: String,
    pub name: String,
    pub key_hash: String,  // bcrypt hash of the actual key
    pub key_prefix: String, // First 7 chars for identification
    pub key_suffix: String, // Last 4 chars for identification
    pub is_active: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ProjectApiKey {
    pub fn new(
        project_id: String,
        name: String,
        key_hash: String,
        key_prefix: String,
        key_suffix: String,
        created_by: String,
    ) -> Self {
        Self {
            id: None,
            key_id: format!("pak_{}", uuid::Uuid::new_v4()),
            project_id,
            name,
            key_hash,
            key_prefix,
            key_suffix,
            is_active: true,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    pub fn revoke(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}