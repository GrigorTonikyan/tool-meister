use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("up-man").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Universal Package Manager Updater",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("up-man").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_validate_nonexistent_config() {
    // Create a temp dir but with a non-existent config path that we'll use
    let temp_dir = tempdir().unwrap();

    // Create a subdirectory to avoid default config creation
    let custom_config_dir = temp_dir.path().join("custom");
    std::fs::create_dir_all(&custom_config_dir).unwrap();

    let mut cmd = Command::cargo_bin("up-man").unwrap();
    cmd.arg("validate").env("HOME", custom_config_dir); // Set HOME to our custom dir

    // Should warn that config doesn't exist, but create a default one
    cmd.assert()
        .success() // It should succeed by creating a default config
        .stderr(predicate::str::contains(
            "Created default configuration file",
        ));
}

#[test]
fn test_detect_command() {
    let mut cmd = Command::cargo_bin("up-man").unwrap();
    cmd.arg("detect");

    // Should run without errors
    cmd.assert().success().stderr(predicate::str::contains(
        "Detecting available package managers",
    ));
}

// TODO: Add tests for commands that require a valid config file
// TODO: Add tests for running updates (with mocked package managers)
// TODO: Add tests for backup functionality
// TODO: Add tests for alias setup
