#![allow(dead_code)]

//! Shared types that must be consistent across backend and AI gateway
//!
//! This file contains enum types and basic structures that are used by both
//! the Control Plane (backend) and Data Plane (AI gateway). These types are
//! automatically synced via git hooks to ensure consistency.
//!
//! **IMPORTANT**: This file is the source of truth and is automatically synced
//! to ai-gateway/src/domain/entities/shared_types.rs. Do not edit the AI gateway
//! version directly - all changes should be made here.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// LLM Provider enumeration
/// Represents all supported LLM providers across the platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
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
            LlmProvider::AwsBedrock => write!(f, "aws-bedrock"),
        }
    }
}

impl std::str::FromStr for LlmProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(LlmProvider::OpenAI),
            "anthropic" => Ok(LlmProvider::Anthropic),
            "google" => Ok(LlmProvider::Google),
            "azure" => Ok(LlmProvider::Azure),
            "aws-bedrock" | "aws_bedrock" => Ok(LlmProvider::AwsBedrock),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}
