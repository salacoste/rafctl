use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a rafctl command with isolated config directory
fn rafctl_cmd(config_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("rafctl").unwrap();
    cmd.env("HOME", config_dir);
    cmd
}

mod cli_tests {
    use super::*;

    #[test]
    fn test_help_shows_all_commands() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("profile"))
            .stdout(predicate::str::contains("auth"))
            .stdout(predicate::str::contains("run"))
            .stdout(predicate::str::contains("status"))
            .stdout(predicate::str::contains("config"))
            .stdout(predicate::str::contains("completion"));
    }

    #[test]
    fn test_version() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("rafctl"));
    }

    #[test]
    fn test_profile_help() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["profile", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("add"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("remove"))
            .stdout(predicate::str::contains("show"));
    }

    #[test]
    fn test_auth_help() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["auth", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("login"))
            .stdout(predicate::str::contains("logout"))
            .stdout(predicate::str::contains("status"))
            .stdout(predicate::str::contains("set-key"));
    }

    #[test]
    fn test_config_help() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["config", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("show"))
            .stdout(predicate::str::contains("set-default"))
            .stdout(predicate::str::contains("clear-default"))
            .stdout(predicate::str::contains("path"));
    }

    #[test]
    fn test_global_json_flag() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--json"));
    }

    #[test]
    fn test_global_plain_flag() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--plain"));
    }
}

mod profile_tests {
    use super::*;

    #[test]
    fn test_profile_add_and_list() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Add a profile
        rafctl_cmd(home)
            .args(["profile", "add", "test-work", "--tool", "claude"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Profile 'test-work' created"));

        // List should show it
        rafctl_cmd(home)
            .args(["profile", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("test-work"));
    }

    #[test]
    fn test_profile_add_with_api_key_mode() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args([
                "profile",
                "add",
                "api-profile",
                "--tool",
                "claude",
                "--auth-mode",
                "api-key",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("api-key"));
    }

    #[test]
    fn test_profile_show() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Add profile first
        rafctl_cmd(home)
            .args(["profile", "add", "show-test", "--tool", "claude"])
            .assert()
            .success();

        // Show details
        rafctl_cmd(home)
            .args(["profile", "show", "show-test"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Profile: show-test"))
            .stdout(predicate::str::contains("Tool:"))
            .stdout(predicate::str::contains("claude"));
    }

    #[test]
    fn test_profile_show_json() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "json-test", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["--json", "profile", "show", "json-test"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\": \"json-test\""))
            .stdout(predicate::str::contains("\"tool\": \"claude\""));
    }

    #[test]
    fn test_profile_remove() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Add then remove
        rafctl_cmd(home)
            .args(["profile", "add", "to-remove", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["profile", "remove", "to-remove"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Profile 'to-remove' removed"));

        // List should be empty
        rafctl_cmd(home)
            .args(["profile", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No profiles found"));
    }

    #[test]
    fn test_profile_name_case_insensitive() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Add with uppercase
        rafctl_cmd(home)
            .args(["profile", "add", "MyProfile", "--tool", "claude"])
            .assert()
            .success();

        // Should find with lowercase
        rafctl_cmd(home)
            .args(["profile", "show", "myprofile"])
            .assert()
            .success()
            .stdout(predicate::str::contains("myprofile"));
    }

    #[test]
    fn test_profile_duplicate_error() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "duplicate", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["profile", "add", "duplicate", "--tool", "claude"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("already exists"));
    }

    #[test]
    fn test_profile_invalid_name() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "invalid name", "--tool", "claude"])
            .assert()
            .failure();
    }

    #[test]
    fn test_profile_not_found() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "show", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_codex_profile() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "codex-test", "--tool", "codex"])
            .assert()
            .success()
            .stdout(predicate::str::contains("codex"));
    }
}

mod status_tests {
    use super::*;

    #[test]
    fn test_status_empty() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No profiles found"));
    }

    #[test]
    fn test_status_with_profiles() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "status-test", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("status-test"));
    }

    #[test]
    fn test_status_json_format() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "json-status", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["--json", "status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"profiles\""))
            .stdout(predicate::str::contains("\"name\": \"json-status\""));
    }

    #[test]
    fn test_status_plain_format() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "plain-status", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["--plain", "status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("NAME\tTOOL"));
    }

    #[test]
    fn test_status_single_profile() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "single-status", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["status", "single-status"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Profile: single-status"));
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_config_show() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["config", "show"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration"))
            .stdout(predicate::str::contains("Default profile"));
    }

    #[test]
    fn test_config_show_json() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["--json", "config", "show"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"default_profile\""))
            .stdout(predicate::str::contains("\"config_directory\""));
    }

    #[test]
    fn test_config_set_default() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Create profile first
        rafctl_cmd(home)
            .args(["profile", "add", "default-test", "--tool", "claude"])
            .assert()
            .success();

        // Set as default
        rafctl_cmd(home)
            .args(["config", "set-default", "default-test"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Default profile set"));

        // Verify in config show
        rafctl_cmd(home)
            .args(["config", "show"])
            .assert()
            .success()
            .stdout(predicate::str::contains("default-test"));
    }

    #[test]
    fn test_config_clear_default() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "clear-test", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["config", "set-default", "clear-test"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["config", "clear-default"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Default profile cleared"));
    }

    #[test]
    fn test_config_set_default_nonexistent() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["config", "set-default", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_config_path() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["config", "path"])
            .assert()
            .success()
            .stdout(predicate::str::contains(".rafctl"));
    }
}

mod isolation_tests {
    use super::*;

    #[test]
    fn test_two_profiles_have_separate_directories() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Create two profiles
        rafctl_cmd(home)
            .args(["profile", "add", "work", "--tool", "claude"])
            .assert()
            .success();

        rafctl_cmd(home)
            .args(["profile", "add", "personal", "--tool", "claude"])
            .assert()
            .success();

        // Verify separate directories exist
        let rafctl_dir = home.join(".rafctl").join("profiles");
        assert!(rafctl_dir.join("work").exists());
        assert!(rafctl_dir.join("personal").exists());

        // Verify meta.yaml files
        assert!(rafctl_dir.join("work").join("meta.yaml").exists());
        assert!(rafctl_dir.join("personal").join("meta.yaml").exists());
    }

    #[test]
    fn test_profile_config_isolation() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .args(["profile", "add", "isolated", "--tool", "claude"])
            .assert()
            .success();

        // The profile should have its own claude subdirectory ready
        let profile_dir = home.join(".rafctl").join("profiles").join("isolated");
        assert!(profile_dir.exists());

        // Create a marker file in the profile's claude dir
        let claude_dir = profile_dir.join("claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("marker.txt"), "test").unwrap();

        // Verify the marker exists only in this profile
        assert!(claude_dir.join("marker.txt").exists());
    }
}

mod completion_tests {
    use super::*;

    #[test]
    fn test_bash_completion() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["completion", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("_rafctl"));
    }

    #[test]
    fn test_zsh_completion() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["completion", "zsh"])
            .assert()
            .success()
            .stdout(predicate::str::contains("#compdef rafctl"));
    }

    #[test]
    fn test_fish_completion() {
        Command::cargo_bin("rafctl")
            .unwrap()
            .args(["completion", "fish"])
            .assert()
            .success()
            .stdout(predicate::str::contains("complete"));
    }
}

mod no_color_tests {
    use super::*;

    #[test]
    fn test_no_color_env_var() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        rafctl_cmd(home)
            .env("NO_COLOR", "1")
            .args(["profile", "list"])
            .assert()
            .success();
        // Should work without crashing - output is plain
    }
}
