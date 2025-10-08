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
    pub url: String,           // Host only (e.g., "localhost")
    pub port: u16,             // Default: 27017
    pub username: String,      // MongoDB username
    pub password: String,      // MongoDB password
    pub database: String,      // Database name
    pub auth_source: String,   // Authentication database (usually "admin")
    pub connection_timeout_ms: u64,
    pub max_pool_size: u32,
    pub min_pool_size: u32,
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
    /// Get MongoDB connection string with authentication
    pub fn get_mongodb_connection_string(&self) -> String {
        format!(
            "mongodb://{}:{}@{}:{}/{}?authSource={}",
            self.database.mongodb.username,
            self.database.mongodb.password,
            self.database.mongodb.url,
            self.database.mongodb.port,
            self.database.mongodb.database,
            self.database.mongodb.auth_source
        )
    }

    /// Load configuration from files and environment variables
    ///
    /// Supports layered .env loading:
    /// 1. Loads base .env file (common settings)
    /// 2. Loads .env.{environment} file (environment-specific overrides)
    /// 3. Environment variables can override all file settings
    pub fn load() -> Result<Self, ConfigError> {
        // Step 1: Determine environment (from shell/docker or default to development)
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Step 2: Load base .env file (common settings for all environments)
        if let Err(e) = dotenv::dotenv() {
            // .env file not found is OK - not all deployments use it
            tracing::debug!("Base .env file not found: {}", e);
        }

        // Step 3: Load environment-specific .env file (overrides base settings)
        let env_file = format!(".env.{}", environment);
        if let Err(e) = dotenv::from_filename(&env_file) {
            tracing::debug!("Environment-specific file {} not found: {}", env_file, e);
        }

        // Step 4: Re-read environment variable (might have been set in .env files)
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config = ConfigBuilder::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3001)?
            .set_default("server.environment", environment.as_str())?
            // MongoDB defaults
            .set_default("database.mongodb.url", "localhost")?
            .set_default("database.mongodb.port", 27017)?
            .set_default("database.mongodb.username", "admin")?
            .set_default("database.mongodb.password", "admin123")?
            .set_default("database.mongodb.database", "llm_hub_dev")?
            .set_default("database.mongodb.auth_source", "admin")?
            .set_default("database.mongodb.connection_timeout_ms", 10000)?
            .set_default("database.mongodb.max_pool_size", 10)?
            .set_default("database.mongodb.min_pool_size", 1)?
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

        let mut config: Config = config.try_deserialize()?;

        // COMPATIBILITY: Support Backend's encryption key variable name
        // Check for PROJECT_API_KEY_ENCRYPTION_KEY (Backend variable name)
        // This takes precedence over AI_GATEWAY_SECURITY_ENCRYPTION_KEY
        if let Ok(backend_key) = env::var("PROJECT_API_KEY_ENCRYPTION_KEY") {
            config.security.encryption_key = backend_key;
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate encryption key
        if self.security.encryption_key.is_empty() {
            return Err("Encryption key is required".to_string());
        }

        // Validate encryption key is base64 and correct length
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::STANDARD;
        match engine.decode(&self.security.encryption_key) {
            Ok(decoded) => {
                if decoded.len() != 32 {
                    return Err(format!(
                        "Encryption key must be 32 bytes (256 bits), got {} bytes",
                        decoded.len()
                    ));
                }
            }
            Err(_) => {
                return Err("Encryption key must be valid base64".to_string());
            }
        }

        // Validate MongoDB configuration
        if self.database.mongodb.url.is_empty() {
            return Err("MongoDB host is required".to_string());
        }

        if self.database.mongodb.port == 0 {
            return Err("MongoDB port must be greater than 0".to_string());
        }

        if self.database.mongodb.username.is_empty() {
            return Err("MongoDB username is required".to_string());
        }

        if self.database.mongodb.password.is_empty() {
            return Err("MongoDB password is required".to_string());
        }

        if self.database.mongodb.database.is_empty() {
            return Err("MongoDB database name is required".to_string());
        }

        if self.database.mongodb.auth_source.is_empty() {
            return Err("MongoDB auth source is required".to_string());
        }

        if self.database.mongodb.max_pool_size == 0 {
            return Err("MongoDB max_pool_size must be greater than 0".to_string());
        }

        if self.database.mongodb.min_pool_size > self.database.mongodb.max_pool_size {
            return Err("MongoDB min_pool_size cannot exceed max_pool_size".to_string());
        }

        // Validate server port
        if self.server.port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }

        Ok(())
    }
}