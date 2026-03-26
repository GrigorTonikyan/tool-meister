use serde::Deserialize;

/// Root configuration structure containing all package manager settings
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigRoot {
    /// List of package managers to manage
    #[serde(rename = "package_manager", default)]
    pub package_managers: Vec<PackageManagerConfig>,
    
    /// Global settings for the application
    pub settings: Option<GlobalSettings>,
}

/// Global settings for the application
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalSettings {
    /// Whether to run updates in parallel
    pub parallel_updates: Option<bool>,
    
    /// Default timeout in seconds for package manager updates
    pub update_timeout_seconds: Option<u64>,
    
    /// Default shell to use for commands
    pub default_shell: Option<String>,
    
    /// Whether to keep a history of update logs
    pub log_history: Option<bool>,
}

/// Configuration for a single package manager
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackageManagerConfig {
    /// Display name for the package manager
    pub name: String,
    /// Whether this package manager is enabled
    pub enabled: bool,
    /// Command to execute when updating this package manager
    pub command: String,
    /// Whether this command requires sudo privileges
    pub needs_sudo: bool,
    // TODO: Add for v0.2.0+:
    // - depends_on: Vec<String> - Depends on other package managers (for ordering)
    // - timeout_seconds: Option<u64> - Custom timeout
    // - priority: Option<i32> - Execution priority
}

// Default implementation to support creating empty config
impl Default for ConfigRoot {
    fn default() -> Self {
        Self {
            package_managers: Vec::new(),
            settings: None, // Initialize settings to None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_root_default() {
        let default_config = ConfigRoot::default();
        assert!(default_config.package_managers.is_empty());
    }

    #[test]
    fn test_deserialize_valid_config() {
        let config_str = r#"
        [[package_manager]]
        name = "TEST"
        enabled = true
        command = "test command"
        needs-sudo = false
        "#;

        let config: ConfigRoot = toml::from_str(config_str).unwrap();
        assert_eq!(config.package_managers.len(), 1);

        let pm = &config.package_managers[0];
        assert_eq!(pm.name, "TEST");
        assert_eq!(pm.enabled, true);
        assert_eq!(pm.command, "test command");
        assert_eq!(pm.needs_sudo, false);
    }

    #[test]
    fn test_deserialize_multiple_package_managers() {
        let config_str = r#"
        [[package_manager]]
        name = "PM1"
        enabled = true
        command = "command1"
        needs-sudo = true

        [[package_manager]]
        name = "PM2"
        enabled = false
        command = "command2"
        needs-sudo = false
        "#;

        let config: ConfigRoot = toml::from_str(config_str).unwrap();
        assert_eq!(config.package_managers.len(), 2);

        assert_eq!(config.package_managers[0].name, "PM1");
        assert_eq!(config.package_managers[0].enabled, true);
        assert_eq!(config.package_managers[0].needs_sudo, true);

        assert_eq!(config.package_managers[1].name, "PM2");
        assert_eq!(config.package_managers[1].enabled, false);
        assert_eq!(config.package_managers[1].needs_sudo, false);
    }

    // TODO: Add tests for future configuration options
    // - Test global settings section
    // - Test package manager dependencies
    // - Test priority ordering
}
