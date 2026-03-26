//! # Core Library for Arch Tool Meister
//!
//! This library provides the core business logic for the Arch Tool Meister application,
//! completely decoupled from any user interface concerns. It handles module management,
//! configuration loading, command execution, and other core functionality.

pub mod command_executor;
pub mod config;
pub mod error;
pub mod module_manager;

use anyhow::Result;
use shared_types::{AppConfig, ModuleInfo};

/// The main facade for interacting with the core library functionality.
///
/// `AtmCore` provides a high-level API that coordinates between the configuration,
/// module management, and command execution subsystems.
pub struct AtmCore {
    config: AppConfig,
    module_manager: module_manager::ModuleManager,
    command_executor: command_executor::CommandExecutor,
}

impl AtmCore {
    /// Create a new builder for constructing an `AtmCore` instance.
    pub fn builder() -> AtmCoreBuilder {
        AtmCoreBuilder::default()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// List all available modules.
    pub async fn list_modules(&self) -> Result<Vec<ModuleInfo>> {
        self.module_manager
            .list_modules()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Get information about a specific module.
    pub async fn get_module(&self, module_name: &str) -> Result<Option<ModuleInfo>> {
        self.module_manager
            .get_module(module_name)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Execute a command from a module.
    pub async fn execute_command(
        &self,
        module_name: &str,
        command_name: &str,
        args: Vec<String>,
    ) -> Result<String> {
        let module = self
            .module_manager
            .get_module(module_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", module_name))?;

        let command = module
            .commands
            .get(command_name)
            .ok_or_else(|| anyhow::anyhow!("Command not found: {}", command_name))?;

        let result = self
            .command_executor
            .execute_function(&module, &command.function, args, None)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(result.stdout)
    }
}

/// Builder for constructing `AtmCore` instances with custom configuration.
#[derive(Default)]
pub struct AtmCoreBuilder {
    config_path: Option<String>,
    modules_path: Option<String>,
}

impl AtmCoreBuilder {
    /// Set the path to the configuration file.
    pub fn with_config_path<S: Into<String>>(mut self, path: S) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set the path to the modules directory.
    pub fn with_modules_path<S: Into<String>>(mut self, path: S) -> Self {
        self.modules_path = Some(path.into());
        self
    }

    /// Build the `AtmCore` instance.
    ///
    /// This method will load the configuration, initialize the module manager,
    /// and set up the command executor.
    pub async fn build(self) -> Result<AtmCore> {
        // For now, return a placeholder implementation
        // This will be fully implemented in later stages
        let config = AppConfig::default();

        // Create config manager and module manager
        let config_manager =
            std::sync::Arc::new(tokio::sync::RwLock::new(crate::config::ConfigManager::new()));
        let module_manager = module_manager::ModuleManager::new(config_manager);
        let command_executor = command_executor::CommandExecutor::new();

        Ok(AtmCore {
            config,
            module_manager,
            command_executor,
        })
    }
}
