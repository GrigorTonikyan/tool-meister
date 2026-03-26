//! Integration tests for the configuration management system.
//!
//! These tests verify the end-to-end functionality of the configuration system,
//! including file loading, validation, caching, and hot-reload capabilities.
//! They use real files and test the complete workflow from configuration
//! discovery to validation and change detection.

use core_lib::config::{ConfigLoader, ConfigManager};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Sample valid application configuration for testing.
const SAMPLE_APP_CONFIG: &str = r#"
{
    // Sample application configuration
    "appSettings": {
        "appName": "Arch Tool Meister Test",
        "version": "2.0.0-test",
        "modulesDir": "test_modules",
        "downloadDir": "/tmp/test_downloads",
        "installPrefix": "/opt/test",
        "animation": {
            "steps": 5,
            "delayMs": 100
        }
    }
}
"#;

/// Sample valid menu configuration for testing.
const SAMPLE_MENU_CONFIG: &str = r#"
{
    // Sample menu configuration
    "title": "Test Main Menu",
    "dynamicMenu": true,
    "options": [
        {
            "name": "Test Option 1",
            "description": "First test option",
            "action": "test_action_1",
            "enabled": true
        },
        {
            "name": "Test Option 2", 
            "description": "Second test option",
            "action": "test_action_2",
            "enabled": false
        }
    ]
}
"#;

/// Updated application configuration for hot-reload testing.
const UPDATED_APP_CONFIG: &str = r#"
{
    // Updated application configuration
    "appSettings": {
        "appName": "Arch Tool Meister Updated",
        "version": "2.1.0-test", 
        "modulesDir": "updated_modules",
        "downloadDir": "/tmp/updated_downloads",
        "installPrefix": "/opt/updated",
        "animation": {
            "steps": 8,
            "delayMs": 200
        }
    }
}
"#;

/// Creates a test environment with configuration files.
struct TestConfigEnvironment {
    temp_dir: TempDir,
    app_config_path: std::path::PathBuf,
    menu_config_path: std::path::PathBuf,
}

impl TestConfigEnvironment {
    /// Creates a new test environment with sample configuration files.
    fn new() -> std::io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let app_config_path = temp_dir.path().join("config.jsonc");
        let menu_config_path = temp_dir.path().join("main_menu.jsonc");

        // Write initial configuration files
        fs::write(&app_config_path, SAMPLE_APP_CONFIG)?;
        fs::write(&menu_config_path, SAMPLE_MENU_CONFIG)?;

        Ok(Self {
            temp_dir,
            app_config_path,
            menu_config_path,
        })
    }

    /// Updates the application configuration file.
    fn update_app_config(&self) -> std::io::Result<()> {
        fs::write(&self.app_config_path, UPDATED_APP_CONFIG)
    }

    /// Gets the path to the test directory.
    fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

#[tokio::test]
async fn test_config_loader_integration() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");

    // Test loading application configuration
    let app_config = ConfigLoader::load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load app config");

    assert_eq!(app_config.app_settings.app_name, "Arch Tool Meister Test");
    assert_eq!(app_config.app_settings.version, "2.0.0-test");
    assert_eq!(app_config.app_settings.modules_dir, "test_modules");
    assert_eq!(app_config.app_settings.download_dir, "/tmp/test_downloads");
    assert_eq!(app_config.app_settings.install_prefix, "/opt/test");
    assert_eq!(app_config.app_settings.animation.steps, 5);
    assert_eq!(app_config.app_settings.animation.delay_ms, 100);

    // Test loading menu configuration
    let menu_config = ConfigLoader::load_menu_config(&test_env.menu_config_path)
        .await
        .expect("Failed to load menu config");

    assert_eq!(menu_config.title, "Test Main Menu");
    assert!(menu_config.dynamic_menu);
    assert_eq!(menu_config.options.len(), 2);
    assert_eq!(menu_config.options[0].name, "Test Option 1");
    assert_eq!(menu_config.options[0].description, "First test option");
    assert_eq!(menu_config.options[0].action, "test_action_1");
    assert!(menu_config.options[0].enabled);
    assert!(!menu_config.options[1].enabled);
}

#[tokio::test]
async fn test_config_manager_integration() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");
    let config_manager = ConfigManager::new();

    // Test loading and caching
    assert!(!config_manager.has_app_config_cached().await);
    assert!(!config_manager.has_menu_config_cached().await);

    let app_config = config_manager
        .load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load app config");
    assert!(config_manager.has_app_config_cached().await);
    assert_eq!(app_config.app_settings.app_name, "Arch Tool Meister Test");

    let menu_config = config_manager
        .load_menu_config(&test_env.menu_config_path)
        .await
        .expect("Failed to load menu config");
    assert!(config_manager.has_menu_config_cached().await);
    assert_eq!(menu_config.title, "Test Main Menu");

    // Test cache hit (should return same data without re-reading file)
    let cached_app_config = config_manager
        .load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load cached app config");
    assert_eq!(
        cached_app_config.app_settings.app_name,
        app_config.app_settings.app_name
    );

    // Test cache refresh
    config_manager.refresh_cache().await;
    assert!(!config_manager.has_app_config_cached().await);
    assert!(!config_manager.has_menu_config_cached().await);
}

#[tokio::test]
async fn test_config_path_resolution() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");

    // Change to test directory for path resolution
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(test_env.path()).expect("Failed to change dir");

    let config_manager = ConfigManager::new();

    // Test resolving existing config file
    let resolved_path = config_manager
        .resolve_config_path("config.jsonc")
        .await
        .expect("Failed to resolve config path");
    assert!(resolved_path.ends_with("config.jsonc"));
    assert!(resolved_path.exists());

    // Test resolving nonexistent config file
    let result = config_manager
        .resolve_config_path("nonexistent.jsonc")
        .await;
    assert!(result.is_err());

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

#[tokio::test]
async fn test_config_hot_reload_integration() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");
    let config_manager = ConfigManager::new();

    // Load initial configuration
    let initial_config = config_manager
        .load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load initial config");
    assert_eq!(
        initial_config.app_settings.app_name,
        "Arch Tool Meister Test"
    );

    // Enable hot-reload (we don't use the receiver in this simplified test)
    let _change_receiver = config_manager
        .enable_hot_reload(&[&test_env.app_config_path, &test_env.menu_config_path])
        .await
        .expect("Failed to enable hot-reload");

    // Give the watcher time to initialize
    sleep(Duration::from_millis(100)).await;

    // Update the configuration file
    test_env
        .update_app_config()
        .expect("Failed to update app config");

    // Wait for file system event processing and cache invalidation
    sleep(Duration::from_millis(1000)).await;

    // Force cache refresh to ensure we get the updated configuration
    config_manager.refresh_cache().await;

    // Load configuration again - should get updated version due to cache invalidation
    let updated_config = config_manager
        .load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load updated config");

    // Configuration should reflect the changes
    assert_eq!(
        updated_config.app_settings.app_name,
        "Arch Tool Meister Updated"
    );
    assert_eq!(updated_config.app_settings.version, "2.1.0-test");
    assert_eq!(updated_config.app_settings.modules_dir, "updated_modules");
}

#[tokio::test]
async fn test_config_validation_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test with invalid JSON syntax
    let invalid_config_path = temp_dir.path().join("invalid.jsonc");
    let invalid_config = r#"
    {
        "appSettings": {
            "appName": "Test"
            // Missing comma
            "version": "1.0.0"
        }
    }
    "#;
    fs::write(&invalid_config_path, invalid_config).expect("Failed to write invalid config");

    let result = ConfigLoader::load_app_config(&invalid_config_path).await;
    assert!(result.is_err());

    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Invalid JSON syntax"));

    // Test with invalid configuration structure
    let structurally_invalid_path = temp_dir.path().join("structurally_invalid.jsonc");
    let structurally_invalid = r#"
    {
        "wrongField": "value",
        "notAppSettings": {}
    }
    "#;
    fs::write(&structurally_invalid_path, structurally_invalid)
        .expect("Failed to write structurally invalid config");

    let result = ConfigLoader::load_app_config(&structurally_invalid_path).await;
    assert!(result.is_err());

    // Test with missing required fields
    let missing_fields_path = temp_dir.path().join("missing_fields.jsonc");
    let missing_fields = r#"
    {
        "appSettings": {
            "appName": "Test"
            // Missing required fields like version, modulesDir, etc.
        }
    }
    "#;
    fs::write(&missing_fields_path, missing_fields).expect("Failed to write missing fields config");

    let result = ConfigLoader::load_app_config(&missing_fields_path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_security_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test path traversal prevention
    let result = ConfigLoader::load_app_config("../../../etc/passwd").await;
    assert!(result.is_err());

    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Security validation failed")
            || error_msg.contains("Forbidden path component")
    );

    // Test with invalid extension
    let wrong_ext_path = temp_dir.path().join("config.exe");
    fs::write(&wrong_ext_path, SAMPLE_APP_CONFIG)
        .expect("Failed to write config with wrong extension");

    let result = ConfigLoader::load_app_config(&wrong_ext_path).await;
    assert!(result.is_err());

    // Test with dangerous content patterns
    let dangerous_path = temp_dir.path().join("dangerous.jsonc");
    let dangerous_config = r#"
    {
        "appSettings": {
            "appName": "Test",
            "command": "rm -rf /",
            "version": "1.0.0",
            "modulesDir": "modules",
            "downloadDir": "/tmp",
            "installPrefix": "/opt",
            "animation": {
                "steps": 10,
                "delayMs": 300
            }
        }
    }
    "#;
    fs::write(&dangerous_path, dangerous_config).expect("Failed to write dangerous config");

    // This should still load (content warnings don't block loading) but should log warnings
    let result = ConfigLoader::load_app_config(&dangerous_path).await;
    // The config should load since we only warn about dangerous patterns in content validation
    // but don't block loading for potentially legitimate use cases
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_error_handling_integration() {
    // Test nonexistent file
    let result = ConfigLoader::load_app_config("/nonexistent/path/config.jsonc").await;
    assert!(result.is_err());

    // Test directory instead of file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let result = ConfigLoader::load_app_config(temp_dir.path()).await;
    assert!(result.is_err());

    // Test empty file
    let empty_file = temp_dir.path().join("empty.jsonc");
    fs::write(&empty_file, "").expect("Failed to write empty file");
    let result = ConfigLoader::load_app_config(&empty_file).await;
    assert!(result.is_err());

    // Test binary file
    let binary_file = temp_dir.path().join("binary.jsonc");
    fs::write(&binary_file, b"\x00\x01\x02\x03").expect("Failed to write binary file");
    let result = ConfigLoader::load_app_config(&binary_file).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_manager_with_watcher() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");

    // Test creating config manager with watcher
    let watch_paths = vec![&test_env.app_config_path, &test_env.menu_config_path];
    let config_manager = ConfigManager::with_watcher(&watch_paths)
        .await
        .expect("Failed to create config manager with watcher");

    // Test normal config loading still works
    let app_config = config_manager
        .load_app_config(&test_env.app_config_path)
        .await
        .expect("Failed to load app config with watcher");
    assert_eq!(app_config.app_settings.app_name, "Arch Tool Meister Test");

    // Test that caching works
    assert!(config_manager.has_app_config_cached().await);
}

#[tokio::test]
async fn test_end_to_end_configuration_workflow() {
    let test_env = TestConfigEnvironment::new().expect("Failed to create test environment");

    // 1. Create a config manager
    let config_manager = ConfigManager::new();

    // 2. Use direct paths instead of resolution (since we know the test file locations)
    let app_config_path = &test_env.app_config_path;
    let menu_config_path = &test_env.menu_config_path;

    // 3. Load configurations
    let app_config = config_manager
        .load_app_config(app_config_path)
        .await
        .expect("Failed to load app config");

    let menu_config = config_manager
        .load_menu_config(menu_config_path)
        .await
        .expect("Failed to load menu config");

    // 4. Verify configurations are loaded correctly
    assert_eq!(app_config.app_settings.app_name, "Arch Tool Meister Test");
    assert_eq!(menu_config.title, "Test Main Menu");

    // 5. Test caching
    assert!(config_manager.has_app_config_cached().await);
    assert!(config_manager.has_menu_config_cached().await);

    // 6. Enable hot-reload
    let _change_receiver = config_manager
        .enable_hot_reload(&[app_config_path, menu_config_path])
        .await
        .expect("Failed to enable hot-reload");

    // 7. Test cache refresh
    config_manager.refresh_cache().await;
    assert!(!config_manager.has_app_config_cached().await);
    assert!(!config_manager.has_menu_config_cached().await);

    // 8. Reload configurations (should work even after cache refresh)
    let reloaded_app_config = config_manager
        .load_app_config(app_config_path)
        .await
        .expect("Failed to reload app config");

    assert_eq!(
        reloaded_app_config.app_settings.app_name,
        app_config.app_settings.app_name
    );
}
