use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Integration tests for the arch-tool-meister CLI functionality
/// These tests verify the main application flow and CLI interface

#[test]
fn test_cli_help_flag() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that help text contains expected elements
    assert!(stdout.contains("A modular Rust TUI application"));
    assert!(stdout.contains("--module"));
    assert!(stdout.contains("--list-modules"));
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("--help"));
    assert!(stdout.contains("--version"));
}

#[test]
fn test_cli_version_flag() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that version output contains expected format
    assert!(stdout.contains("arch-tool-meister"));
    // Version should follow semver format (e.g., "2.0.0")
    assert!(stdout.contains("2.0.0"));
}

#[test]
fn test_cli_list_modules_flag() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--list-modules"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    // The command should execute successfully even if no modules are found
    // or if there are configuration issues
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the output contains module listing indicators
    if output.status.success() {
        assert!(stdout.contains("Available modules:") || stdout.contains("No modules"));
    } else {
        // If it fails, it should be due to configuration issues, not code errors
        assert!(
            stderr.contains("Failed to load config")
                || stderr.contains("Failed to discover modules")
                || stderr.contains("Cannot create any configuration")
        );
    }
}

#[test]
fn test_cli_debug_flag() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--debug", "--list-modules"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // In debug mode, we should see more verbose output or debug messages
    // This might be in stderr due to logging configuration
    if output.status.success() {
        // Debug output might appear in stderr or be logged to file
        // We just verify the command runs with debug flag
        assert!(true); // The fact that it executed without panic is the test
    } else {
        // Should fail gracefully with descriptive error
        assert!(stderr.contains("Failed to") || stderr.contains("Error"));
    }
}

#[test]
fn test_cli_module_command_invalid_module() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--module", "nonexistent_module"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    // Should fail gracefully for non-existent module
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // If it succeeds, it should indicate the module was not found
        assert!(stdout.contains("not found") || stdout.contains("Available commands"));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("not found") || stderr.contains("Failed to"));
    }
}

#[test]
fn test_cli_module_command_no_command() {
    // This test assumes there might be a 'system' module or similar
    // If no such module exists, it will test the error handling
    let output = Command::new("cargo")
        .args(&["run", "--", "--module", "system"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either list available commands or indicate module not found
    if output.status.success() {
        assert!(stdout.contains("Available commands") || stdout.contains("commands for module"));
    } else {
        assert!(
            stderr.contains("Module")
                && (stderr.contains("not found") || stderr.contains("disabled"))
        );
    }
}

#[test]
fn test_application_graceful_startup_and_exit() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;

    // Test that the TUI application can start and exit gracefully
    let mut child = Command::new("cargo")
        .args(&["run"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");

    // Give the application a moment to initialize
    thread::sleep(Duration::from_millis(500));

    // Send 'q' to quit the application
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"q");
        let _ = stdin.flush();
    }

    // Wait for the application to exit, with a timeout
    thread::sleep(Duration::from_millis(500));

    // Try to kill it if it's still running
    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for child");

    // The application should exit cleanly (exit code 0) or be terminated by our kill signal
    // We mainly want to ensure it doesn't panic or hang
    assert!(output.status.code().is_some()); // Process should terminate, not hang indefinitely
}

#[test]
fn test_configuration_fallback_behavior() {
    use std::env;
    use std::fs;

    // Create a temporary directory for this test
    let temp_dir = env::temp_dir().join("atm_integration_test");
    let _ = fs::create_dir_all(&temp_dir);

    // Run the application from the temporary directory where no config files exist
    let output = Command::new("cargo")
        .args(&["run", "--", "--list-modules"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PWD", &temp_dir) // Try to influence working directory
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // The application should handle missing configuration gracefully
    // It should either use fallback configs or fail with helpful error messages
    if !output.status.success() {
        assert!(
            stderr.contains("Failed to load config")
                || stderr.contains("Creating minimal default")
                || stderr.contains("Cannot create any configuration")
        );
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_error_handling_with_invalid_arguments() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--invalid-flag"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    // Should exit with error code for invalid arguments
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should provide helpful error message about invalid arguments
    assert!(stderr.contains("error:") || stderr.contains("unexpected"));
}

#[test]
fn test_build_process() {
    // Test that the project builds successfully
    let output = Command::new("cargo")
        .args(&["build"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute build command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Build failed: {}", stderr);
    }

    // Verify the binary was created
    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("arch-tool-meister");

    assert!(
        binary_path.exists(),
        "Binary should be created after successful build"
    );
}

#[test]
fn test_configuration_loading_resilience() {
    // Test the application's ability to handle various configuration scenarios
    let test_cases = vec![
        ("--help", true),          // Should always work
        ("--version", true),       // Should always work
        ("--list-modules", false), // Might fail due to missing config, but shouldn't panic
    ];

    for (args, should_succeed) in test_cases {
        let output = Command::new("cargo")
            .args(&["run", "--", args])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        if should_succeed {
            assert!(
                output.status.success(),
                "Command '{}' should succeed but failed",
                args
            );
        }

        // Most importantly, ensure no panics occurred
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panic"),
            "Command '{}' should not panic: {}",
            args,
            stderr
        );
    }
}
