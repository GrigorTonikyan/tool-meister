use super::model::ConfigRoot;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_DIR_NAME: &str = ".config";
pub const CONFIG_FILE_NAME: &str = "up-all.toml";
pub const DEFAULT_CONFIG: &str = include_str!("../../resources/default_config.toml");

/// Manages configuration file operations for the application
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the default config file location
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let config_path = home_dir.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME);
        Ok(Self { config_path })
    }

    /// Creates a ConfigManager with a specific config file path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            config_path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns the path to the config file
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Creates a default config file if one doesn't exist
    ///
    /// # Returns
    /// * `Result<bool>` - True if a new default config was created, false if it already existed
    pub fn create_default_if_missing(&self) -> Result<bool> {
        if !self.config_path.exists() {
            debug!(
                "Config file not found, creating default at {:?}",
                self.config_path
            );

            // Ensure parent directory exists
            if let Some(parent_dir) = self.config_path.parent() {
                fs::create_dir_all(parent_dir).with_context(|| {
                    format!(
                        "Failed to create config directory: {}",
                        parent_dir.display()
                    )
                })?;
            }

            // Write default config
            fs::write(&self.config_path, DEFAULT_CONFIG).with_context(|| {
                format!(
                    "Failed to write default config: {}",
                    self.config_path.display()
                )
            })?;

            info!("Created default configuration file");
            return Ok(true);
        }
        Ok(false)
    }

    /// Loads and parses the configuration file
    ///
    /// # Returns
    /// * `Result<ConfigRoot>` - The parsed configuration
    pub fn load_config(&self) -> Result<ConfigRoot> {
        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config: {}", self.config_path.display()))?;

        let config: ConfigRoot = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML: {}", self.config_path.display()))?;

        debug!(
            "Loaded {} package managers from config",
            config.package_managers.len()
        );
        Ok(config)
    }

    /// Validates the configuration file
    ///
    /// # Returns
    /// * `Result<bool>` - True if the configuration is valid, false otherwise
    pub fn validate(&self) -> Result<bool> {
        info!("Validating configuration at {}", self.config_path.display());
        let mut is_valid = true;

        if !self.config_path.exists() {
            warn!("Config file doesn't exist");
            return Ok(false);
        }

        match self.load_config() {
            Ok(config) => {
                // Check for empty commands in enabled managers
                for pm in &config.package_managers {
                    if pm.enabled && pm.command.trim().is_empty() {
                        error!("Empty 'command' for enabled package manager '{}'", pm.name);
                        is_valid = false;
                    }
                    if pm.name.trim().is_empty() {
                        error!("Empty 'name' found for a package manager entry");
                        is_valid = false;
                    }
                }
            }
            Err(e) => {
                error!("Failed to load config: {:#}", e);
                is_valid = false;
            }
        }

        if is_valid {
            info!("Configuration appears valid");
        } else {
            warn!("Configuration validation failed");
        }

        Ok(is_valid)
    }

    /// Creates a backup of the configuration file
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The path to the backup file
    pub fn backup(&self) -> Result<PathBuf> {
        if !self.config_path.exists() {
            info!("Config file does not exist. Nothing to back up.");
            return Err(anyhow::anyhow!("Config file does not exist"));
        }

        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        let filename = self
            .config_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| CONFIG_FILE_NAME.to_string());

        let backup_filename = format!("{}.backup.{}", filename, timestamp);
        let backup_path = self.config_path.with_file_name(backup_filename);

        fs::copy(&self.config_path, &backup_path)?;
        info!("Configuration backed up to {}", backup_path.display());

        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_default_if_missing() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let config_manager = ConfigManager::with_path(&config_path);

        // File doesn't exist yet
        assert!(!config_path.exists());

        // Create default file
        let created = config_manager.create_default_if_missing().unwrap();
        assert!(created);
        assert!(config_path.exists());

        // Try creating again - should return false
        let created_again = config_manager.create_default_if_missing().unwrap();
        assert!(!created_again);
    }

    #[test]
    fn test_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a test config file
        let config_content = r#"
        [[package_manager]]
        name = "TEST_PM"
        enabled = true
        command = "test command"
        needs-sudo = false
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config_manager = ConfigManager::with_path(&config_path);
        let config = config_manager.load_config().unwrap();

        assert_eq!(config.package_managers.len(), 1);
        assert_eq!(config.package_managers[0].name, "TEST_PM");
    }

    #[test]
    fn test_validate_valid_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a valid config file
        let config_content = r#"
        [[package_manager]]
        name = "TEST_PM"
        enabled = true
        command = "test command"
        needs-sudo = false
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config_manager = ConfigManager::with_path(&config_path);
        let is_valid = config_manager.validate().unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_validate_invalid_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create an invalid config file (empty command for enabled PM)
        let config_content = r#"
        [[package_manager]]
        name = "TEST_PM"
        enabled = true
        command = ""
        needs-sudo = false
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config_manager = ConfigManager::with_path(&config_path);
        let is_valid = config_manager.validate().unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_backup_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a test config file
        let config_content = "# Test config";
        fs::write(&config_path, config_content).unwrap();

        let config_manager = ConfigManager::with_path(&config_path);
        let backup_path = config_manager.backup().unwrap();

        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backup"));

        // Check the content
        let backup_content = fs::read_to_string(backup_path).unwrap();
        assert_eq!(backup_content, config_content);
    }

    // TODO: Add tests for error handling scenarios
    // TODO: Add tests for future config features
}
