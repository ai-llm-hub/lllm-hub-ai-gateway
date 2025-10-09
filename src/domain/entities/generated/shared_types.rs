// This file was auto-generated from OpenAPI schemas
// Do not make direct changes to this file.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Supported LLM providers across the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    Openai,
    Anthropic,
    Google,
    Azure,
    AwsBedrock,
}

/// Project lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Inactive,
    Suspended,
    Archived,
    Deleted,
}

/// Project visibility scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectVisibility {
    Private,
    OrganizationWide,
    Public,
}

/// Project member role with different permission levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRole {
    Owner,
    Admin,
    Contributor,
    Viewer,
}

/// Type of LLM provider API key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmApiKeyType {
    Admin,
    Standard,
    ServiceAccount,
    ShortTerm,
    LongTerm,
    ProjectScoped,
    WorkspaceScoped,
}

impl Default for LlmApiKeyType {
    fn default() -> Self {
        Self::Standard
    }
}

/// Rate limiting configuration for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Maximum requests per minute
    #[serde(default)]
    pub requests_per_minute: u32,

    /// Maximum tokens per minute (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_minute: Option<u32>,

    /// Maximum file upload size in megabytes
    #[serde(default)]
    pub max_file_size_mb: u32,

    /// Maximum concurrent requests
    #[serde(default)]
    pub max_concurrent_requests: u32,

}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            tokens_per_minute: Some(100000),
            max_file_size_mb: 25,
            max_concurrent_requests: 10,
        }
    }
}


// ============= Implementation blocks =============

impl LlmProvider {
    /// Convert to lowercase string representation for database queries
    pub fn to_lowercase(&self) -> String {
        match self {
            LlmProvider::Openai => "openai".to_string(),
            LlmProvider::Anthropic => "anthropic".to_string(),
            LlmProvider::Google => "google".to_string(),
            LlmProvider::Azure => "azure".to_string(),
            LlmProvider::AwsBedrock => "aws_bedrock".to_string(),
        }
    }
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lowercase())
    }
}

impl std::str::FromStr for LlmProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(LlmProvider::Openai),
            "anthropic" => Ok(LlmProvider::Anthropic),
            "google" => Ok(LlmProvider::Google),
            "azure" => Ok(LlmProvider::Azure),
            "aws_bedrock" | "aws-bedrock" => Ok(LlmProvider::AwsBedrock),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

// BSON serialization support for MongoDB
impl From<LlmProvider> for mongodb::bson::Bson {
    fn from(provider: LlmProvider) -> Self {
        mongodb::bson::Bson::String(provider.to_lowercase())
    }
}
