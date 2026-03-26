//! Core library error types that wrap SharedError and add core-specific variants.
//!
//! This module defines error types specific to the core business logic operations
//! such as module management, command execution, and configuration handling.

use shared_types::SharedError;
use thiserror::Error;

/// Core library error type that wraps SharedError and adds core-specific error variants.
///
/// This error type represents all errors that can occur within the core library
/// operations. It wraps the foundational SharedError and adds specific variants
/// for core business logic errors.
///
/// # Design Principles
/// - Wraps SharedError for consistent error handling
/// - Adds context specific to core library operations
/// - Maintains error chain for debugging
/// - Provides ergonomic conversion from SharedError
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CoreError {
    /// Shared error from the foundational error types
    #[error(transparent)]
    Shared(#[from] SharedError),

    /// Module-related errors (discovery, validation, loading)
    #[error("Module error: {message}")]
    Module {
        /// Human-readable error message
        message: String,
        /// Module name or path that caused the error
        module_name: Option<String>,
    },

    /// Command execution errors
    #[error("Command execution failed: {message}")]
    Command {
        /// Human-readable error message
        message: String,
        /// Command that failed to execute
        command: Option<String>,
        /// Exit code if available
        exit_code: Option<i32>,
    },

    /// Configuration management errors
    #[error("Configuration management error: {message}")]
    Config {
        /// Human-readable error message
        message: String,
        /// Configuration file or key that caused the error
        config_path: Option<String>,
    },

    /// Dependency resolution and validation errors
    #[error("Dependency error: {message}")]
    Dependency {
        /// Human-readable error message
        message: String,
        /// Missing or problematic dependency
        dependency: Option<String>,
    },

    /// Initialization and setup errors
    #[error("Initialization error: {message}")]
    Initialization {
        /// Human-readable error message
        message: String,
        /// Component that failed to initialize
        component: Option<String>,
    },

    /// State management errors
    #[error("State management error: {message}")]
    State {
        /// Human-readable error message
        message: String,
        /// Operation that failed
        operation: Option<String>,
    },
}

/// Result type alias for core library operations
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    /// Creates a new module error with a message
    pub fn module<S: Into<String>>(message: S) -> Self {
        Self::Module {
            message: message.into(),
            module_name: None,
        }
    }

    /// Creates a new module error with a message and module name
    pub fn module_with_name<S: Into<String>, N: Into<String>>(message: S, module_name: N) -> Self {
        Self::Module {
            message: message.into(),
            module_name: Some(module_name.into()),
        }
    }

    /// Creates a new command error with a message
    pub fn command<S: Into<String>>(message: S) -> Self {
        Self::Command {
            message: message.into(),
            command: None,
            exit_code: None,
        }
    }

    /// Creates a new command error with message, command, and exit code
    pub fn command_with_details<S: Into<String>, C: Into<String>>(
        message: S,
        command: C,
        exit_code: Option<i32>,
    ) -> Self {
        Self::Command {
            message: message.into(),
            command: Some(command.into()),
            exit_code,
        }
    }

    /// Creates a new config error with a message
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
            config_path: None,
        }
    }

    /// Creates a new config error with a message and config path
    pub fn config_with_path<S: Into<String>, P: Into<String>>(message: S, config_path: P) -> Self {
        Self::Config {
            message: message.into(),
            config_path: Some(config_path.into()),
        }
    }

    /// Creates a new dependency error with a message
    pub fn dependency<S: Into<String>>(message: S) -> Self {
        Self::Dependency {
            message: message.into(),
            dependency: None,
        }
    }

    /// Creates a new dependency error with a message and dependency name
    pub fn dependency_with_name<S: Into<String>, D: Into<String>>(
        message: S,
        dependency: D,
    ) -> Self {
        Self::Dependency {
            message: message.into(),
            dependency: Some(dependency.into()),
        }
    }

    /// Creates a new initialization error with a message
    pub fn initialization<S: Into<String>>(message: S) -> Self {
        Self::Initialization {
            message: message.into(),
            component: None,
        }
    }

    /// Creates a new initialization error with a message and component name
    pub fn initialization_with_component<S: Into<String>, C: Into<String>>(
        message: S,
        component: C,
    ) -> Self {
        Self::Initialization {
            message: message.into(),
            component: Some(component.into()),
        }
    }

    /// Creates a new state error with a message
    pub fn state<S: Into<String>>(message: S) -> Self {
        Self::State {
            message: message.into(),
            operation: None,
        }
    }

    /// Creates a new state error with a message and operation
    pub fn state_with_operation<S: Into<String>, O: Into<String>>(
        message: S,
        operation: O,
    ) -> Self {
        Self::State {
            message: message.into(),
            operation: Some(operation.into()),
        }
    }

    /// Returns true if the error represents a retryable operation
    pub fn is_retryable(&self) -> bool {
        match self {
            CoreError::Shared(shared_err) => shared_err.is_retryable(),
            CoreError::Command { .. } => true, // Commands might be retryable
            CoreError::Config { .. } => false, // Config errors are not retryable
            CoreError::Module { .. } => false, // Module errors are not retryable
            CoreError::Dependency { .. } => false, // Dependency errors are not retryable
            CoreError::Initialization { .. } => false, // Init errors are not retryable
            CoreError::State { .. } => true,   // State errors might be retryable
        }
    }

    /// Returns true if the error represents a user input error
    pub fn is_user_error(&self) -> bool {
        match self {
            CoreError::Shared(shared_err) => shared_err.is_user_error(),
            CoreError::Config { .. } => true, // Config errors are often user errors
            _ => false,
        }
    }

    /// Returns an error code suitable for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            CoreError::Shared(shared_err) => shared_err.error_code(),
            CoreError::Module { .. } => "MODULE_ERROR",
            CoreError::Command { .. } => "COMMAND_ERROR",
            CoreError::Config { .. } => "CONFIG_ERROR",
            CoreError::Dependency { .. } => "DEPENDENCY_ERROR",
            CoreError::Initialization { .. } => "INITIALIZATION_ERROR",
            CoreError::State { .. } => "STATE_ERROR",
        }
    }

    /// Returns the underlying shared error if this is a shared error
    pub fn as_shared_error(&self) -> Option<&SharedError> {
        match self {
            CoreError::Shared(shared_err) => Some(shared_err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let module_error = CoreError::module_with_name("Invalid module format", "test-module");
        assert_eq!(module_error.error_code(), "MODULE_ERROR");
        assert!(!module_error.is_retryable());

        let command_error = CoreError::command_with_details("Command failed", "ls -la", Some(1));
        assert_eq!(command_error.error_code(), "COMMAND_ERROR");
        assert!(command_error.is_retryable());
    }

    #[test]
    fn test_shared_error_wrapping() {
        use shared_types::SharedError;

        let shared_error = SharedError::io("File not found");
        let core_error: CoreError = shared_error.into();

        assert!(matches!(core_error, CoreError::Shared(_)));
        assert_eq!(core_error.error_code(), "IO_ERROR");
        assert!(core_error.is_retryable());
    }

    #[test]
    fn test_error_display() {
        let error = CoreError::config_with_path("Invalid configuration", "config.json");
        assert_eq!(
            error.to_string(),
            "Configuration management error: Invalid configuration"
        );
    }

    #[test]
    fn test_error_classification() {
        let config_error = CoreError::config("Invalid config");
        assert!(config_error.is_user_error());
        assert!(!config_error.is_retryable());

        let state_error = CoreError::state("State inconsistency");
        assert!(!state_error.is_user_error());
        assert!(state_error.is_retryable());
    }

    #[test]
    fn test_as_shared_error() {
        use shared_types::SharedError;

        let shared_error = SharedError::validation("Invalid input");
        let core_error = CoreError::Shared(shared_error.clone());

        assert_eq!(core_error.as_shared_error(), Some(&shared_error));

        let module_error = CoreError::module("Module error");
        assert_eq!(module_error.as_shared_error(), None);
    }
}
