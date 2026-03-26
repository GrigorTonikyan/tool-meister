//! Foundational error types shared across all crates in the workspace.
//!
//! This module provides the base error types that can be used by all crates
//! to ensure consistent error handling and conversion patterns throughout
//! the application architecture.

use thiserror::Error;

/// Foundational error type shared across all crates in the workspace.
///
/// This enum contains common error variants that can occur in any crate.
/// Each crate can wrap this error type and add their own specific variants
/// while maintaining the ability to convert between error types.
///
/// # Design Principles
/// - Maintains error chain context for debugging
/// - Provides user-friendly display messages
/// - Enables ergonomic error conversion between layers
/// - Follows security guidelines for error information exposure
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SharedError {
    /// IO-related errors (file operations, network, etc.)
    #[error("IO operation failed: {message}")]
    Io {
        /// Human-readable error message
        message: String,
        /// Optional error code for categorization
        code: Option<String>,
    },

    /// Parsing or serialization errors
    #[error("Parse error: {message}")]
    Parse {
        /// Human-readable error message
        message: String,
        /// File or data source where parsing failed
        source_name: Option<String>,
    },

    /// Validation errors for input data or configuration
    #[error("Validation failed: {message}")]
    Validation {
        /// Human-readable error message
        message: String,
        /// Field or property that failed validation
        field: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        /// Human-readable error message
        message: String,
        /// Optional status code or error code
        code: Option<u16>,
    },

    /// Permission or access-related errors
    #[error("Permission denied: {message}")]
    Permission {
        /// Human-readable error message
        message: String,
        /// Resource that access was denied to
        resource: Option<String>,
    },

    /// Timeout errors for operations that exceeded time limits
    #[error("Operation timed out: {message}")]
    Timeout {
        /// Human-readable error message
        message: String,
        /// Duration that was exceeded (in seconds)
        duration: Option<u64>,
    },

    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Human-readable error message
        message: String,
        /// Configuration key or section that caused the error
        key: Option<String>,
    },

    /// Security-related errors for command validation
    #[error("Security error: {message}")]
    Security {
        /// Human-readable error message
        message: String,
        /// Security rule that was violated
        rule: Option<String>,
    },

    /// Dependency-related errors
    #[error("Dependency error: {message}")]
    Dependency {
        /// Human-readable error message
        message: String,
        /// Dependency name that failed
        dependency: Option<String>,
    },

    /// Generic internal errors that don't fit other categories
    #[error("Internal error: {message}")]
    Internal {
        /// Human-readable error message
        message: String,
    },
}

/// Result type alias for operations that return SharedError
pub type SharedResult<T> = Result<T, SharedError>;

impl SharedError {
    /// Creates a new IO error with a message
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io {
            message: message.into(),
            code: None,
        }
    }

    /// Creates a new IO error with a message and code
    pub fn io_with_code<S: Into<String>, C: Into<String>>(message: S, code: C) -> Self {
        Self::Io {
            message: message.into(),
            code: Some(code.into()),
        }
    }

    /// Creates a new parse error with a message
    pub fn parse<S: Into<String>>(message: S) -> Self {
        Self::Parse {
            message: message.into(),
            source_name: None,
        }
    }

    /// Creates a new parse error with a message and source
    pub fn parse_with_source<S: Into<String>, R: Into<String>>(message: S, source: R) -> Self {
        Self::Parse {
            message: message.into(),
            source_name: Some(source.into()),
        }
    }

    /// Creates a new validation error with a message
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
        }
    }

    /// Creates a new validation error with a message and field
    pub fn validation_with_field<S: Into<String>, F: Into<String>>(message: S, field: F) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Creates a new network error with a message
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            code: None,
        }
    }

    /// Creates a new network error with a message and status code
    pub fn network_with_code<S: Into<String>>(message: S, code: u16) -> Self {
        Self::Network {
            message: message.into(),
            code: Some(code),
        }
    }

    /// Creates a new permission error with a message
    pub fn permission<S: Into<String>>(message: S) -> Self {
        Self::Permission {
            message: message.into(),
            resource: None,
        }
    }

    /// Creates a new permission error with a message and resource
    pub fn permission_with_resource<S: Into<String>, R: Into<String>>(
        message: S,
        resource: R,
    ) -> Self {
        Self::Permission {
            message: message.into(),
            resource: Some(resource.into()),
        }
    }

    /// Creates a new timeout error with a message
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout {
            message: message.into(),
            duration: None,
        }
    }

    /// Creates a new timeout error with a message and duration
    pub fn timeout_with_duration<S: Into<String>>(message: S, duration: u64) -> Self {
        Self::Timeout {
            message: message.into(),
            duration: Some(duration),
        }
    }

    /// Creates a new configuration error with a message
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            key: None,
        }
    }

    /// Creates a new configuration error with a message and key
    pub fn configuration_with_key<S: Into<String>, K: Into<String>>(message: S, key: K) -> Self {
        Self::Configuration {
            message: message.into(),
            key: Some(key.into()),
        }
    }

    /// Creates a new security error with a message
    pub fn security<S: Into<String>>(message: S) -> Self {
        Self::Security {
            message: message.into(),
            rule: None,
        }
    }

    /// Creates a new security error with a message and rule
    pub fn security_with_rule<S: Into<String>, R: Into<String>>(message: S, rule: R) -> Self {
        Self::Security {
            message: message.into(),
            rule: Some(rule.into()),
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

    /// Creates a new internal error with a message
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Returns true if this error represents a retryable operation
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SharedError::Network { .. } | SharedError::Timeout { .. } | SharedError::Io { .. }
        )
    }

    /// Returns true if this error represents a user input error
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            SharedError::Validation { .. } | SharedError::Parse { .. }
        )
    }

    /// Returns true if this error represents a system/permission issue
    pub fn is_system_error(&self) -> bool {
        matches!(
            self,
            SharedError::Permission { .. } | SharedError::Security { .. }
        )
    }

    /// Returns an error code suitable for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            SharedError::Io { .. } => "IO_ERROR",
            SharedError::Parse { .. } => "PARSE_ERROR",
            SharedError::Validation { .. } => "VALIDATION_ERROR",
            SharedError::Network { .. } => "NETWORK_ERROR",
            SharedError::Permission { .. } => "PERMISSION_ERROR",
            SharedError::Timeout { .. } => "TIMEOUT_ERROR",
            SharedError::Configuration { .. } => "CONFIGURATION_ERROR",
            SharedError::Security { .. } => "SECURITY_ERROR",
            SharedError::Dependency { .. } => "DEPENDENCY_ERROR",
            SharedError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

// Standard conversions for common error types
impl From<std::io::Error> for SharedError {
    fn from(err: std::io::Error) -> Self {
        SharedError::io(err.to_string())
    }
}

impl From<serde_json::Error> for SharedError {
    fn from(err: serde_json::Error) -> Self {
        SharedError::parse(format!("JSON error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let io_error = SharedError::io("File not found");
        assert_eq!(io_error.error_code(), "IO_ERROR");
        assert!(io_error.is_retryable());
        assert!(!io_error.is_user_error());

        let validation_error = SharedError::validation_with_field("Invalid format", "email");
        assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");
        assert!(!validation_error.is_retryable());
        assert!(validation_error.is_user_error());
    }

    #[test]
    fn test_error_display() {
        let error = SharedError::parse_with_source("Invalid syntax", "config.json");
        assert_eq!(error.to_string(), "Parse error: Invalid syntax");
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let shared_error: SharedError = io_error.into();
        assert!(matches!(shared_error, SharedError::Io { .. }));
    }

    #[test]
    fn test_error_classification() {
        assert!(SharedError::network("Connection failed").is_retryable());
        assert!(SharedError::validation("Invalid input").is_user_error());
        assert!(SharedError::permission("Access denied").is_system_error());
    }

    #[test]
    fn test_configuration_error() {
        let error = SharedError::configuration_with_key("Invalid value", "database.port");
        assert_eq!(error.to_string(), "Configuration error: Invalid value");
        assert_eq!(error.error_code(), "CONFIGURATION_ERROR");
    }
}
