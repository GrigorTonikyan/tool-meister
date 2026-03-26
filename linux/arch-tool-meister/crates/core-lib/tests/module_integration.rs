//! Integration tests for the module management system.
//!
//! These tests use temporary directories and realistic module configurations
//! to test the complete module discovery, validation, and registry functionality.

use core_lib::config::ConfigManager;
use core_lib::module_manager::{
    discovery::ModuleDiscovery, registry::ModuleRegistry, ModuleManager,
};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::sync::RwLock;

/// Create a temporary directory structure with test modules
async fn create_test_modules_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir_all(&modules_dir).await.unwrap();

    // Create a valid test module
    let test_module_dir = modules_dir.join("test_module");
    fs::create_dir_all(&test_module_dir).await.unwrap();

    let valid_config = r#"{
    "name": "test_module",
    "description": "A test module for integration testing",
    "version": "1.0.0",
    "enabled": true,
    "settings": {
        "test_setting": "test_value"
    },
    "menu": {
        "title": "Test Module",
        "mainMenuEntry": "Test Module",
        "options": [
            {
                "text": "Run Test Command",
                "type": "scriptFunction",
                "functionName": "test_cmd"
            },
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {
        "test_cmd": {
            "description": "Test command",
            "function": "test_function",
            "args": ["--verbose"]
        }
    },
    "functions": {
        "test_function": {
            "code": "echo 'Hello from test module!'"
        }
    }
}"#;

    let config_path = test_module_dir.join("module.jsonc");
    fs::write(config_path, valid_config).await.unwrap();

    // Create another test module
    let another_module_dir = modules_dir.join("another_module");
    fs::create_dir_all(&another_module_dir).await.unwrap();

    let another_config = r#"{
    "name": "another_module",
    "description": "Another test module",
    "version": "2.1.0",
    "enabled": false,
    "settings": {},
    "menu": {
        "title": "Another Module",
        "mainMenuEntry": "Another Module",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {},
    "functions": {}
}"#;

    let another_config_path = another_module_dir.join("module.jsonc");
    fs::write(another_config_path, another_config)
        .await
        .unwrap();

    // Create a module with invalid configuration (dangerous command)
    let invalid_module_dir = modules_dir.join("invalid_module");
    fs::create_dir_all(&invalid_module_dir).await.unwrap();

    let invalid_config = r#"{
    "name": "invalid_module",
    "description": "Invalid module with dangerous commands",
    "version": "1.0.0",
    "enabled": true,
    "settings": {},
    "menu": {
        "title": "Invalid Module",
        "mainMenuEntry": "Invalid",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {
        "dangerous_cmd": {
            "description": "Dangerous command",
            "function": "dangerous_function",
            "args": []
        }
    },
    "functions": {
        "dangerous_function": {
            "code": "rm -rf /"
        }
    }
}"#;

    let invalid_config_path = invalid_module_dir.join("module.jsonc");
    fs::write(invalid_config_path, invalid_config)
        .await
        .unwrap();

    temp_dir
}

#[tokio::test]
async fn test_module_discovery_integration() {
    let temp_dir = create_test_modules_dir().await;
    let modules_dir = temp_dir.path().join("modules");

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    // Should discover 2 valid modules (invalid one should be filtered out)
    assert_eq!(discovered.len(), 2);
    assert!(discovered.contains_key("test_module"));
    assert!(discovered.contains_key("another_module"));

    // Validate test module
    let test_module = &discovered["test_module"];
    assert_eq!(test_module.name, "test_module");
    assert_eq!(test_module.version, "1.0.0");
    assert!(test_module.enabled);
    assert!(test_module.settings.contains_key("test_setting"));
    assert_eq!(test_module.commands.len(), 1);
    assert_eq!(test_module.functions.len(), 1);

    // Validate another module
    let another_module = &discovered["another_module"];
    assert_eq!(another_module.name, "another_module");
    assert_eq!(another_module.version, "2.1.0");
    assert!(!another_module.enabled);
    assert!(another_module.commands.is_empty());
    assert!(another_module.functions.is_empty());
}

#[tokio::test]
async fn test_module_registry_integration() {
    let temp_dir = create_test_modules_dir().await;
    let modules_dir = temp_dir.path().join("modules");

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    let registry = ModuleRegistry::new();
    registry.update_registry(discovered).await.unwrap();

    // Test registry operations
    let all_modules = registry.list_modules().await;
    assert_eq!(all_modules.len(), 2);

    let test_module = registry.get_module("test_module").await;
    assert!(test_module.is_some());

    let enabled_modules = registry.get_enabled_modules().await;
    assert_eq!(enabled_modules.len(), 1);
    assert_eq!(enabled_modules[0].name, "test_module");

    let stats = registry.get_stats().await;
    assert_eq!(stats.total_modules, 2);
    assert_eq!(stats.enabled_modules, 1);
    assert_eq!(stats.disabled_modules, 1);

    // Test filtering
    let filtered = registry
        .filter_modules(|m| m.version.starts_with('2'))
        .await;
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "another_module");
}

#[tokio::test]
async fn test_module_manager_integration() {
    let temp_dir = create_test_modules_dir().await;
    let modules_dir = temp_dir.path().join("modules");

    // Create a config manager (mock)
    let config_manager = Arc::new(RwLock::new(ConfigManager::new()));

    // Create module manager
    let manager = ModuleManager::new(config_manager);

    // For testing, we need to create a separate discovery and registry
    // and use their results to test the manager's functionality
    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    let registry = ModuleRegistry::new();
    registry.update_registry(discovered).await.unwrap();

    // Test high-level operations by comparing with expected results
    // Note: In a real scenario, we would call manager.initialize() with proper config

    // For now, test that the manager can be created and basic methods work
    let modules = manager.list_modules().await.unwrap();
    // Initially empty because we haven't called initialize() with proper paths
    assert_eq!(modules.len(), 0);

    let test_module = manager.get_module("test_module").await.unwrap();
    assert!(test_module.is_none());

    let enabled = manager.get_enabled_modules().await.unwrap();
    assert_eq!(enabled.len(), 0);

    let search_results = manager.search_modules("test").await.unwrap();
    assert_eq!(search_results.len(), 0);

    let has_module = manager.has_module("test_module").await.unwrap();
    assert!(!has_module);

    let stats = manager.get_stats().await.unwrap();
    assert_eq!(stats.total_modules, 0);

    let validation_issues = manager.validate_registry().await.unwrap();
    assert!(validation_issues.is_empty());

    // Test that manager methods execute without error
    let module_names = manager.get_module_names().await.unwrap();
    assert_eq!(module_names.len(), 0);

    let dependents = manager.find_dependents("nonexistent").await.unwrap();
    assert_eq!(dependents.len(), 0);
}
#[tokio::test]
async fn test_jsonc_comment_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir_all(&modules_dir).await.unwrap();

    let test_module_dir = modules_dir.join("jsonc_test");
    fs::create_dir_all(&test_module_dir).await.unwrap();

    // Create a module config with JSONC comments
    let jsonc_config = r#"{
    // This is a test module with comments
    "name": "jsonc_test",
    "description": "Module with JSONC comments", // inline comment
    "version": "1.0.0",
    "enabled": true,
    "settings": {
        // Settings section
        "debug": true // debug flag
    },
    "menu": {
        "title": "JSONC Test Module",
        "mainMenuEntry": "JSONC Test",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return" // menu type
            }
        ]
    },
    "commands": {},
    "functions": {}
}"#;

    let config_path = test_module_dir.join("module.jsonc");
    fs::write(config_path, jsonc_config).await.unwrap();

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    assert_eq!(discovered.len(), 1);
    let module = &discovered["jsonc_test"];
    assert_eq!(module.name, "jsonc_test");
    assert_eq!(module.description, "Module with JSONC comments");
    assert!(module.settings.contains_key("debug"));
}

#[tokio::test]
async fn test_module_validation_security() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir_all(&modules_dir).await.unwrap();

    // Create a module with dangerous commands
    let dangerous_module_dir = modules_dir.join("dangerous_module");
    fs::create_dir_all(&dangerous_module_dir).await.unwrap();

    let dangerous_config = r#"{
    "name": "dangerous_module",
    "description": "Module with dangerous commands",
    "version": "1.0.0",
    "enabled": true,
    "settings": {},
    "menu": {
        "title": "Dangerous Module",
        "mainMenuEntry": "Dangerous",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {
        "dangerous_cmd": {
            "description": "Dangerous command",
            "function": "dangerous_function",
            "args": []
        }
    },
    "functions": {
        "dangerous_function": {
            "code": "rm -rf /"
        }
    }
}"#;

    let config_path = dangerous_module_dir.join("module.jsonc");
    fs::write(config_path, dangerous_config).await.unwrap();

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    // Should be empty because dangerous module was rejected by security validation
    assert_eq!(discovered.len(), 0);
}

#[tokio::test]
async fn test_module_command_function_validation() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir_all(&modules_dir).await.unwrap();

    let invalid_ref_module_dir = modules_dir.join("invalid_ref_module");
    fs::create_dir_all(&invalid_ref_module_dir).await.unwrap();

    // Create a module where command references non-existent function
    let invalid_ref_config = r#"{
    "name": "invalid_ref_module",
    "description": "Module with invalid function reference",
    "version": "1.0.0",
    "enabled": true,
    "settings": {},
    "menu": {
        "title": "Invalid Ref Module",
        "mainMenuEntry": "Invalid Ref",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {
        "test_cmd": {
            "description": "Test command",
            "function": "non_existent_function",
            "args": []
        }
    },
    "functions": {
        "existing_function": {
            "code": "echo 'This function exists'"
        }
    }
}"#;

    let config_path = invalid_ref_module_dir.join("module.jsonc");
    fs::write(config_path, invalid_ref_config).await.unwrap();

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    // Should be empty because module has invalid function reference
    assert_eq!(discovered.len(), 0);
}

#[tokio::test]
async fn test_recursive_directory_scanning() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");

    // Create nested directory structure
    let category_dir = modules_dir.join("category");
    let subcategory_dir = category_dir.join("subcategory");
    let nested_module_dir = subcategory_dir.join("nested_module");
    fs::create_dir_all(&nested_module_dir).await.unwrap();

    let nested_config = r#"{
    "name": "nested_module",
    "description": "Module in nested directory",
    "version": "1.0.0",
    "enabled": true,
    "settings": {},
    "menu": {
        "title": "Nested Module",
        "mainMenuEntry": "Nested",
        "options": [
            {
                "text": "Return to Main Menu",
                "type": "return"
            }
        ]
    },
    "commands": {},
    "functions": {}
}"#;

    let config_path = nested_module_dir.join("module.jsonc");
    fs::write(config_path, nested_config).await.unwrap();

    let discovery = ModuleDiscovery::with_directories(vec![modules_dir]);
    let discovered = discovery.discover_all_modules().await.unwrap();

    assert_eq!(discovered.len(), 1);
    assert!(discovered.contains_key("nested_module"));
}
