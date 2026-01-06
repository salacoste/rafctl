//! Cross-platform credential storage using the `keyring` crate.
//!
//! Supports:
//! - macOS: Keychain
//! - Linux: secret-service (libsecret)
//! - Windows: Windows Credential Manager

use crate::error::RafctlError;

const SERVICE_PREFIX: &str = "rafctl";

/// Credential types stored in the secure store
#[derive(Debug, Clone, Copy)]
pub enum CredentialType {
    /// OAuth token for a profile
    OAuthToken,
    /// API key for a profile
    ApiKey,
}

impl CredentialType {
    fn as_str(&self) -> &'static str {
        match self {
            CredentialType::OAuthToken => "oauth-token",
            CredentialType::ApiKey => "api-key",
        }
    }
}

/// Build the service name for keyring storage
fn build_service_name(profile_name: &str, cred_type: CredentialType) -> String {
    format!("{}-{}-{}", SERVICE_PREFIX, profile_name, cred_type.as_str())
}

/// Get the username for keyring (consistent across platforms)
fn get_username() -> String {
    whoami::username()
}

/// Store a credential securely
pub fn store_credential(
    profile_name: &str,
    cred_type: CredentialType,
    secret: &str,
) -> Result<(), RafctlError> {
    let service = build_service_name(profile_name, cred_type);
    let username = get_username();

    let entry = keyring::Entry::new(&service, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to create keyring entry: {}", e))
    })?;

    entry
        .set_password(secret)
        .map_err(|e| RafctlError::KeychainError(format!("Failed to store credential: {}", e)))?;

    Ok(())
}

/// Retrieve a credential from secure storage
pub fn get_credential(
    profile_name: &str,
    cred_type: CredentialType,
) -> Result<Option<String>, RafctlError> {
    let service = build_service_name(profile_name, cred_type);
    let username = get_username();

    let entry = keyring::Entry::new(&service, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to create keyring entry: {}", e))
    })?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(RafctlError::KeychainError(format!(
            "Failed to retrieve credential: {}",
            e
        ))),
    }
}

/// Delete a credential from secure storage
pub fn delete_credential(profile_name: &str, cred_type: CredentialType) -> Result<(), RafctlError> {
    let service = build_service_name(profile_name, cred_type);
    let username = get_username();

    let entry = keyring::Entry::new(&service, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to create keyring entry: {}", e))
    })?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted, that's fine
        Err(e) => Err(RafctlError::KeychainError(format!(
            "Failed to delete credential: {}",
            e
        ))),
    }
}

/// Check if a credential exists
pub fn has_credential(profile_name: &str, cred_type: CredentialType) -> Result<bool, RafctlError> {
    Ok(get_credential(profile_name, cred_type)?.is_some())
}

// ============================================================================
// Claude-specific OAuth token handling (for token swapping)
// ============================================================================

const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Read the current Claude Code OAuth token from system keychain
/// This is the token that Claude Code itself uses
pub fn read_claude_system_token() -> Result<Option<String>, RafctlError> {
    let username = get_username();

    let entry = keyring::Entry::new(CLAUDE_KEYCHAIN_SERVICE, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to access Claude keychain: {}", e))
    })?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(RafctlError::KeychainError(format!(
            "Failed to read Claude token: {}",
            e
        ))),
    }
}

/// Write a token to Claude Code's system keychain location
/// Used for OAuth token swapping
pub fn write_claude_system_token(token: &str) -> Result<(), RafctlError> {
    let username = get_username();

    // Delete existing entry first
    let entry = keyring::Entry::new(CLAUDE_KEYCHAIN_SERVICE, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to access Claude keychain: {}", e))
    })?;

    // Ignore errors on delete (might not exist)
    let _ = entry.delete_credential();

    // Create new entry and set password
    let entry = keyring::Entry::new(CLAUDE_KEYCHAIN_SERVICE, &username).map_err(|e| {
        RafctlError::KeychainError(format!("Failed to create Claude keychain entry: {}", e))
    })?;

    entry
        .set_password(token)
        .map_err(|e| RafctlError::KeychainError(format!("Failed to write Claude token: {}", e)))?;

    Ok(())
}

// ============================================================================
// Migration helpers
// ============================================================================

/// Migrate an API key from plaintext profile storage to secure keyring
pub fn migrate_api_key_to_keyring(profile_name: &str, api_key: &str) -> Result<(), RafctlError> {
    store_credential(profile_name, CredentialType::ApiKey, api_key)
}

/// Check if API key is configured (either in keyring or legacy plaintext)
/// This provides backwards compatibility during migration
pub fn has_api_key_configured(profile_name: &str, legacy_api_key: &Option<String>) -> bool {
    if legacy_api_key.is_some() {
        return true;
    }
    has_credential(profile_name, CredentialType::ApiKey).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_service_name() {
        assert_eq!(
            build_service_name("work", CredentialType::OAuthToken),
            "rafctl-work-oauth-token"
        );
        assert_eq!(
            build_service_name("personal", CredentialType::ApiKey),
            "rafctl-personal-api-key"
        );
    }

    #[test]
    fn test_credential_type_as_str() {
        assert_eq!(CredentialType::OAuthToken.as_str(), "oauth-token");
        assert_eq!(CredentialType::ApiKey.as_str(), "api-key");
    }
}
