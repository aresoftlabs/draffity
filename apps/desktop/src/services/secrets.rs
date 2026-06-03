//! Secret storage abstraction (E-01). BYOK API keys never touch the plain-text
//! `settings` table — they live in the OS keyring
//! (Windows Credential Manager / macOS Keychain / Linux Secret Service).
//!
//! `KeyringSecretStorage` is the real impl. `InMemorySecretStorage` is the
//! test/fallback impl: unit tests must never poke the real OS keychain, and a
//! headless environment without a Secret Service degrades to in-memory rather
//! than crashing. The trait is the contract the rest of the app depends on.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

/// Logical service name under which all Draffity secrets are grouped in the
/// OS keychain. The `key` becomes the per-account identifier.
const KEYRING_SERVICE: &str = "draffity";

pub trait SecretStorage: Send + Sync {
    /// Returns the stored secret, or `None` if absent.
    fn get_secret(&self, key: &str) -> AppResult<Option<String>>;
    /// Stores (or overwrites) the secret.
    fn set_secret(&self, key: &str, value: &str) -> AppResult<()>;
    /// Removes the secret. Removing an absent key is a no-op (Ok).
    fn delete_secret(&self, key: &str) -> AppResult<()>;
}

/// OS keyring-backed storage. Each secret is one credential entry under the
/// shared `draffity` service.
pub struct KeyringSecretStorage;

impl KeyringSecretStorage {
    pub fn new() -> Self {
        Self
    }

    fn entry(key: &str) -> AppResult<keyring::Entry> {
        keyring::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| AppError::Unexpected(format!("keyring open: {e}")))
    }
}

impl Default for KeyringSecretStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStorage for KeyringSecretStorage {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>> {
        match Self::entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Unexpected(format!("keyring get: {e}"))),
        }
    }

    fn set_secret(&self, key: &str, value: &str) -> AppResult<()> {
        Self::entry(key)?
            .set_password(value)
            .map_err(|e| AppError::Unexpected(format!("keyring set: {e}")))
    }

    fn delete_secret(&self, key: &str) -> AppResult<()> {
        match Self::entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Unexpected(format!("keyring delete: {e}"))),
        }
    }
}

/// In-memory storage for tests and headless fallback. Never persisted.
#[derive(Default)]
pub struct InMemorySecretStorage {
    map: Mutex<HashMap<String, String>>,
}

impl InMemorySecretStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStorage for InMemorySecretStorage {
    fn get_secret(&self, key: &str) -> AppResult<Option<String>> {
        Ok(self
            .map
            .lock()
            .map_err(|_| AppError::Unexpected("secret store poisoned".into()))?
            .get(key)
            .cloned())
    }

    fn set_secret(&self, key: &str, value: &str) -> AppResult<()> {
        self.map
            .lock()
            .map_err(|_| AppError::Unexpected("secret store poisoned".into()))?
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete_secret(&self, key: &str) -> AppResult<()> {
        self.map
            .lock()
            .map_err(|_| AppError::Unexpected("secret store poisoned".into()))?
            .remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_round_trips_and_deletes() {
        let s = InMemorySecretStorage::new();
        assert_eq!(s.get_secret("k").unwrap(), None);
        s.set_secret("k", "v").unwrap();
        assert_eq!(s.get_secret("k").unwrap(), Some("v".into()));
        // Overwrite.
        s.set_secret("k", "v2").unwrap();
        assert_eq!(s.get_secret("k").unwrap(), Some("v2".into()));
        // Delete is idempotent.
        s.delete_secret("k").unwrap();
        s.delete_secret("k").unwrap();
        assert_eq!(s.get_secret("k").unwrap(), None);
    }
}
