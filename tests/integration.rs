use assert_cmd::Command;
use predicates::prelude::*;

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
            .stdout(predicate::str::contains("status"));
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
            .stdout(predicate::str::contains("status"));
    }
}

mod isolation_tests {
    #[test]
    #[ignore = "Epic 8, Story 8.1 - requires full implementation"]
    fn test_two_profiles_dont_share_config() {
        unimplemented!("Critical isolation test - validates core mechanism");
    }
}
