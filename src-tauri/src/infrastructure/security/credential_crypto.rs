use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use rand::TryRng;
use rand::rngs::SysRng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::domain::channel::ChannelType;
use crate::domain::errors::ApplicationError;
use crate::domain::validation::validate_slug;

const AES_GCM_NONCE_BYTES: usize = 12;
const GENERATED_SECRET_BYTES: usize = 24;
const GENERATED_IDENTIFIER_BYTES: usize = 8;
const SECRET_TOKEN_PREFIX: &str = "ntsec_";

/// Newly generated credential identifier and secret material before persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedChannelCredential {
    pub credential_identifier: String,
    pub secret_token: String,
    pub composite_key: String,
    pub credential_hash: String,
    pub encrypted_credential: String,
}

/// Generates a random credential pair, hashes the secret with SHA-256, and encrypts it with AES-256-GCM.
pub fn generate_channel_credential(
    channel_type: ChannelType,
    custom_identifier: Option<&str>,
    master_key: &[u8; 32],
) -> Result<GeneratedChannelCredential, ApplicationError> {
    let credential_identifier = match custom_identifier {
        Some(explicit) if !explicit.trim().is_empty() => validate_slug(explicit)?,
        _ => generate_random_identifier(channel_type)?,
    };

    let secret_token = generate_secret_token()?;
    let composite_key = format!("{credential_identifier}.{secret_token}");
    let credential_hash = hash_secret_hex(&secret_token);
    let encrypted_credential = encrypt_secret(master_key, &secret_token)?;

    Ok(GeneratedChannelCredential {
        credential_identifier,
        secret_token,
        composite_key,
        credential_hash,
        encrypted_credential,
    })
}

/// Computes the lowercase hexadecimal SHA-256 digest of a raw secret token.
pub fn hash_secret_hex(raw_secret: &str) -> String {
    let digest = Sha256::digest(raw_secret.as_bytes());
    hex::encode(digest)
}

/// Verifies a candidate secret against a stored SHA-256 hex digest in constant time.
pub fn verify_secret_constant_time(raw_secret: &str, expected_hash_hex: &str) -> bool {
    let Ok(expected_digest) = hex::decode(expected_hash_hex) else {
        return false;
    };
    if expected_digest.len() != 32 {
        return false;
    }
    let computed_digest = Sha256::digest(raw_secret.as_bytes());
    bool::from(computed_digest.as_slice().ct_eq(expected_digest.as_slice()))
}

/// Parses `<credential_identifier>.<secret_token>` from headers or query parameters.
pub fn parse_composite_channel_key(composite_key: &str) -> Option<(&str, &str)> {
    let trimmed = composite_key.trim();
    let (identifier, secret) = trimmed.split_once('.')?;
    if identifier.is_empty() || secret.is_empty() {
        return None;
    }
    Some((identifier, secret))
}

/// Encrypts plaintext secret material using AES-256-GCM and a fresh CSPRNG nonce.
pub fn encrypt_secret(master_key: &[u8; 32], plaintext: &str) -> Result<String, ApplicationError> {
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_BYTES];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|error| ApplicationError::Internal(format!("CSPRNG nonce failure: {error}")))?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| ApplicationError::Internal("Failed to encrypt channel secret".to_string()))?;

    let mut envelope = Vec::with_capacity(AES_GCM_NONCE_BYTES + ciphertext.len());
    envelope.extend_from_slice(&nonce_bytes);
    envelope.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(envelope))
}

/// Decrypts AES-256-GCM envelope ciphertext produced by `encrypt_secret`.
pub fn decrypt_secret(master_key: &[u8; 32], encoded: &str) -> Result<String, ApplicationError> {
    let envelope = STANDARD.decode(encoded).map_err(|_| {
        ApplicationError::Internal("Invalid encrypted credential format".to_string())
    })?;

    if envelope.len() <= AES_GCM_NONCE_BYTES {
        return Err(ApplicationError::Internal(
            "Encrypted credential envelope is truncated".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = envelope.split_at(AES_GCM_NONCE_BYTES);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| ApplicationError::Internal("Failed to decrypt channel secret".to_string()))?;

    String::from_utf8(plaintext_bytes).map_err(|_| {
        ApplicationError::Internal("Decrypted credential is not valid UTF-8".to_string())
    })
}

fn generate_random_identifier(channel_type: ChannelType) -> Result<String, ApplicationError> {
    let mut random_bytes = [0u8; GENERATED_IDENTIFIER_BYTES];
    SysRng
        .try_fill_bytes(&mut random_bytes)
        .map_err(|error| ApplicationError::Internal(format!("CSPRNG failure: {error}")))?;

    Ok(format!(
        "{}-{}",
        channel_type.credential_prefix(),
        hex::encode(random_bytes)
    ))
}

fn generate_secret_token() -> Result<String, ApplicationError> {
    let mut random_bytes = [0u8; GENERATED_SECRET_BYTES];
    SysRng
        .try_fill_bytes(&mut random_bytes)
        .map_err(|error| ApplicationError::Internal(format!("CSPRNG failure: {error}")))?;

    Ok(format!(
        "{SECRET_TOKEN_PREFIX}{}",
        URL_SAFE_NO_PAD.encode(random_bytes)
    ))
}
