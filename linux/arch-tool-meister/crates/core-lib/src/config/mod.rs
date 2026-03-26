//! Configuration Management System
//!
//! This module provides configuration loading, validation, caching, and hot-reload
//! functionality for the Arch Tool Meister core library. It handles JSONC parsing,
//! file system watching, and secure configuration validation following security
//! best practices.
//!
//! # Features
//!
//! - JSONC configuration file parsing with comment support
//! - Configuration validation and error reporting
//! - File system watching for hot-reload functionality
//! - Configuration caching with refresh mechanisms
//! - Path resolution with fallback locations
//! - Security-focused validation and sanitization
//!
//! # Usage
//!
//! ```rust,no_run
//! use core_lib::config::{ConfigLoader, ConfigManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config_manager = ConfigManager::new();
//! let app_config = config_manager.load_app_config("config.jsonc").await?;
//! println!("Loaded configuration: {}", app_config.app_settings.app_name);
//! # Ok(())
//! # }
//! ```

pub mod loader;
pub mod watcher;

use crate::error::{CoreError, CoreResult};
use shared_types::config::{AppConfig, MenuConfig};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration manager that provides centralized configuration loading,
/// caching, and hot-reload capabilities.
///
/// The `ConfigManager` serves as the primary interface for configuration
/// operations in the core library. It provides async methods for loading
/// configurations, handles caching for performance, and supports hot-reload
/// functionality for development workflows.
#[derive(Debug)]
pub struct ConfigManager {
    /// Cached application configuration
    app_config_cache: Arc<RwLock<Option<(PathBuf, AppConfig)>>>,
    /// Cached menu configuration
    menu_config_cache: Arc<RwLock<Option<(PathBuf, MenuConfig)>>>,
    /// Configuration file watcher for hot-reload
    _watcher: Option<watcher::ConfigWatcher>,
}

impl ConfigManager {
    /// Creates a new configuration manager instance.
    ///
    /// The configuration manager starts with empty caches and no active watchers.
    /// Use the various `load_*` methods to populate the configuration cache.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use core_lib::config::ConfigManager;
    ///
    /// let config_manager = ConfigManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            app_config_cache: Arc::new(RwLock::new(None)),
            menu_config_cache: Arc::new(RwLock::new(None)),
            _watcher: None,
        }
    }

    /// Creates a new configuration manager with hot-reload capability.
    ///
    /// This enables automatic reloading of configuration files when they change
    /// on disk. This is particularly useful during development or for systems
    /// that need to respond to configuration changes without restart.
    ///
    /// # Arguments
    ///
    /// * `watch_paths` - Paths to watch for configuration changes
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let watch_paths = vec![Path::new("config.jsonc"), Path::new("main_menu.jsonc")];
    /// let config_manager = ConfigManager::with_watcher(&watch_paths).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_watcher<P: AsRef<Path>>(watch_paths: &[P]) -> CoreResult<Self> {
        let watcher = watcher::ConfigWatcher::new(watch_paths).await?;
        Ok(Self {
            app_config_cache: Arc::new(RwLock::new(None)),
            menu_config_cache: Arc::new(RwLock::new(None)),
            _watcher: Some(watcher),
        })
    }

    /// Loads application configuration from the specified path.
    ///
    /// This method loads and validates the application configuration file,
    /// caching the result for future access. If the configuration is already
    /// cached and the file hasn't changed, returns the cached version.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the application configuration file (typically config.jsonc)
    ///
    /// # Returns
    ///
    /// Returns the loaded and validated application configuration.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Config` if:
    /// - The file cannot be read
    /// - The JSONC syntax is invalid
    /// - Configuration validation fails
    /// - File permissions are insufficient
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_manager = ConfigManager::new();
    /// let app_config = config_manager.load_app_config("config.jsonc").await?;
    /// println!("App name: {}", app_config.app_settings.app_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_app_config<P: AsRef<Path>>(&self, path: P) -> CoreResult<AppConfig> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            let cache = self.app_config_cache.read().await;
            if let Some((cached_path, cached_config)) = &*cache {
                if *cached_path == path {
                    debug!("Returning cached app configuration for: {}", path.display());
                    return Ok(cached_config.clone());
                }
            }
        }

        info!("Loading application configuration from: {}", path.display());
        let config = loader::ConfigLoader::load_app_config(&path).await?;

        // Update cache
        {
            let mut cache = self.app_config_cache.write().await;
            *cache = Some((path.clone(), config.clone()));
        }

        debug!("Successfully loaded and cached app configuration");
        Ok(config)
    }

    /// Loads menu configuration from the specified path.
    ///
    /// Similar to `load_app_config`, this method loads and caches menu
    /// configuration with validation and error handling.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the menu configuration file (typically main_menu.jsonc)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_manager = ConfigManager::new();
    /// let menu_config = config_manager.load_menu_config("main_menu.jsonc").await?;
    /// println!("Menu title: {}", menu_config.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_menu_config<P: AsRef<Path>>(&self, path: P) -> CoreResult<MenuConfig> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            let cache = self.menu_config_cache.read().await;
            if let Some((cached_path, cached_config)) = &*cache {
                if *cached_path == path {
                    debug!(
                        "Returning cached menu configuration for: {}",
                        path.display()
                    );
                    return Ok(cached_config.clone());
                }
            }
        }

        info!("Loading menu configuration from: {}", path.display());
        let config = loader::ConfigLoader::load_menu_config(&path).await?;

        // Update cache
        {
            let mut cache = self.menu_config_cache.write().await;
            *cache = Some((path.clone(), config.clone()));
        }

        debug!("Successfully loaded and cached menu configuration");
        Ok(config)
    }

    /// Resolves configuration file paths with fallback locations.
    ///
    /// This method searches for configuration files in standard locations,
    /// providing a fallback mechanism when files are not found in the
    /// primary location.
    ///
    /// # Arguments
    ///
    /// * `filename` - The configuration filename to search for
    ///
    /// # Returns
    ///
    /// Returns the first valid path found, or an error if no valid
    /// configuration file is found in any location.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_manager = ConfigManager::new();
    /// let config_path = config_manager.resolve_config_path("config.jsonc").await?;
    /// println!("Found config at: {}", config_path.display());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_config_path<P: AsRef<Path>>(&self, filename: P) -> CoreResult<PathBuf> {
        let filename = filename.as_ref();

        // Standard search locations
        let search_locations = vec![
            // Current directory
            std::env::current_dir().unwrap_or_default().join(filename),
            // User config directory
            directories::UserDirs::new()
                .map(|user_dirs| {
                    user_dirs
                        .home_dir()
                        .join(".config/arch-tool-meister")
                        .join(filename)
                })
                .unwrap_or_else(|| PathBuf::from("~/.config/arch-tool-meister").join(filename)),
            // System config directory
            PathBuf::from("/etc/arch-tool-meister").join(filename),
            // Fallback to relative path in project structure
            PathBuf::from("atm-rust-tui").join(filename),
        ];

        for location in &search_locations {
            if location.exists() && location.is_file() {
                info!("Found configuration file at: {}", location.display());
                return Ok(location.clone());
            }
        }

        warn!(
            "Configuration file '{}' not found in any standard location",
            filename.display()
        );
        Err(CoreError::config_with_path(
            format!("Configuration file '{}' not found", filename.display()),
            filename.to_string_lossy().to_string(),
        ))
    }

    /// Refreshes all cached configurations by reloading from disk.
    ///
    /// This method clears the cache and forces a reload of all configurations
    /// from their source files. Useful when you know configurations have
    /// changed and want to ensure the latest versions are loaded.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_manager = ConfigManager::new();
    /// config_manager.refresh_cache().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_cache(&self) {
        info!("Refreshing configuration cache");

        // Clear caches
        {
            let mut app_cache = self.app_config_cache.write().await;
            *app_cache = None;
        }
        {
            let mut menu_cache = self.menu_config_cache.write().await;
            *menu_cache = None;
        }

        debug!("Configuration cache cleared");
    }

    /// Returns true if the application configuration is cached.
    pub async fn has_app_config_cached(&self) -> bool {
        let cache = self.app_config_cache.read().await;
        cache.is_some()
    }

    /// Returns true if the menu configuration is cached.
    pub async fn has_menu_config_cached(&self) -> bool {
        let cache = self.menu_config_cache.read().await;
        cache.is_some()
    }

    /// Enables hot-reload for configuration files.
    ///
    /// This method sets up file system watching and automatic cache invalidation
    /// when configuration files change. When a configuration file is modified,
    /// the cache is automatically refreshed on the next access.
    ///
    /// # Arguments
    ///
    /// * `config_paths` - Paths to configuration files to watch for changes
    ///
    /// # Returns
    ///
    /// Returns a receiver for configuration change events.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_manager = ConfigManager::new();
    /// let paths = vec![Path::new("config.jsonc"), Path::new("main_menu.jsonc")];
    ///
    /// let mut change_receiver = config_manager.enable_hot_reload(&paths).await?;
    ///
    /// // Handle configuration change events
    /// while let Ok(event) = change_receiver.recv().await {
    ///     println!("Configuration changed: {:?}", event.path);
    ///     // Configuration will be automatically reloaded on next access
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enable_hot_reload<P: AsRef<Path>>(
        &self,
        config_paths: &[P],
    ) -> CoreResult<tokio::sync::broadcast::Receiver<watcher::ConfigChangeEvent>> {
        info!(
            "Enabling hot-reload for {} configuration files",
            config_paths.len()
        );

        let mut watcher = watcher::ConfigWatcher::new(config_paths).await?;
        watcher.start().await?;

        let mut receiver = watcher.subscribe().await;

        // Spawn a task to handle configuration change events
        let app_config_cache = Arc::clone(&self.app_config_cache);
        let menu_config_cache = Arc::clone(&self.menu_config_cache);

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                info!(
                    "Configuration file changed: {:?} - invalidating cache",
                    event.path
                );

                match event.event_kind {
                    watcher::ConfigEventKind::Modified | watcher::ConfigEventKind::Created => {
                        // Invalidate appropriate cache based on file name
                        let file_name = event
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase();

                        if file_name.contains("config") && !file_name.contains("menu") {
                            // Likely application config
                            let mut cache = app_config_cache.write().await;
                            *cache = None;
                            debug!("Invalidated application configuration cache");
                        } else if file_name.contains("menu") {
                            // Likely menu config
                            let mut cache = menu_config_cache.write().await;
                            *cache = None;
                            debug!("Invalidated menu configuration cache");
                        } else {
                            // Invalidate both caches to be safe
                            let mut app_cache = app_config_cache.write().await;
                            let mut menu_cache = menu_config_cache.write().await;
                            *app_cache = None;
                            *menu_cache = None;
                            debug!("Invalidated all configuration caches");
                        }
                    }
                    watcher::ConfigEventKind::Deleted => {
                        warn!("Configuration file deleted: {:?}", event.path);
                        // Invalidate cache but don't reload until file is recreated
                        let mut app_cache = app_config_cache.write().await;
                        let mut menu_cache = menu_config_cache.write().await;
                        *app_cache = None;
                        *menu_cache = None;
                    }
                    _ => {
                        debug!("Ignoring configuration file event: {:?}", event);
                    }
                }
            }
        });

        // Return a new receiver for the caller
        Ok(watcher.subscribe().await)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export commonly used types and functions
pub use loader::ConfigLoader;
pub use watcher::ConfigWatcher;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let config_manager = ConfigManager::new();
        assert!(!config_manager.has_app_config_cached().await);
        assert!(!config_manager.has_menu_config_cached().await);
    }

    #[tokio::test]
    async fn test_config_path_resolution() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.jsonc");

        // Create a test config file
        let test_config = r#"
        {
            "appSettings": {
                "appName": "Test App",
                "version": "1.0.0",
                "modulesDir": "modules",
                "downloadDir": "/tmp/downloads",
                "installPrefix": "/opt",
                "animation": {
                    "steps": 10,
                    "delayMs": 320
                }
            }
        }
        "#;

        fs::write(&config_path, test_config).expect("Failed to write test config");

        // Change to temp directory so resolution finds our test file
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

        let config_manager = ConfigManager::new();
        let resolved_path = config_manager
            .resolve_config_path("test_config.jsonc")
            .await;

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");

        assert!(resolved_path.is_ok());
        assert!(resolved_path.unwrap().ends_with("test_config.jsonc"));
    }

    #[tokio::test]
    async fn test_cache_refresh() {
        let config_manager = ConfigManager::new();

        // Initially no cache
        assert!(!config_manager.has_app_config_cached().await);

        // Refresh should not crash even with empty cache
        config_manager.refresh_cache().await;
        assert!(!config_manager.has_app_config_cached().await);
    }

    #[tokio::test]
    async fn test_config_path_not_found() {
        let config_manager = ConfigManager::new();
        let result = config_manager
            .resolve_config_path("nonexistent_config.jsonc")
            .await;

        assert!(result.is_err());
        if let Err(CoreError::Config { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected CoreError::Config");
        }
    }

    #[tokio::test]
    async fn test_default_implementation() {
        let config_manager = ConfigManager::default();
        assert!(!config_manager.has_app_config_cached().await);
        assert!(!config_manager.has_menu_config_cached().await);
    }
}
