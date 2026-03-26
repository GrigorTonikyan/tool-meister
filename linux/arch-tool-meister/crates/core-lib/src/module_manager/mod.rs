//! Module Management System
//!
//! This module provides comprehensive functionality for discovering, validating,
//! and managing modules within the Arch Tool Meister ecosystem.

use crate::config::ConfigManager;
use crate::error::CoreResult;
use shared_types::module::ModuleInfo;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, instrument};

pub mod discovery;
pub mod registry;

use discovery::ModuleDiscovery;
use registry::{ModuleRegistry, RegistryStats};

/// Main module manager providing high-level module operations
#[derive(Debug)]

pub struct ModuleManager {
    registry: Arc<RwLock<ModuleRegistry>>,
    discovery: Arc<RwLock<ModuleDiscovery>>,
    config_manager: Arc<RwLock<ConfigManager>>,
}

impl ModuleManager {
    /// Create a new module manager instance
    pub fn new(config_manager: Arc<RwLock<ConfigManager>>) -> Self {
        let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
        let discovery = Arc::new(RwLock::new(ModuleDiscovery::new()));

        Self {
            registry,
            discovery,
            config_manager,
        }
    }

    /// Initialize the module manager with default module paths
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> CoreResult<()> {
        info!("Initializing module manager");
        self.discover_and_register_modules().await?;
        Ok(())
    }

    /// Discover and register all available modules
    #[instrument(skip(self))]
    pub async fn discover_and_register_modules(&self) -> CoreResult<()> {
        let discovery = self.discovery.read().await;
        let modules = discovery.discover_all_modules().await?;

        let registry = self.registry.write().await;
        registry.update_registry(modules).await?;

        Ok(())
    }

    /// Get all registered modules
    pub async fn list_modules(&self) -> CoreResult<Vec<ModuleInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.list_modules().await)
    }

    /// Get a specific module by name
    pub async fn get_module(&self, name: &str) -> CoreResult<Option<ModuleInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.get_module(name).await)
    }

    /// Get only enabled modules
    pub async fn get_enabled_modules(&self) -> CoreResult<Vec<ModuleInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.get_enabled_modules().await)
    }

    /// Search modules by name or description
    pub async fn search_modules(&self, query: &str) -> CoreResult<Vec<ModuleInfo>> {
        let registry = self.registry.read().await;
        let query_lower = query.to_lowercase();

        Ok(registry
            .filter_modules(|module| {
                module.name.to_lowercase().contains(&query_lower)
                    || module.description.to_lowercase().contains(&query_lower)
            })
            .await)
    }

    /// Filter modules by a custom predicate
    pub async fn filter_modules<F>(&self, predicate: F) -> CoreResult<Vec<ModuleInfo>>
    where
        F: Fn(&ModuleInfo) -> bool + Send,
    {
        let registry = self.registry.read().await;
        Ok(registry.filter_modules(predicate).await)
    }

    /// Check if a module is registered
    pub async fn has_module(&self, name: &str) -> CoreResult<bool> {
        let registry = self.registry.read().await;
        Ok(registry.has_module(name).await)
    }

    /// Validate a specific module configuration
    pub async fn validate_module(&self, module_path: &PathBuf) -> CoreResult<bool> {
        let discovery = self.discovery.read().await;

        // Check if the path contains a valid module configuration
        let config_file = module_path.join("module.jsonc");
        if !config_file.exists() {
            let alt_config = module_path.join("module.json");
            if !alt_config.exists() {
                return Ok(false);
            }
        }

        // Try to load the module configuration
        match discovery.find_module("").await {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Refresh the module registry by re-discovering all modules
    #[instrument(skip(self))]
    pub async fn refresh(&self) -> CoreResult<()> {
        info!("Refreshing module registry");

        // Clear current registry
        {
            let registry = self.registry.write().await;
            registry.clear().await?;
        }

        // Re-discover and register modules
        self.discover_and_register_modules().await?;

        Ok(())
    }

    /// Get module registry statistics
    pub async fn get_stats(&self) -> CoreResult<RegistryStats> {
        let registry = self.registry.read().await;
        Ok(registry.get_stats().await)
    }

    /// Validate the entire module registry
    pub async fn validate_registry(&self) -> CoreResult<Vec<String>> {
        let registry = self.registry.read().await;
        registry.validate_registry().await
    }

    /// Find modules that depend on a specific module
    pub async fn find_dependents(&self, module_name: &str) -> CoreResult<Vec<ModuleInfo>> {
        let registry = self.registry.read().await;
        Ok(registry.find_dependents(module_name).await)
    }

    /// Get module names
    pub async fn get_module_names(&self) -> CoreResult<Vec<String>> {
        let registry = self.registry.read().await;
        Ok(registry.get_module_names().await)
    }
}
