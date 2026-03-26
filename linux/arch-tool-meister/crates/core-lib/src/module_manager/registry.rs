//! Module registry for tracking and managing discovered modules.
//!
//! The module registry maintains a centralized index of all discovered modules,
//! their metadata, status, and relationships. It provides functionality for
//! querying, filtering, and managing the lifecycle of modules.

use crate::error::CoreError;
use shared_types::module::ModuleInfo;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Central registry for managing discovered modules.
#[derive(Debug)]
pub struct ModuleRegistry {
    /// Registry of all modules indexed by name
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a module in the registry
    pub async fn register_module(&self, module: ModuleInfo) -> Result<(), CoreError> {
        let module_name = module.name.clone();
        debug!("Registering module: {}", module_name);

        let mut modules = self.modules.write().await;
        modules.insert(module_name.clone(), module);

        info!("Module '{}' registered successfully", module_name);
        Ok(())
    }

    /// Register multiple modules at once
    pub async fn register_modules(&self, modules: Vec<ModuleInfo>) -> Result<(), CoreError> {
        let mut registry = self.modules.write().await;

        for module in modules {
            let module_name = module.name.clone();
            debug!("Registering module: {}", module_name);
            registry.insert(module_name, module);
        }

        info!("Registered {} modules", registry.len());
        Ok(())
    }

    /// Update the entire registry with a new set of modules
    pub async fn update_registry(
        &self,
        modules: HashMap<String, ModuleInfo>,
    ) -> Result<(), CoreError> {
        let mut registry = self.modules.write().await;
        *registry = modules;
        info!("Registry updated with {} modules", registry.len());
        Ok(())
    }

    /// Get a module by name
    pub async fn get_module(&self, name: &str) -> Option<ModuleInfo> {
        let modules = self.modules.read().await;
        modules.get(name).cloned()
    }

    /// Get all registered modules
    pub async fn list_modules(&self) -> Vec<ModuleInfo> {
        let modules = self.modules.read().await;
        modules.values().cloned().collect()
    }

    /// Get modules that match a predicate
    pub async fn filter_modules<F>(&self, predicate: F) -> Vec<ModuleInfo>
    where
        F: Fn(&ModuleInfo) -> bool,
    {
        let modules = self.modules.read().await;
        modules.values().filter(|m| predicate(m)).cloned().collect()
    }

    /// Get enabled modules only
    pub async fn get_enabled_modules(&self) -> Vec<ModuleInfo> {
        self.filter_modules(|module| module.enabled).await
    }

    /// Get module names
    pub async fn get_module_names(&self) -> Vec<String> {
        let modules = self.modules.read().await;
        modules.keys().cloned().collect()
    }

    /// Check if a module is registered
    pub async fn has_module(&self, name: &str) -> bool {
        let modules = self.modules.read().await;
        modules.contains_key(name)
    }

    /// Remove a module from the registry
    pub async fn unregister_module(&self, name: &str) -> Result<Option<ModuleInfo>, CoreError> {
        let mut modules = self.modules.write().await;
        let removed = modules.remove(name);

        if removed.is_some() {
            info!("Module '{}' unregistered", name);
        } else {
            warn!("Attempted to unregister non-existent module: {}", name);
        }

        Ok(removed)
    }

    /// Clear all modules from the registry
    pub async fn clear(&self) -> Result<(), CoreError> {
        let mut modules = self.modules.write().await;
        let count = modules.len();
        modules.clear();
        info!("Registry cleared, removed {} modules", count);
        Ok(())
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let modules = self.modules.read().await;
        let total = modules.len();
        let enabled = modules.values().filter(|m| m.enabled).count();
        let disabled = total - enabled;

        RegistryStats {
            total_modules: total,
            enabled_modules: enabled,
            disabled_modules: disabled,
        }
    }

    /// Find modules that depend on a specific module
    pub async fn find_dependents(&self, module_name: &str) -> Vec<ModuleInfo> {
        let modules = self.modules.read().await;
        modules
            .values()
            .filter(|module| {
                // Check if any commands reference functions from the target module
                // Note: This is a simplified dependency check
                module
                    .commands
                    .values()
                    .any(|cmd| cmd.function.starts_with(&format!("{}.", module_name)))
            })
            .cloned()
            .collect()
    }

    /// Validate registry consistency
    pub async fn validate_registry(&self) -> Result<Vec<String>, CoreError> {
        let modules = self.modules.read().await;
        let mut issues = Vec::new();

        for (name, module) in modules.iter() {
            // Check for name consistency
            if name != &module.name {
                issues.push(format!(
                    "Module name mismatch: registry key '{}' vs module name '{}'",
                    name, module.name
                ));
            }

            // Validate individual module
            if let Err(e) = module.validate() {
                issues.push(format!("Module '{}' validation failed: {}", name, e));
            }
        }

        if issues.is_empty() {
            info!("Registry validation passed for {} modules", modules.len());
        } else {
            warn!("Registry validation found {} issues", issues.len());
        }

        Ok(issues)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of modules
    pub total_modules: usize,
    /// Number of enabled modules
    pub enabled_modules: usize,
    /// Number of disabled modules
    pub disabled_modules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::module::{ModuleCommand, ModuleFunction, ModuleMenu};

    fn create_test_module(name: &str, enabled: bool) -> ModuleInfo {
        ModuleInfo::builder(
            name.to_string(),
            format!("Test module {}", name),
            "1.0.0".to_string(),
        )
        .enabled(enabled)
        .build()
        .unwrap()
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ModuleRegistry::new();
        let stats = registry.get_stats().await;

        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.enabled_modules, 0);
        assert_eq!(stats.disabled_modules, 0);
    }

    #[tokio::test]
    async fn test_register_and_get_module() {
        let registry = ModuleRegistry::new();
        let test_module = create_test_module("test-module", true);

        registry.register_module(test_module.clone()).await.unwrap();

        let retrieved = registry.get_module("test-module").await;
        assert_eq!(retrieved, Some(test_module));

        let missing = registry.get_module("nonexistent").await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_list_modules() {
        let registry = ModuleRegistry::new();
        let module1 = create_test_module("module1", true);
        let module2 = create_test_module("module2", false);

        registry.register_module(module1.clone()).await.unwrap();
        registry.register_module(module2.clone()).await.unwrap();

        let modules = registry.list_modules().await;
        assert_eq!(modules.len(), 2);
        assert!(modules.contains(&module1));
        assert!(modules.contains(&module2));
    }

    #[tokio::test]
    async fn test_filter_modules() {
        let registry = ModuleRegistry::new();
        let enabled_module = create_test_module("enabled", true);
        let disabled_module = create_test_module("disabled", false);

        registry
            .register_module(enabled_module.clone())
            .await
            .unwrap();
        registry
            .register_module(disabled_module.clone())
            .await
            .unwrap();

        let enabled_modules = registry.get_enabled_modules().await;
        assert_eq!(enabled_modules.len(), 1);
        assert_eq!(enabled_modules[0].name, "enabled");
    }

    #[tokio::test]
    async fn test_unregister_module() {
        let registry = ModuleRegistry::new();
        let test_module = create_test_module("test-module", true);

        registry.register_module(test_module.clone()).await.unwrap();

        let removed = registry.unregister_module("test-module").await.unwrap();
        assert_eq!(removed, Some(test_module));

        let missing = registry.get_module("test-module").await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let registry = ModuleRegistry::new();
        let enabled_module = create_test_module("enabled", true);
        let disabled_module = create_test_module("disabled", false);

        registry.register_module(enabled_module).await.unwrap();
        registry.register_module(disabled_module).await.unwrap();

        let stats = registry.get_stats().await;
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.enabled_modules, 1);
        assert_eq!(stats.disabled_modules, 1);
    }

    #[tokio::test]
    async fn test_clear_registry() {
        let registry = ModuleRegistry::new();
        let module1 = create_test_module("module1", true);
        let module2 = create_test_module("module2", true);

        registry.register_module(module1).await.unwrap();
        registry.register_module(module2).await.unwrap();

        registry.clear().await.unwrap();

        let stats = registry.get_stats().await;
        assert_eq!(stats.total_modules, 0);
    }

    #[tokio::test]
    async fn test_has_module() {
        let registry = ModuleRegistry::new();
        let test_module = create_test_module("test-module", true);

        assert!(!registry.has_module("test-module").await);

        registry.register_module(test_module).await.unwrap();
        assert!(registry.has_module("test-module").await);
    }

    #[tokio::test]
    async fn test_update_registry() {
        let registry = ModuleRegistry::new();
        let module1 = create_test_module("module1", true);

        // Initial state
        registry.register_module(module1).await.unwrap();
        assert_eq!(registry.get_stats().await.total_modules, 1);

        // Update with new set
        let mut new_modules = HashMap::new();
        new_modules.insert("module2".to_string(), create_test_module("module2", true));
        new_modules.insert("module3".to_string(), create_test_module("module3", false));

        registry.update_registry(new_modules).await.unwrap();

        let stats = registry.get_stats().await;
        assert_eq!(stats.total_modules, 2);
        assert!(!registry.has_module("module1").await);
        assert!(registry.has_module("module2").await);
        assert!(registry.has_module("module3").await);
    }
}
