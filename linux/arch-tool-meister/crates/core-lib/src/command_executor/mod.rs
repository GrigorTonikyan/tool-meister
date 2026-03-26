//! Command Execution System
//!
//! This module provides secure command execution capabilities for Arch Tool Meister.
//! It handles execution of module functions, security validation, dependency checking,
//! and provides features like dry-run mode, command history, and output streaming.

use crate::error::{CoreError, CoreResult};
use shared_types::module::{ModuleFunction, ModuleInfo};
use std::time::Duration;
use tracing::{info, instrument};

pub mod dependency_checker;
pub mod runner;
pub mod security;

/// Command execution context with metadata
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The module this command belongs to
    pub module_name: String,
    /// The command/function being executed
    pub function_name: String,
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// Whether this is a dry-run (preview only)
    pub dry_run: bool,
    /// Maximum execution timeout
    pub timeout: Duration,
    /// Working directory for execution
    pub working_dir: Option<String>,
    /// Environment variables to set
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error output
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
    /// Whether the command was cancelled due to timeout
    pub timed_out: bool,
}

/// Command execution options and settings
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// Enable dry-run mode (preview only, no execution)
    pub dry_run: bool,
    /// Maximum execution timeout
    pub timeout: Duration,
    /// Enable streaming output during execution
    pub stream_output: bool,
    /// Enable privilege escalation if needed
    pub allow_sudo: bool,
    /// Additional environment variables
    pub env_vars: std::collections::HashMap<String, String>,
    /// Working directory override
    pub working_dir: Option<String>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            timeout: Duration::from_secs(300), // 5 minutes default
            stream_output: false,
            allow_sudo: false,
            env_vars: std::collections::HashMap::new(),
            working_dir: None,
        }
    }
}

/// Main command executor managing secure command execution
#[derive(Debug)]
pub struct CommandExecutor {
    /// Security validator
    security_validator: security::SecurityValidator,
    /// Dependency checker
    dependency_checker: dependency_checker::DependencyChecker,
    /// Command runner
    command_runner: runner::CommandRunner,
    /// Default execution options
    default_options: ExecutionOptions,
}

impl CommandExecutor {
    /// Create a new command executor with default settings
    pub fn new() -> Self {
        Self {
            security_validator: security::SecurityValidator::new(),
            dependency_checker: dependency_checker::DependencyChecker::new(),
            command_runner: runner::CommandRunner::new(),
            default_options: ExecutionOptions::default(),
        }
    }

    /// Create a new command executor with custom options
    pub fn with_options(options: ExecutionOptions) -> Self {
        Self {
            security_validator: security::SecurityValidator::new(),
            dependency_checker: dependency_checker::DependencyChecker::new(),
            command_runner: runner::CommandRunner::new(),
            default_options: options,
        }
    }

    /// Execute a module function with the given arguments
    #[instrument(skip(self, module, function_name), fields(module = %module.name, function = %function_name))]
    pub async fn execute_function(
        &self,
        module: &ModuleInfo,
        function_name: &str,
        args: Vec<String>,
        options: Option<ExecutionOptions>,
    ) -> CoreResult<ExecutionResult> {
        let opts = options.unwrap_or_else(|| self.default_options.clone());

        // Get the function from the module
        let function = module.functions.get(function_name).ok_or_else(|| {
            CoreError::from(shared_types::SharedError::validation(format!(
                "Function '{}' not found in module '{}'",
                function_name, module.name
            )))
        })?;

        // Create execution context
        let context = ExecutionContext {
            module_name: module.name.clone(),
            function_name: function_name.to_string(),
            args: args.clone(),
            dry_run: opts.dry_run,
            timeout: opts.timeout,
            working_dir: opts.working_dir.clone(),
            env_vars: opts.env_vars.clone(),
        };

        info!(
            "Executing function '{}' from module '{}'",
            function_name, module.name
        );

        // Security validation
        self.security_validator
            .validate_function(function, &context)
            .await?;

        // Dependency checking
        self.dependency_checker
            .check_dependencies(function, &context)
            .await?;

        if opts.dry_run {
            info!(
                "Dry-run mode: Would execute function '{}' with args: {:?}",
                function_name, args
            );
            return Ok(ExecutionResult {
                exit_code: 0,
                stdout: format!("[DRY-RUN] Would execute: {}", function.code),
                stderr: String::new(),
                duration: Duration::from_millis(0),
                timed_out: false,
            });
        }

        // Execute the command
        self.command_runner
            .run_function(function, &context, &opts)
            .await
    }

    /// Execute a command directly (for non-module commands)
    #[instrument(skip(self, command))]
    pub async fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
        options: Option<ExecutionOptions>,
    ) -> CoreResult<ExecutionResult> {
        let opts = options.unwrap_or_else(|| self.default_options.clone());

        let context = ExecutionContext {
            module_name: "direct".to_string(),
            function_name: "command".to_string(),
            args: args.clone(),
            dry_run: opts.dry_run,
            timeout: opts.timeout,
            working_dir: opts.working_dir.clone(),
            env_vars: opts.env_vars.clone(),
        };

        info!("Executing direct command: {} {:?}", command, args);

        // Security validation for direct commands
        self.security_validator
            .validate_command(command, &args, &context)
            .await?;

        if opts.dry_run {
            info!(
                "Dry-run mode: Would execute command: {} {:?}",
                command, args
            );
            return Ok(ExecutionResult {
                exit_code: 0,
                stdout: format!("[DRY-RUN] Would execute: {} {}", command, args.join(" ")),
                stderr: String::new(),
                duration: Duration::from_millis(0),
                timed_out: false,
            });
        }

        // Execute the command directly
        self.command_runner
            .run_command(command, &args, &context, &opts)
            .await
    }

    /// Check if dependencies are met for a function
    pub async fn check_dependencies(&self, function: &ModuleFunction) -> CoreResult<bool> {
        let context = ExecutionContext {
            module_name: "check".to_string(),
            function_name: "dependency_check".to_string(),
            args: vec![],
            dry_run: true,
            timeout: Duration::from_secs(10),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        };

        match self
            .dependency_checker
            .check_dependencies(function, &context)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate a function for security issues
    pub async fn validate_function(&self, function: &ModuleFunction) -> CoreResult<bool> {
        let context = ExecutionContext {
            module_name: "validate".to_string(),
            function_name: "security_check".to_string(),
            args: vec![],
            dry_run: true,
            timeout: Duration::from_secs(10),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        };

        match self
            .security_validator
            .validate_function(function, &context)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Update default execution options
    pub fn set_default_options(&mut self, options: ExecutionOptions) {
        self.default_options = options;
    }

    /// Get current default execution options
    pub fn get_default_options(&self) -> &ExecutionOptions {
        &self.default_options
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}
