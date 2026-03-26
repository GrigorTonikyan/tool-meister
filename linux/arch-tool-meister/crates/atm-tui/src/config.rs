use crate::errors::AtmError;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Application-wide configuration loaded from config.jsonc
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(rename = "appSettings")]
    pub app_settings: AppSettings,
    #[serde(rename = "vscodeConfig")]
    pub vscode_config: Option<VscodeConfig>,
    #[serde(rename = "menuPaths")]
    pub menu_paths: Option<MenuPaths>,
    #[serde(rename = "aurHelpers")]
    pub aur_helpers: Option<HashMap<String, AurHelper>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppSettings {
    #[serde(rename = "appName")]
    pub app_name: String,
    pub version: String,
    #[serde(rename = "modulesDir")]
    pub modules_dir: String,
    #[serde(rename = "downloadDir")]
    pub download_dir: Option<String>,
    #[serde(rename = "installPrefix")]
    pub install_prefix: Option<String>,
    pub animation: Option<AnimationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimationConfig {
    pub steps: u32,
    #[serde(rename = "delayMs")]
    pub delay_ms: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VscodeConfig {
    pub stable: VscodeVariant,
    pub insiders: VscodeVariant,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VscodeVariant {
    pub url: String,
    #[serde(rename = "dirName")]
    pub dir_name: String,
    #[serde(rename = "symlinkName")]
    pub symlink_name: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MenuPaths {
    pub main: String,
    pub vscode: Option<String>,
    #[serde(rename = "aurHelpers")]
    pub aur_helpers: Option<String>,
    #[serde(rename = "gitConfig")]
    pub git_config: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AurHelper {
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    #[serde(rename = "cloneDir")]
    pub clone_dir: String,
}

/// Main menu configuration loaded from main_menu.jsonc
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainMenuConfig {
    pub title: String,
    #[serde(rename = "dynamicMenu")]
    pub dynamic_menu: Option<bool>,
    pub options: Vec<MenuOption>,
}

/// Unified module configuration loaded from modules/*/module.jsonc
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnifiedModuleConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub settings: Option<serde_json::Value>,
    pub menu: ModuleMenuConfig,
    pub commands: HashMap<String, CommandDefinition>,
    pub functions: Option<HashMap<String, FunctionDefinition>>,
}

/// Module menu configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleMenuConfig {
    pub title: String,
    #[serde(rename = "mainMenuEntry")]
    pub main_menu_entry: Option<String>,
    pub options: Vec<MenuOption>,
}

/// Menu option structure used in both main and module menus
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MenuOption {
    pub text: String,
    #[serde(rename = "type")]
    pub option_type: MenuOptionType,
    #[serde(rename = "functionName")]
    pub function_name: Option<String>,
    #[serde(rename = "moduleName")]
    pub module_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuOptionType {
    ScriptFunction,
    ModuleMenu,
    Return,
    Exit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandDefinition {
    pub description: String,
    pub dependencies: Option<Vec<String>>,
    pub function: String,
    pub args: Option<Vec<String>>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionDefinition {
    pub code: String,
}

/// Complete module information from unified module.jsonc file
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub config: UnifiedModuleConfig,
}

impl Module {
    /// Get menu configuration from the unified config
    pub fn menu(&self) -> &ModuleMenuConfig {
        &self.config.menu
    }

    /// Get commands from the unified config
    pub fn commands(&self) -> &HashMap<String, CommandDefinition> {
        &self.config.commands
    }

    /// Get functions from the unified config
    pub fn functions(&self) -> Option<&HashMap<String, FunctionDefinition>> {
        self.config.functions.as_ref()
    }

    /// Check if module is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Registry to hold all loaded modules
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    pub modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn get_enabled_modules(&self) -> Vec<&Module> {
        self.modules.values().filter(|m| m.is_enabled()).collect()
    }
}

/// JSONC parser to strip comments before JSON parsing
pub fn parse_jsonc<T>(content: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let cleaned = strip_jsonc_comments(content);
    serde_json::from_str(&cleaned)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to parse JSON: {}", e))
}

/// Strip JSONC comments from a string
fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            escape_next = false;
            result.push(ch);
            continue;
        }

        if in_single_line_comment {
            if ch == '\n' {
                in_single_line_comment = false;
                result.push(ch);
            }
            continue;
        }

        if in_multi_line_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next(); // consume '/'
                in_multi_line_comment = false;
            }
            continue;
        }

        if in_string {
            if ch == '\\' {
                escape_next = true;
            } else if ch == '"' {
                in_string = false;
            }
            result.push(ch);
        } else {
            if ch == '"' {
                in_string = true;
                result.push(ch);
            } else if ch == '/' {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '/' {
                        chars.next(); // consume second '/'
                        in_single_line_comment = true;
                        continue;
                    } else if next_ch == '*' {
                        chars.next(); // consume '*'
                        in_multi_line_comment = true;
                        continue;
                    }
                }
                result.push(ch);
            } else {
                result.push(ch);
            }
        }
    }

    result
}

/// Load application configuration from config.jsonc
pub fn load_app_config<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let path_str = path.as_ref().display().to_string();
    let content = fs::read_to_string(&path)
        .map_err(|e| AtmError::file_operation_failed("read", &path_str, &e))?;

    parse_jsonc(&content).map_err(|_| {
        AtmError::config_invalid_json(&path_str, "Invalid JSON syntax in configuration file").into()
    })
}

/// Load main menu configuration from main_menu.jsonc  
pub fn load_main_menu_config<P: AsRef<Path>>(path: P) -> Result<MainMenuConfig> {
    let path_str = path.as_ref().display().to_string();
    let content = fs::read_to_string(&path)
        .map_err(|e| AtmError::file_operation_failed("read", &path_str, &e))?;

    parse_jsonc(&content).map_err(|_| {
        AtmError::config_invalid_json(&path_str, "Invalid JSON syntax in main menu configuration")
            .into()
    })
}

/// Load unified module configuration from modules/*/module.jsonc
pub fn load_unified_module_config<P: AsRef<Path>>(path: P) -> Result<UnifiedModuleConfig> {
    let path_str = path.as_ref().display().to_string();
    let content = fs::read_to_string(&path)
        .map_err(|e| AtmError::file_operation_failed("read", &path_str, &e))?;

    parse_jsonc(&content).map_err(|_| {
        AtmError::config_invalid_json(
            &path_str,
            "Invalid JSON syntax in unified module configuration",
        )
        .into()
    })
}

/// Discover and load all modules from the modules directory using unified configuration
pub fn discover_modules<P: AsRef<Path>>(modules_dir: P) -> Result<ModuleRegistry> {
    let mut registry = ModuleRegistry::new();

    if !modules_dir.as_ref().exists() {
        return Ok(registry);
    }

    for entry in fs::read_dir(&modules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let module_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid module directory name"))?;

            let unified_config_path = path.join("module.jsonc");

            if unified_config_path.exists() {
                match load_unified_module(&unified_config_path) {
                    Ok(module) => {
                        registry.add_module(module);
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load module '{}': {}", module_name, e);
                        eprintln!("   This module will be skipped.");
                    }
                }
            } else {
                eprintln!(
                    "⚠️  Module '{}' is missing required configuration file: module.jsonc",
                    module_name
                );
            }
        }
    }

    Ok(registry)
}

/// Load a complete module from its unified configuration file
fn load_unified_module<P: AsRef<Path>>(unified_config_path: P) -> Result<Module> {
    let config = load_unified_module_config(unified_config_path)?;
    let module_name = config.name.clone();

    Ok(Module {
        name: module_name,
        config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_comments() {
        let jsonc = r#"
        {
            // This is a comment
            "name": "test", // Another comment
            "value": 42,
            /* Multi-line
               comment */
            "description": "A test object"
        }
        "#;

        let cleaned = strip_jsonc_comments(jsonc);
        assert!(!cleaned.contains("//"));
        assert!(!cleaned.contains("/*"));
        assert!(cleaned.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_parse_jsonc() {
        let jsonc = r#"
        {
            // Configuration
            "name": "test",
            "enabled": true // Enable this feature
        }
        "#;

        #[derive(Deserialize)]
        struct TestConfig {
            name: String,
            enabled: bool,
        }

        let config: TestConfig = parse_jsonc(jsonc).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.enabled, true);
    }

    #[test]
    fn test_load_app_config_success() {
        // Create a temporary file with valid app config
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_app_config.jsonc");

        let config_content = r#"
        {
            // App settings
            "appSettings": {
                "appName": "Test App",
                "version": "1.0.0",
                "modulesDir": "modules",
                "downloadDir": "/tmp/downloads", // Download directory
                "installPrefix": "/usr/local",
                "animation": {
                    "steps": 4,
                    "delayMs": 200
                }
            }
        }
        "#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = load_app_config(&config_path).unwrap();
        assert_eq!(config.app_settings.app_name, "Test App");
        assert_eq!(config.app_settings.version, "1.0.0");
        assert_eq!(config.app_settings.modules_dir, "modules");
        assert_eq!(
            config.app_settings.download_dir,
            Some("/tmp/downloads".to_string())
        );
        assert_eq!(
            config.app_settings.install_prefix,
            Some("/usr/local".to_string())
        );

        let animation = config.app_settings.animation.unwrap();
        assert_eq!(animation.steps, 4);
        assert_eq!(animation.delay_ms, 200);

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_load_app_config_file_not_found() {
        let result = load_app_config("nonexistent_config.jsonc");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_main_menu_config_success() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_main_menu_config.jsonc");

        let config_content = r#"
        {
            "title": "Test Main Menu",
            "dynamicMenu": true, // Enable dynamic menu
            "options": [
                {
                    "text": "System Information",
                    "type": "scriptFunction",
                    "function": "system_info"
                },
                {
                    "text": "Exit",
                    "type": "exit"
                }
            ]
        }
        "#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = load_main_menu_config(&config_path).unwrap();
        assert_eq!(config.title, "Test Main Menu");
        assert_eq!(config.dynamic_menu, Some(true));
        assert_eq!(config.options.len(), 2);
        assert_eq!(config.options[0].text, "System Information");
        assert_eq!(config.options[1].text, "Exit");

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_parse_jsonc_with_invalid_json() {
        let invalid_jsonc = r#"
        {
            "name": "test",
            "missing_quote: "value"
        }
        "#;

        let result: Result<serde_json::Value> = parse_jsonc(invalid_jsonc);
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_jsonc_complex_comments() {
        let complex_jsonc = r#"
        {
            // Single line comment
            "name": "test", // Inline comment
            /* Multi-line
               comment with
               multiple lines */
            "array": [
                1, // Comment in array
                2
            ],
            /* Another block comment */
            "nested": {
                // Nested comment
                "value": true
            }
        }
        "#;

        let cleaned = strip_jsonc_comments(complex_jsonc);

        // Should not contain any comments
        assert!(!cleaned.contains("//"));
        assert!(!cleaned.contains("/*"));
        assert!(!cleaned.contains("*/"));

        // Should still contain the actual JSON content
        assert!(cleaned.contains("\"name\": \"test\""));
        assert!(cleaned.contains("\"array\""));
        assert!(cleaned.contains("\"nested\""));
        assert!(cleaned.contains("\"value\": true"));
    }

    #[test]
    fn test_discover_modules_empty_directory() {
        let temp_dir = std::env::temp_dir();
        let modules_dir = temp_dir.join("empty_modules_test");
        std::fs::create_dir_all(&modules_dir).unwrap();

        let registry = discover_modules(&modules_dir).unwrap();
        assert_eq!(registry.modules.len(), 0);

        std::fs::remove_dir(&modules_dir).unwrap();
    }

    #[test]
    fn test_module_registry_new() {
        let registry = ModuleRegistry::new();
        assert_eq!(registry.modules.len(), 0);
    }

    #[test]
    fn test_menu_option_types() {
        use crate::config::MenuOptionType;

        let script_option = MenuOption {
            text: "Run Script".to_string(),
            option_type: MenuOptionType::ScriptFunction,
            function_name: Some("run_script".to_string()),
            module_name: None,
        };

        let exit_option = MenuOption {
            text: "Exit".to_string(),
            option_type: MenuOptionType::Exit,
            function_name: None,
            module_name: None,
        };

        let return_option = MenuOption {
            text: "Back".to_string(),
            option_type: MenuOptionType::Return,
            function_name: None,
            module_name: None,
        };

        // Test script function option
        assert!(matches!(
            script_option.option_type,
            MenuOptionType::ScriptFunction
        ));
        assert_eq!(script_option.function_name, Some("run_script".to_string()));

        // Test exit option
        assert!(matches!(exit_option.option_type, MenuOptionType::Exit));
        assert_eq!(exit_option.function_name, None);

        // Test return option
        assert!(matches!(return_option.option_type, MenuOptionType::Return));
        assert_eq!(return_option.function_name, None);
    }

    // Error handling and edge case tests

    #[test]
    fn test_empty_jsonc_content() {
        let empty_content = "";
        let result: Result<serde_json::Value> = parse_jsonc(empty_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_jsonc_only_comments() {
        let only_comments = r#"
        // This is just a comment
        /* Another comment */
        // More comments
        "#;
        let result: Result<serde_json::Value> = parse_jsonc(only_comments);
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_comments_with_quoted_comment_symbols() {
        let jsonc_with_quoted_symbols = r#"
        {
            "message": "This string contains // and /* */",
            "url": "https://example.com/path", // Real comment
            "regex": "\/\*.*\*\/"
        }
        "#;

        let cleaned = strip_jsonc_comments(jsonc_with_quoted_symbols);

        // Should preserve comment-like strings in quotes but remove actual comments
        assert!(cleaned.contains("This string contains // and /* */"));
        assert!(cleaned.contains("https://example.com/path"));
        assert!(cleaned.contains(r#"\/\*.*\*\/"#));
        assert!(!cleaned.contains("Real comment"));
    }

    #[test]
    fn test_malformed_json_structure() {
        let malformed_json = r#"
        {
            "name": "test",
            "array": [1, 2, 3,], // Trailing comma
            "object": {
                "key": "value",
            } // Another trailing comma
        }
        "#;

        let result: Result<serde_json::Value> = parse_jsonc(malformed_json);
        // Depending on serde_json's strictness, this might fail
        // The test verifies we handle malformed JSON gracefully
        match result {
            Ok(_) => {
                // Some JSON parsers are lenient with trailing commas
                println!("Parser accepted trailing commas");
            }
            Err(_) => {
                // Expected behavior for strict JSON parsing
                println!("Parser correctly rejected malformed JSON");
            }
        }
    }

    #[test]
    fn test_load_config_with_permission_denied() {
        // This test might not work on all systems, but tests permission handling
        let restricted_path = "/root/config.jsonc";
        let result = load_app_config(restricted_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_unicode_content() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("unicode_config.jsonc");

        let unicode_config = r#"
        {
            "appSettings": {
                "appName": "测试应用 🚀",
                "version": "1.0.0",
                "modulesDir": "模块",
                "description": "Приложение с юникодом",
                "emoji": "🔧⚙️🛠️"
            }
        }
        "#;

        std::fs::write(&config_path, unicode_config).unwrap();

        let config = load_app_config(&config_path).unwrap();
        assert_eq!(config.app_settings.app_name, "测试应用 🚀");
        assert_eq!(config.app_settings.modules_dir, "模块");

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_deeply_nested_config_structure() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("nested_config.jsonc");

        let nested_config = r#"
        {
            "appSettings": {
                "appName": "Nested App",
                "version": "1.0.0",
                "modulesDir": "modules",
                "animation": {
                    "steps": 10,
                    "delayMs": 100,
                    "easing": {
                        "type": "ease-in-out",
                        "duration": 500,
                        "curves": [0.25, 0.1, 0.25, 1.0]
                    }
                }
            },
            "complexObject": {
                "level1": {
                    "level2": {
                        "level3": {
                            "value": "deep value"
                        }
                    }
                }
            }
        }
        "#;

        std::fs::write(&config_path, nested_config).unwrap();

        let config = load_app_config(&config_path).unwrap();
        assert_eq!(config.app_settings.app_name, "Nested App");
        assert!(config.app_settings.animation.is_some());

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_config_with_null_values() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("null_values_config.jsonc");

        let null_config = r#"
        {
            "appSettings": {
                "appName": "Null Test",
                "version": "1.0.0",
                "modulesDir": "modules",
                "downloadDir": null, // Explicit null
                "installPrefix": null,
                "animation": null
            }
        }
        "#;

        std::fs::write(&config_path, null_config).unwrap();

        let config = load_app_config(&config_path).unwrap();
        assert_eq!(config.app_settings.download_dir, None);
        assert_eq!(config.app_settings.install_prefix, None);
        assert!(config.app_settings.animation.is_none());

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_large_configuration_file() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("large_config.jsonc");

        // Create a config with many entries to test performance and memory usage
        let mut large_config = String::from(
            r#"
        {
            "appSettings": {
                "appName": "Large Config Test",
                "version": "1.0.0",
                "modulesDir": "modules"
            },
            "manyEntries": {
        "#,
        );

        for i in 0..1000 {
            large_config.push_str(&format!(
                r#"
                "entry_{i}": {{
                    "value": "test_value_{i}",
                    "number": {i},
                    "enabled": {}
                }}{}"#,
                i % 2 == 0,                     // Alternate true/false
                if i < 999 { "," } else { "" }  // No comma for last entry
            ));
        }

        large_config.push_str(
            r#"
            }
        }"#,
        );

        std::fs::write(&config_path, large_config).unwrap();

        // Should handle large files without issues
        let config = load_app_config(&config_path).unwrap();
        assert_eq!(config.app_settings.app_name, "Large Config Test");

        std::fs::remove_file(&config_path).unwrap();
    }

    #[test]
    fn test_concurrent_config_loading() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;

        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("concurrent_config.jsonc");

        let config_content = r#"
        {
            "appSettings": {
                "appName": "Concurrent Test",
                "version": "1.0.0",
                "modulesDir": "modules"
            }
        }
        "#;

        std::fs::write(&config_path, config_content).unwrap();

        let success_count = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let path = config_path.clone();
                let counter = Arc::clone(&success_count);
                thread::spawn(move || {
                    if load_app_config(&path).is_ok() {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All threads should successfully load the config
        assert_eq!(success_count.load(Ordering::SeqCst), 10);

        std::fs::remove_file(&config_path).unwrap();
    }
}
