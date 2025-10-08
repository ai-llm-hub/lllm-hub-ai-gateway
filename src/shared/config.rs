use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub providers: ProvidersConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub mongodb: MongoDbConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MongoDbConfig {
    pub url: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub encryption_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvidersConfig {
    #[serde(skip_serializing)]
    pub openai_api_key: Option<String>,
    #[serde(skip_serializing)]
    pub anthropic_api_key: Option<String>,
    #[serde(skip_serializing)]
    pub google_api_key: Option<String>,
}

impl Config {
    /// Load configuration from files and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config = ConfigBuilder::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3001)?
            .set_default("server.environment", environment.as_str())?
            .set_default("database.mongodb.url", "mongodb://localhost:27017")?
            .set_default("database.mongodb.database", "llm_hub_dev")?
            // Load configuration from TOML file
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name(&format!("config.{}", environment)).required(false))
            // Override with environment variables
            .add_source(
                Environment::with_prefix("AI_GATEWAY")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate encryption key
        if self.security.encryption_key.is_empty() {
            return Err("Encryption key is required".to_string());
        }

        // Validate encryption key is base64
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::STANDARD;
        if engine.decode(&self.security.encryption_key).is_err() {
            return Err("Encryption key must be valid base64".to_string());
        }

        // Validate MongoDB URL
        if self.database.mongodb.url.is_empty() {
            return Err("MongoDB URL is required".to_string());
        }

        // Validate database name
        if self.database.mongodb.database.is_empty() {
            return Err("MongoDB database name is required".to_string());
        }

        // Validate port
        if self.server.port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }

        Ok(())
    }
}