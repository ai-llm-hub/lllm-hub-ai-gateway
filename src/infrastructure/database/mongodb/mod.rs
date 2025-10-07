pub mod llm_api_key_repo;
pub mod project_repo;
pub mod transcription_repo;
pub mod usage_repo;

use mongodb::{Client, Database};

pub use llm_api_key_repo::MongoLlmApiKeyRepository;
pub use project_repo::MongoProjectRepository;
pub use transcription_repo::MongoTranscriptionRepository;
pub use usage_repo::MongoUsageRepository;

/// Connect to MongoDB and return database handle
pub async fn connect_mongodb(url: &str, database_name: &str) -> Result<Database, mongodb::error::Error> {
    let client = Client::with_uri_str(url).await?;
    Ok(client.database(database_name))
}