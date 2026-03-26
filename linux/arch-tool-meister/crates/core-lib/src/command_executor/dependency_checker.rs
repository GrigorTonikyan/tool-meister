//! Dependency Checker Module
//!
//! This module validates that all required dependencies are available before
//! executing module functions. It checks for system packages, binaries,
//! files, and other prerequisites.

use crate::error::CoreResult;
use shared_types::module::{ModuleDependency, ModuleFunction};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tokio::fs;
use tracing::{debug, instrument};

use super::ExecutionContext;

/// Dependency checking configuration
#[derive(Debug, Clone)]
pub struct DependencyConfig {
    /// Cache dependency check results for performance
    pub cache_results: bool,
    /// Timeout for dependency checks
    pub check_timeout_secs: u64,
    /// Skip dependency checks (for testing)
    pub skip_checks: bool,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            cache_results: true,
            check_timeout_secs: 30,
            skip_checks: false,
        }
    }
}

/// Result of a dependency check
#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    /// The dependency that was checked
    pub dependency: ModuleDependency,
    /// Whether the dependency is satisfied
    pub satisfied: bool,
    /// Version information if available
    pub version: Option<String>,
    /// Error message if check failed
    pub error: Option<String>,
}

/// Dependency checker for validating module requirements
#[derive(Debug)]
pub struct DependencyChecker {
    config: DependencyConfig,
    /// Cache of dependency check results
    cache: HashMap<String, DependencyCheckResult>,
}

impl DependencyChecker {
    /// Create a new dependency checker with default configuration
    pub fn new() -> Self {
        Self {
            config: DependencyConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Create a new dependency checker with custom configuration
    pub fn with_config(config: DependencyConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Check dependencies for a module function
    /// For now, this is a placeholder since ModuleFunction doesn't have dependencies field
    #[instrument(skip(self, _function, _context))]
    pub async fn check_dependencies(
        &self,
        _function: &ModuleFunction,
        _context: &ExecutionContext,
    ) -> CoreResult<Vec<DependencyCheckResult>> {
        if self.config.skip_checks {
            debug!("Skipping dependency checks (disabled in config)");
            return Ok(vec![]);
        }

        debug!("Checking dependencies for function");

        // For now, return empty results since ModuleFunction doesn't have dependencies
        // This will be properly implemented when the module structure is enhanced
        Ok(vec![])
    }

    /// Check a single dependency
    #[instrument(skip(self, dependency))]
    pub async fn check_single_dependency(
        &self,
        dependency: &ModuleDependency,
    ) -> CoreResult<DependencyCheckResult> {
        let result = match dependency {
            ModuleDependency::SystemPackage { name, version, .. } => {
                self.check_package_dependency(name, version.as_deref())
                    .await
            }
            ModuleDependency::Command {
                name,
                check_command,
                ..
            } => self.check_command_dependency(name, check_command).await,
            ModuleDependency::File { path, .. } => self.check_file_dependency(path).await,
            ModuleDependency::Service { name, .. } => self.check_service_dependency(name).await,
            ModuleDependency::Module { name, .. } => self.check_module_dependency(name).await,
            ModuleDependency::Environment { variable, .. } => {
                self.check_environment_dependency(variable).await
            }
        }?;

        Ok(result)
    }

    /// Clear the dependency cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Update dependency checker configuration
    pub fn update_config(&mut self, config: DependencyConfig) {
        self.config = config;
        if !self.config.cache_results {
            self.clear_cache();
        }
    }

    // Private helper methods

    /// Check if a package is installed
    async fn check_package_dependency(
        &self,
        package_name: &str,
        _required_version: Option<&str>,
    ) -> CoreResult<DependencyCheckResult> {
        debug!("Checking package dependency: {}", package_name);

        // Simple check using pacman
        let output = Command::new("pacman").args(&["-Q", package_name]).output();

        let satisfied = match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        Ok(DependencyCheckResult {
            dependency: ModuleDependency::SystemPackage {
                name: package_name.to_string(),
                version: None,
                package_manager: Some("pacman".to_string()),
            },
            satisfied,
            version: None,
            error: if satisfied {
                None
            } else {
                Some(format!("Package '{}' not found", package_name))
            },
        })
    }

    /// Check if a command is available
    async fn check_command_dependency(
        &self,
        command_name: &str,
        check_command: &str,
    ) -> CoreResult<DependencyCheckResult> {
        debug!("Checking command dependency: {}", command_name);

        let output = Command::new("sh").arg("-c").arg(check_command).output();

        let command_available = match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        Ok(DependencyCheckResult {
            dependency: ModuleDependency::Command {
                name: command_name.to_string(),
                check_command: check_command.to_string(),
                install_command: None,
            },
            satisfied: command_available,
            version: None,
            error: if command_available {
                None
            } else {
                Some(format!("Command '{}' not available", command_name))
            },
        })
    }

    /// Check if a file exists
    async fn check_file_dependency(&self, file_path: &str) -> CoreResult<DependencyCheckResult> {
        debug!("Checking file dependency: {}", file_path);

        let path = Path::new(file_path);
        let exists = fs::try_exists(path).await.unwrap_or(false);

        Ok(DependencyCheckResult {
            dependency: ModuleDependency::File {
                path: file_path.to_string(),
                required: true,
                description: None,
            },
            satisfied: exists,
            version: None,
            error: if exists {
                None
            } else {
                Some(format!("File '{}' does not exist", file_path))
            },
        })
    }

    /// Check if a service is available/active
    async fn check_service_dependency(
        &self,
        service_name: &str,
    ) -> CoreResult<DependencyCheckResult> {
        debug!("Checking service dependency: {}", service_name);

        let output = Command::new("systemctl")
            .args(&["is-active", service_name])
            .output();

        let service_active = match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        Ok(DependencyCheckResult {
            dependency: ModuleDependency::Service {
                name: service_name.to_string(),
                url: None,
                check_method: None,
            },
            satisfied: service_active,
            version: None,
            error: if service_active {
                None
            } else {
                Some(format!("Service '{}' is not active", service_name))
            },
        })
    }

    /// Check if another module is available
    async fn check_module_dependency(
        &self,
        module_name: &str,
    ) -> CoreResult<DependencyCheckResult> {
        debug!("Checking module dependency: {}", module_name);

        // For now, assume it's available
        Ok(DependencyCheckResult {
            dependency: ModuleDependency::Module {
                name: module_name.to_string(),
                version: None,
                required: true,
            },
            satisfied: true,
            version: None,
            error: None,
        })
    }

    /// Check if an environment variable is set
    async fn check_environment_dependency(
        &self,
        var_name: &str,
    ) -> CoreResult<DependencyCheckResult> {
        debug!("Checking environment variable dependency: {}", var_name);

        let value = std::env::var(var_name).ok();
        let satisfied = value.is_some();

        Ok(DependencyCheckResult {
            dependency: ModuleDependency::Environment {
                variable: var_name.to_string(),
                expected_value: None,
                required: true,
            },
            satisfied,
            version: value,
            error: if satisfied {
                None
            } else {
                Some(format!("Environment variable '{}' is not set", var_name))
            },
        })
    }
}

impl Default for DependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}
