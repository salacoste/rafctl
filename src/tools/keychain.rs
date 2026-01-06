use std::process::Command;

use crate::error::RafctlError;

const RAFCTL_SERVICE_PREFIX: &str = "rafctl-profile-";
const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

pub fn read_oauth_token(profile_name: &str) -> Result<Option<String>, RafctlError> {
    let service = format!("{}{}", RAFCTL_SERVICE_PREFIX, profile_name);
    read_keychain_password(&service)
}

pub fn save_oauth_token(profile_name: &str, token: &str) -> Result<(), RafctlError> {
    let service = format!("{}{}", RAFCTL_SERVICE_PREFIX, profile_name);
    let account = whoami::username();

    delete_keychain_password(&service).ok();

    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            &service,
            "-a",
            &account,
            "-w",
            token,
            "-U",
        ])
        .output()
        .map_err(|e| {
            RafctlError::KeychainError(format!("Failed to run security command: {}", e))
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(RafctlError::KeychainError(format!(
            "Failed to save token: {}",
            stderr
        )))
    }
}

pub fn delete_oauth_token(profile_name: &str) -> Result<(), RafctlError> {
    let service = format!("{}{}", RAFCTL_SERVICE_PREFIX, profile_name);
    delete_keychain_password(&service)
}

pub fn read_claude_keychain() -> Result<Option<String>, RafctlError> {
    read_keychain_password(CLAUDE_KEYCHAIN_SERVICE)
}

pub fn swap_to_claude_keychain(token: &str) -> Result<(), RafctlError> {
    let account = whoami::username();

    delete_keychain_password(CLAUDE_KEYCHAIN_SERVICE).ok();

    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            CLAUDE_KEYCHAIN_SERVICE,
            "-a",
            &account,
            "-w",
            token,
            "-U",
        ])
        .output()
        .map_err(|e| {
            RafctlError::KeychainError(format!("Failed to run security command: {}", e))
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(RafctlError::KeychainError(format!(
            "Failed to swap keychain: {}",
            stderr
        )))
    }
}

pub fn capture_oauth_from_claude(profile_name: &str) -> Result<(), RafctlError> {
    match read_claude_keychain()? {
        Some(token) => {
            save_oauth_token(profile_name, &token)?;
            Ok(())
        }
        None => Err(RafctlError::KeychainError(
            "No Claude OAuth token found in keychain".to_string(),
        )),
    }
}

fn read_keychain_password(service: &str) -> Result<Option<String>, RafctlError> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .map_err(|e| {
            RafctlError::KeychainError(format!("Failed to run security command: {}", e))
        })?;

    if output.status.success() {
        let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if password.is_empty() {
            Ok(None)
        } else {
            Ok(Some(password))
        }
    } else {
        Ok(None)
    }
}

fn delete_keychain_password(service: &str) -> Result<(), RafctlError> {
    let output = Command::new("security")
        .args(["delete-generic-password", "-s", service])
        .output()
        .map_err(|e| {
            RafctlError::KeychainError(format!("Failed to run security command: {}", e))
        })?;

    if output.status.success() || output.status.code() == Some(44) {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(RafctlError::KeychainError(format!(
            "Failed to delete keychain entry: {}",
            stderr
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_format() {
        let service = format!("{}{}", RAFCTL_SERVICE_PREFIX, "work");
        assert_eq!(service, "rafctl-profile-work");
    }

    #[test]
    fn test_claude_service_constant() {
        assert_eq!(CLAUDE_KEYCHAIN_SERVICE, "Claude Code-credentials");
    }
}
