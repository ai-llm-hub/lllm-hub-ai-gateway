use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::shared::error::AppError;

/// AES-256-GCM Encryption service
#[derive(Clone)]
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Create new encryption service with the given key
    pub fn new(base64_key: &str) -> Result<Self, AppError> {
        let key_bytes = BASE64.decode(base64_key)
            .map_err(|e| AppError::ConfigError(format!("Invalid encryption key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(AppError::ConfigError(
                "Encryption key must be 32 bytes (256 bits)".to_string(),
            ));
        }

        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| AppError::ConfigError("Failed to convert key to array".to_string()))?;

        let cipher = Aes256Gcm::new(&key_array.into());

        Ok(Self { cipher })
    }

    /// Encrypt plaintext
    pub fn encrypt(&self, plaintext: &str) -> Result<String, AppError> {
        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Combine nonce and ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        // Return base64 encoded
        Ok(BASE64.encode(&combined))
    }

    /// Decrypt ciphertext
    pub fn decrypt(&self, encrypted: &str) -> Result<String, AppError> {
        // Decode from base64
        let combined = BASE64.decode(encrypted)
            .map_err(|e| AppError::EncryptionError(format!("Invalid ciphertext: {}", e)))?;

        if combined.len() < 12 {
            return Err(AppError::EncryptionError(
                "Ciphertext too short".to_string(),
            ));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::EncryptionError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::EncryptionError(format!("Invalid UTF-8: {}", e)))
    }

    /// Generate a new encryption key
    pub fn generate_key() -> String {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        BASE64.encode(&key)
    }
}

/// Hash service for API keys and passwords
#[allow(dead_code)]
pub struct HashService;

#[allow(dead_code)]
impl HashService {
    /// Hash an API key using bcrypt
    pub fn hash_api_key(api_key: &str) -> Result<String, AppError> {
        bcrypt::hash(api_key, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash API key: {}", e)))
    }

    /// Verify an API key against a hash
    pub fn verify_api_key(api_key: &str, hash: &str) -> Result<bool, AppError> {
        bcrypt::verify(api_key, hash)
            .map_err(|e| AppError::InternalError(format!("Failed to verify API key: {}", e)))
    }

    /// Generate SHA-256 hash
    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}