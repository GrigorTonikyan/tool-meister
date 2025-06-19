use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{tempdir, TempDir};

fn create_isolated_command() -> Command {
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("tool-meister").unwrap();

    // Use a temporary directory for tests to avoid interference
    cmd.env("XDG_CONFIG_HOME", temp_dir.path())
        .env("HOME", temp_dir.path());

    // Keep the temp_dir alive by storing it in an environment variable
    // This is a hack, but it works for integration tests
    cmd.env("_TEST_TEMP_DIR", temp_dir.path());

    // Leak the temp_dir to keep it alive for the test duration
    std::mem::forget(temp_dir);

    cmd
}

// Helper for tests that need to share state across multiple commands
fn with_shared_test_env<F>(test_fn: F)
where
    F: FnOnce(&TempDir) -> (),
{
    let temp_dir = tempdir().unwrap();
    test_fn(&temp_dir);
}

fn create_command_with_env(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tool-meister").unwrap();

    cmd.env("XDG_CONFIG_HOME", temp_dir.path())
        .env("HOME", temp_dir.path());

    cmd
}

#[test]
fn test_manifests_help() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage manifest sources"))
        .stdout(predicate::str::contains("add-source"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn test_manifests_add_source_help() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests").arg("add-source").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Add a manifest source"))
        .stdout(predicate::str::contains("--source-type"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--no-auto-update"));
}

#[test]
fn test_manifests_add_source_nonexistent_path() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("local")
        .arg("/nonexistent/path");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Path does not exist"));
}

#[test]
fn test_manifests_add_source_file_instead_of_directory() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test").unwrap();

    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("local")
        .arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("must be a directory"));
}

#[test]
fn test_manifests_add_source_valid_local_directory() {
    let temp_dir = tempdir().unwrap();
    let manifest_dir = temp_dir.path().join("manifests");
    fs::create_dir_all(&manifest_dir).unwrap();

    // Create a test manifest
    let test_manifest = r#"{
  "repo": {
    "name": "test-tool",
    "url": "https://github.com/example/test.git"
  },
  "actions": {
    "installation": [{"type": "git_clone", "url": "{{ repo.url }}"}],
    "update": [{"type": "git_pull"}],
    "build": [{"type": "shell", "command": "cargo build"}],
    "run": [{"type": "shell", "command": "cargo run"}]
  }
}"#;
    fs::write(manifest_dir.join("test-tool.jsonc"), test_manifest).unwrap();

    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("local")
        .arg(manifest_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✅ Added manifest source: local"))
        .stdout(predicate::str::contains(manifest_dir.to_str().unwrap()));
}

#[test]
fn test_manifests_add_source_invalid_git_url() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("git")
        .arg("invalid-url");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("must be a valid git URL"));
}

#[test]
fn test_manifests_add_source_valid_git_url() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("git")
        .arg("--branch")
        .arg("main")
        .arg("https://github.com/example/manifests.git");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Added manifest source: git https://github.com/example/manifests.git",
        ))
        .stdout(predicate::str::contains("(branch: main)"));
}

#[test]
fn test_manifests_add_source_invalid_url() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("url")
        .arg("ftp://example.com/manifests");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("must be a valid HTTP/HTTPS URL"));
}

#[test]
fn test_manifests_add_source_valid_url() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("url")
        .arg("--no-auto-update")
        .arg("https://example.com/manifests");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Added manifest source: url https://example.com/manifests",
        ))
        .stdout(predicate::str::contains("without auto-update"));
}

#[test]
fn test_manifests_add_source_invalid_type() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("invalid")
        .arg("/some/path");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid source type"));
}

#[test]
fn test_manifests_list_empty() {
    // Reset config first
    let mut reset_cmd = create_isolated_command();
    reset_cmd.arg("config").arg("--reset");
    reset_cmd.assert().success();

    let mut cmd = create_isolated_command();
    cmd.arg("manifests").arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configured manifest sources:"))
        .stdout(predicate::str::contains("local")); // Should have at least the default source
}

#[test]
fn test_manifests_info_with_existing_manifests() {
    with_shared_test_env(|config_dir| {
        let temp_dir = tempdir().unwrap();
        let manifest_dir = temp_dir.path().join("manifests");
        fs::create_dir_all(&manifest_dir).unwrap();

        // Create test manifests
        let test_manifest1 = r#"{"repo": {"name": "tool1"}, "actions": {}}"#;
        let test_manifest2 = r#"{"repo": {"name": "tool2"}, "actions": {}}"#;

        fs::write(manifest_dir.join("tool1.jsonc"), test_manifest1).unwrap();
        fs::write(manifest_dir.join("tool2.jsonc"), test_manifest2).unwrap();

        // Add the manifest source
        let mut add_cmd = create_command_with_env(config_dir);
        add_cmd
            .arg("manifests")
            .arg("add-source")
            .arg("--source-type")
            .arg("local")
            .arg(manifest_dir.to_str().unwrap());
        add_cmd.assert().success();

        // Test info command
        let mut cmd = create_command_with_env(config_dir);
        cmd.arg("manifests").arg("info");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Manifest source information:"))
            .stdout(predicate::str::contains("Available manifests:"))
            .stdout(predicate::str::contains("tool1"))
            .stdout(predicate::str::contains("tool2"));
    });
}

#[test]
fn test_manifests_info_nonexistent_directory() {
    let mut cmd = create_isolated_command();
    cmd.arg("manifests").arg("info");

    cmd.assert().success().stdout(
        predicate::str::contains("Directory not found")
            .or(predicate::str::contains("Available manifests:")),
    );
}

#[test]
fn test_config_show_uses_cargo_metadata() {
    let mut cmd = create_isolated_command();
    cmd.arg("config").arg("--show");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Current app configuration:"))
        .stdout(predicate::str::contains("tools_dir"))
        .stdout(predicate::str::contains("manifest_sources"));
}

#[test]
fn test_config_reset() {
    let mut cmd = create_isolated_command();
    cmd.arg("config").arg("--reset");

    cmd.assert().success().stdout(predicate::str::contains(
        "✅ App configuration reset to defaults",
    ));
}

#[test]
fn test_main_help_shows_manifests_command() {
    let mut cmd = create_isolated_command();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("manifests"))
        .stdout(predicate::str::contains("Manage manifest sources"));
}

#[test]
fn test_relative_path_resolution() {
    let temp_dir = tempdir().unwrap();
    let manifest_dir = temp_dir.path().join("manifests");
    fs::create_dir_all(&manifest_dir).unwrap();

    // Change to the temp directory to test relative paths
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let mut cmd = create_isolated_command();
    cmd.arg("manifests")
        .arg("add-source")
        .arg("--source-type")
        .arg("local")
        .arg("manifests"); // relative path

    let output = cmd.assert().success();

    // The output should show the absolute path, not the relative one
    output
        .stdout(predicate::str::contains("✅ Added manifest source: local"))
        .stdout(predicate::str::contains(manifest_dir.to_str().unwrap()));

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_duplicate_source_detection() {
    with_shared_test_env(|config_dir| {
        let temp_dir = tempdir().unwrap();
        let manifest_dir = temp_dir.path().join("manifests");
        fs::create_dir_all(&manifest_dir).unwrap();

        // Add source first time
        let mut cmd1 = create_command_with_env(config_dir);
        cmd1.arg("manifests")
            .arg("add-source")
            .arg("--source-type")
            .arg("local")
            .arg(manifest_dir.to_str().unwrap());
        cmd1.assert().success();

        // Try to add same source again
        let mut cmd2 = create_command_with_env(config_dir);
        cmd2.arg("manifests")
            .arg("add-source")
            .arg("--source-type")
            .arg("local")
            .arg(manifest_dir.to_str().unwrap());

        cmd2.assert()
            .failure()
            .stderr(predicate::str::contains("already exists"));
    });
}
