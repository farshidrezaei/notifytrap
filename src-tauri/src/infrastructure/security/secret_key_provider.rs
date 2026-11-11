use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::TryRng;
use rand::rngs::SysRng;

use crate::domain::errors::ApplicationError;

const MASTER_KEY_BYTE_LENGTH: usize = 32;
const KEYRING_SERVICE_NAME: &str = "notifytrap";
const KEYRING_ACCOUNT_NAME: &str = "master-encryption-key";
const FALLBACK_KEY_FILENAME: &str = "master.key";

/// Provides the 256-bit local master encryption key used for channel credentials.
pub trait SecretKeyProvider: Send + Sync {
    fn get_or_create_master_key(&self) -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError>;
}

/// In-memory random key provider for isolated tests without touching OS keychains.
#[derive(Default)]
pub struct EphemeralSecretKeyProvider {
    cached_key: Mutex<Option<[u8; MASTER_KEY_BYTE_LENGTH]>>,
}

impl EphemeralSecretKeyProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretKeyProvider for EphemeralSecretKeyProvider {
    fn get_or_create_master_key(&self) -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError> {
        let mut guard = self
            .cached_key
            .lock()
            .map_err(|_| ApplicationError::Internal("Key provider mutex poisoned".to_string()))?;

        if let Some(existing_key) = *guard {
            return Ok(existing_key);
        }

        let generated = generate_master_key_bytes()?;
        *guard = Some(generated);
        Ok(generated)
    }
}

/// Production key provider backed by the OS keyring with an owner-only file fallback.
pub struct DesktopSecretKeyProvider {
    fallback_directory: PathBuf,
    cached_key: Mutex<Option<[u8; MASTER_KEY_BYTE_LENGTH]>>,
}

impl DesktopSecretKeyProvider {
    pub fn new(fallback_directory: PathBuf) -> Self {
        Self {
            fallback_directory,
            cached_key: Mutex::new(None),
        }
    }

    fn try_read_from_keyring() -> Result<Option<[u8; MASTER_KEY_BYTE_LENGTH]>, ApplicationError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, KEYRING_ACCOUNT_NAME)
            .map_err(|error| ApplicationError::Internal(format!("Keyring init failed: {error}")))?;

        match entry.get_password() {
            Ok(encoded) => decode_key_base64(&encoded).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(ApplicationError::Internal(format!(
                "Keyring unavailable: {error}"
            ))),
        }
    }

    fn try_write_to_keyring(key: &[u8; MASTER_KEY_BYTE_LENGTH]) -> Result<(), ApplicationError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, KEYRING_ACCOUNT_NAME)
            .map_err(|error| ApplicationError::Internal(format!("Keyring init failed: {error}")))?;
        let encoded = STANDARD.encode(key);
        entry
            .set_password(&encoded)
            .map_err(|error| ApplicationError::Internal(format!("Keyring store failed: {error}")))
    }

    fn try_read_existing_fallback_file_key(
        &self,
    ) -> Result<Option<[u8; MASTER_KEY_BYTE_LENGTH]>, ApplicationError> {
        let key_path = self.fallback_directory.join(FALLBACK_KEY_FILENAME);
        if !key_path.exists() {
            return Ok(None);
        }

        let encoded = fs::read_to_string(&key_path).map_err(|error| {
            ApplicationError::Internal(format!("Unable to read master key file: {error}"))
        })?;
        decode_key_base64(encoded.trim()).map(Some)
    }

    fn get_or_create_fallback_file_key(
        &self,
    ) -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError> {
        if let Some(existing_key) = self.try_read_existing_fallback_file_key()? {
            return Ok(existing_key);
        }

        let key_path = self.fallback_directory.join(FALLBACK_KEY_FILENAME);
        fs::create_dir_all(&self.fallback_directory).map_err(|error| {
            ApplicationError::Internal(format!("Unable to create key directory: {error}"))
        })?;

        let generated = generate_master_key_bytes()?;
        let encoded = STANDARD.encode(generated);
        write_restricted_key_file(&key_path, &encoded)?;

        Ok(generated)
    }
}

fn write_restricted_key_file(
    path: &std::path::Path,
    contents: &str,
) -> Result<(), ApplicationError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|error| {
                ApplicationError::Internal(format!("Unable to create master key file: {error}"))
            })?;
        file.write_all(contents.as_bytes()).map_err(|error| {
            ApplicationError::Internal(format!("Unable to persist master key file: {error}"))
        })?;
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents).map_err(|error| {
            ApplicationError::Internal(format!("Unable to persist master key file: {error}"))
        })?;
    }

    Ok(())
}

impl SecretKeyProvider for DesktopSecretKeyProvider {
    fn get_or_create_master_key(&self) -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError> {
        let mut guard = self
            .cached_key
            .lock()
            .map_err(|_| ApplicationError::Internal("Key provider mutex poisoned".to_string()))?;

        if let Some(existing_key) = *guard {
            return Ok(existing_key);
        }

        let resolved = match Self::try_read_from_keyring() {
            Ok(Some(keyring_key)) => keyring_key,
            Ok(None) => {
                if let Some(existing_fallback_key) = self.try_read_existing_fallback_file_key()? {
                    if let Err(keyring_error) = Self::try_write_to_keyring(&existing_fallback_key) {
                        tracing::warn!(
                            "Could not migrate existing key file into OS keyring: {keyring_error}"
                        );
                    }
                    existing_fallback_key
                } else {
                    let generated = generate_master_key_bytes()?;
                    if let Err(keyring_error) = Self::try_write_to_keyring(&generated) {
                        tracing::warn!(
                            "OS keyring storage failed, using restricted key file: {keyring_error}"
                        );
                        self.get_or_create_fallback_file_key()?
                    } else {
                        generated
                    }
                }
            }
            Err(keyring_error) => {
                tracing::warn!(
                    "OS keyring read unavailable, using restricted key file: {keyring_error}"
                );
                self.get_or_create_fallback_file_key()?
            }
        };

        *guard = Some(resolved);
        Ok(resolved)
    }
}

fn generate_master_key_bytes() -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError> {
    let mut key = [0u8; MASTER_KEY_BYTE_LENGTH];
    SysRng
        .try_fill_bytes(&mut key)
        .map_err(|error| ApplicationError::Internal(format!("CSPRNG entropy failure: {error}")))?;
    Ok(key)
}

fn decode_key_base64(encoded: &str) -> Result<[u8; MASTER_KEY_BYTE_LENGTH], ApplicationError> {
    let decoded = STANDARD.decode(encoded).map_err(|error| {
        ApplicationError::Internal(format!("Corrupted master key encoding: {error}"))
    })?;
    decoded
        .try_into()
        .map_err(|_| ApplicationError::Internal("Master key has invalid byte length".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fallback_master_key_persists_across_provider_restarts() {
        let directory = tempdir().expect("temporary directory should be created");
        let first_provider = DesktopSecretKeyProvider::new(directory.path().to_path_buf());

        assert_eq!(
            first_provider
                .try_read_existing_fallback_file_key()
                .expect("reading missing key should succeed"),
            None
        );

        let created_key = first_provider
            .get_or_create_fallback_file_key()
            .expect("fallback key creation should succeed");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(directory.path().join(FALLBACK_KEY_FILENAME))
                .expect("key file metadata should exist");
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        }

        let restarted_provider = DesktopSecretKeyProvider::new(directory.path().to_path_buf());
        let reloaded_key = restarted_provider
            .try_read_existing_fallback_file_key()
            .expect("reloading key should succeed")
            .expect("persisted key should exist on restart");

        assert_eq!(created_key, reloaded_key);
    }
}
