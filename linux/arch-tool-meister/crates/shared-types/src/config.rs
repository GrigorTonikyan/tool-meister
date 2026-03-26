//! Configuration data structures and validation.
//!
//! This module defines the configuration types used across the application,
//! including application-wide settings and menu configurations that match
//! the structure of config.jsonc and main_menu.jsonc files.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main application configuration structure matching config.jsonc.
///
/// This struct represents the overall configuration for the Arch Tool Meister application,
/// including paths, behavior settings, UI preferences, and animation settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application settings section
    #[serde(rename = "appSettings")]
    pub app_settings: AppSettings,
}

/// Application settings subsection of the configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Application name
    #[serde(rename = "appName")]
    pub app_name: String,
    /// Application version
    pub version: String,
    /// Path to the modules directory
    #[serde(rename = "modulesDir")]
    pub modules_dir: String,
    /// Path to the download directory
    #[serde(rename = "downloadDir")]
    pub download_dir: String,
    /// Installation prefix path
    #[serde(rename = "installPrefix")]
    pub install_prefix: String,
    /// Animation settings
    pub animation: AnimationSettings,
}

/// Animation settings for the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Number of animation steps
    pub steps: u32,
    /// Delay between animation frames in milliseconds
    #[serde(rename = "delayMs")]
    pub delay_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_settings: AppSettings::default(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_name: "Arch Tool Meister".to_string(),
            version: "2.0.0".to_string(),
            modules_dir: "modules".to_string(),
            download_dir: "/tmp/downloads".to_string(),
            install_prefix: "/opt".to_string(),
            animation: AnimationSettings::default(),
        }
    }
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            steps: 10,
            delay_ms: 320,
        }
    }
}

/// Menu configuration structure matching main_menu.jsonc.
///
/// This struct represents the menu structure and navigation options
/// available in the TUI interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuConfig {
    /// Menu title displayed at the top
    pub title: String,
    /// Whether the menu is dynamically generated from modules
    #[serde(rename = "dynamicMenu")]
    pub dynamic_menu: bool,
    /// Static menu options (used when dynamic_menu is false)
    pub options: Vec<MenuOption>,
}

/// Individual menu option configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuOption {
    /// Display name of the menu option
    pub name: String,
    /// Description of what this menu option does
    pub description: String,
    /// Associated module or action identifier
    pub action: String,
    /// Whether this option is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Optional hotkey for quick access
    pub hotkey: Option<char>,
}

/// Helper function for serde default
fn default_enabled() -> bool {
    true
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            title: "== Arch Tool Meister - Modular ==".to_string(),
            dynamic_menu: true,
            options: vec![],
        }
    }
}

// Validation methods implementation
impl AppConfig {
    /// Validates the application configuration
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        self.app_settings.validate()?;
        Ok(())
    }

    /// Returns the full path to the modules directory
    pub fn modules_path(&self) -> &str {
        &self.app_settings.modules_dir
    }

    /// Returns the download directory path
    pub fn download_path(&self) -> &str {
        &self.app_settings.download_dir
    }

    /// Returns the install prefix path
    pub fn install_prefix(&self) -> &str {
        &self.app_settings.install_prefix
    }
}

impl AppSettings {
    /// Validates the application settings
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        // Validate version format
        if self.version.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Version cannot be empty",
                "version",
            ));
        }

        // Validate module directory path
        if self.modules_dir.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Modules directory cannot be empty",
                "modules_dir",
            ));
        }

        // Validate download directory path
        if self.download_dir.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Download directory cannot be empty",
                "download_dir",
            ));
        }

        // Validate paths are valid
        if !self.is_valid_path(&self.download_dir) {
            return Err(crate::error::SharedError::validation_with_field(
                "Invalid download directory path",
                "download_dir",
            ));
        }

        if !self.is_valid_path(&self.install_prefix) {
            return Err(crate::error::SharedError::validation_with_field(
                "Invalid install prefix path",
                "install_prefix",
            ));
        }

        // Validate animation settings
        self.animation.validate()?;

        Ok(())
    }

    /// Checks if a path string is valid
    fn is_valid_path(&self, path: &str) -> bool {
        !path.is_empty() && Path::new(path).is_absolute()
    }
}

impl AnimationSettings {
    /// Validates animation settings
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.steps == 0 {
            return Err(crate::error::SharedError::validation_with_field(
                "Animation steps must be greater than 0",
                "steps",
            ));
        }

        if self.steps > 100 {
            return Err(crate::error::SharedError::validation_with_field(
                "Animation steps cannot exceed 100",
                "steps",
            ));
        }

        if self.delay_ms > 5000 {
            return Err(crate::error::SharedError::validation_with_field(
                "Animation delay cannot exceed 5000ms",
                "delay_ms",
            ));
        }

        Ok(())
    }

    /// Returns the total animation duration in milliseconds
    pub fn total_duration_ms(&self) -> u64 {
        self.steps as u64 * self.delay_ms
    }
}

impl MenuConfig {
    /// Validates the menu configuration
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.title.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Menu title cannot be empty",
                "title",
            ));
        }

        // Validate menu options
        for (index, option) in self.options.iter().enumerate() {
            option.validate().map_err(|e| {
                crate::error::SharedError::validation_with_field(
                    format!("Menu option {} validation failed: {}", index, e),
                    "options",
                )
            })?;
        }

        // Check for duplicate hotkeys
        let mut hotkeys = std::collections::HashSet::new();
        for option in &self.options {
            if let Some(hotkey) = option.hotkey {
                if !hotkeys.insert(hotkey) {
                    return Err(crate::error::SharedError::validation_with_field(
                        format!("Duplicate hotkey found: '{}'", hotkey),
                        "options",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Returns enabled menu options only
    pub fn enabled_options(&self) -> Vec<&MenuOption> {
        self.options.iter().filter(|opt| opt.enabled).collect()
    }

    /// Finds a menu option by hotkey
    pub fn find_by_hotkey(&self, hotkey: char) -> Option<&MenuOption> {
        self.options
            .iter()
            .find(|opt| opt.hotkey == Some(hotkey) && opt.enabled)
    }
}

impl MenuOption {
    /// Validates a menu option
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.name.is_empty() {
            return Err(crate::error::SharedError::validation(
                "Menu option name cannot be empty",
            ));
        }

        if self.action.is_empty() {
            return Err(crate::error::SharedError::validation(
                "Menu option action cannot be empty",
            ));
        }

        // Validate hotkey if present
        if let Some(hotkey) = self.hotkey {
            if !hotkey.is_ascii_alphanumeric() && !hotkey.is_ascii_punctuation() {
                return Err(crate::error::SharedError::validation(
                    "Menu option hotkey must be ASCII alphanumeric or punctuation",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.app_settings.app_name, "Arch Tool Meister");
        assert_eq!(config.app_settings.version, "2.0.0");
        assert_eq!(config.app_settings.modules_dir, "modules");
        assert_eq!(config.app_settings.animation.steps, 10);
        assert_eq!(config.app_settings.animation.delay_ms, 320);
    }

    #[test]
    fn test_menu_config_default() {
        let config = MenuConfig::default();
        assert_eq!(config.title, "== Arch Tool Meister - Modular ==");
        assert!(config.dynamic_menu);
        assert!(config.options.is_empty());
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);

        let menu_config = MenuConfig::default();
        let menu_serialized = serde_json::to_string(&menu_config).unwrap();
        let menu_deserialized: MenuConfig = serde_json::from_str(&menu_serialized).unwrap();
        assert_eq!(menu_config, menu_deserialized);
    }

    #[test]
    fn test_validation_success() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());

        let menu_config = MenuConfig::default();
        assert!(menu_config.validate().is_ok());
    }

    #[test]
    fn test_validation_failures() {
        // Test empty version
        let mut config = AppConfig::default();
        config.app_settings.version = String::new();
        assert!(config.validate().is_err());

        // Test invalid animation steps
        let mut config = AppConfig::default();
        config.app_settings.animation.steps = 0;
        assert!(config.validate().is_err());

        config.app_settings.animation.steps = 150;
        assert!(config.validate().is_err());

        // Test menu validation
        let mut menu_config = MenuConfig::default();
        menu_config.title = String::new();
        assert!(menu_config.validate().is_err());
    }

    #[test]
    fn test_menu_option_validation() {
        let valid_option = MenuOption {
            name: "Test Option".to_string(),
            description: "Test Description".to_string(),
            action: "test_action".to_string(),
            enabled: true,
            hotkey: Some('t'),
        };
        assert!(valid_option.validate().is_ok());

        // Test empty name
        let mut invalid_option = valid_option.clone();
        invalid_option.name = String::new();
        assert!(invalid_option.validate().is_err());

        // Test empty action
        let mut invalid_option = valid_option.clone();
        invalid_option.action = String::new();
        assert!(invalid_option.validate().is_err());
    }

    #[test]
    fn test_menu_hotkey_functionality() {
        let mut menu_config = MenuConfig::default();
        menu_config.options = vec![
            MenuOption {
                name: "Option 1".to_string(),
                description: "First option".to_string(),
                action: "action1".to_string(),
                enabled: true,
                hotkey: Some('1'),
            },
            MenuOption {
                name: "Option 2".to_string(),
                description: "Second option".to_string(),
                action: "action2".to_string(),
                enabled: true,
                hotkey: Some('2'),
            },
        ];

        let found = menu_config.find_by_hotkey('1');
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Option 1");

        let not_found = menu_config.find_by_hotkey('9');
        assert!(not_found.is_none());
    }

    #[test]
    fn test_enabled_options_filter() {
        let mut menu_config = MenuConfig::default();
        menu_config.options = vec![
            MenuOption {
                name: "Enabled Option".to_string(),
                description: "This is enabled".to_string(),
                action: "enabled_action".to_string(),
                enabled: true,
                hotkey: None,
            },
            MenuOption {
                name: "Disabled Option".to_string(),
                description: "This is disabled".to_string(),
                action: "disabled_action".to_string(),
                enabled: false,
                hotkey: None,
            },
        ];

        let enabled = menu_config.enabled_options();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Enabled Option");
    }

    #[test]
    fn test_animation_duration() {
        let animation = AnimationSettings {
            steps: 10,
            delay_ms: 100,
        };
        assert_eq!(animation.total_duration_ms(), 1000);
    }
}
