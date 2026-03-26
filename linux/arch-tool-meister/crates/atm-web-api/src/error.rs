//! Web API-specific error types that wrap CoreError and convert to HTTP responses.
//!
//! This module defines error types specific to the web API server, wrapping
//! the core library errors with HTTP status codes and structured error responses
//! suitable for REST API clients.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use core_lib::CoreError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Web API-specific error type that wraps CoreError and provides HTTP response conversion.
///
/// This error type represents all errors that can occur within the web API operations.
/// It wraps CoreError to maintain the error chain while providing appropriate HTTP
/// status codes and structured JSON error responses for API clients.
///
/// # Design Principles
/// - Wraps CoreError for consistent error handling across layers
/// - Provides appropriate HTTP status codes for different error types
/// - Generates structured JSON error responses
/// - Maintains security by not exposing internal implementation details
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ApiError {
    /// Core library error wrapped with API context
    #[error(transparent)]
    Core(#[from] CoreError),

    /// HTTP request validation errors
    #[error("Request validation error: {message}")]
    Validation {
        /// Human-readable error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Validation rule that was violated
        rule: Option<String>,
    },

    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    Authentication {
        /// Human-readable error message
        message: String,
        /// Authentication method that failed
        method: Option<String>,
    },

    /// Authorization errors
    #[error("Authorization error: {message}")]
    Authorization {
        /// Human-readable error message
        message: String,
        /// Resource that access was denied to
        resource: Option<String>,
    },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        /// Human-readable error message
        message: String,
        /// Time until rate limit resets (in seconds)
        reset_after: Option<u64>,
    },

    /// Request payload errors
    #[error("Request payload error: {message}")]
    Payload {
        /// Human-readable error message
        message: String,
        /// Maximum allowed size if applicable
        max_size: Option<usize>,
    },

    /// Resource not found errors
    #[error("Resource not found: {message}")]
    NotFound {
        /// Human-readable error message
        message: String,
        /// Resource type that was not found
        resource_type: Option<String>,
        /// Resource identifier that was not found
        resource_id: Option<String>,
    },

    /// Resource conflict errors (e.g., already exists)
    #[error("Resource conflict: {message}")]
    Conflict {
        /// Human-readable error message
        message: String,
        /// Resource that conflicts
        resource: Option<String>,
    },

    /// API server internal errors
    #[error("Internal server error: {message}")]
    Internal {
        /// Human-readable error message
        message: String,
        /// Internal error ID for tracking
        error_id: Option<String>,
    },
}

/// Structured error response for API clients
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<ErrorDetails>,
    /// Request tracking ID
    pub request_id: Option<String>,
}

/// Additional error details for specific error types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ErrorDetails {
    /// Validation error details
    Validation {
        field: Option<String>,
        rule: Option<String>,
    },
    /// Rate limit error details
    RateLimit { reset_after: Option<u64> },
    /// Resource error details
    Resource {
        resource_type: Option<String>,
        resource_id: Option<String>,
    },
    /// Generic string details
    Generic(String),
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    /// Creates a new validation error with a message
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            rule: None,
        }
    }

    /// Creates a new validation error with field and rule details
    pub fn validation_with_details<S: Into<String>, F: Into<String>, R: Into<String>>(
        message: S,
        field: F,
        rule: R,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.into()),
            rule: Some(rule.into()),
        }
    }

    /// Creates a new authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
            method: None,
        }
    }

    /// Creates a new authentication error with method details
    pub fn authentication_with_method<S: Into<String>, M: Into<String>>(
        message: S,
        method: M,
    ) -> Self {
        Self::Authentication {
            message: message.into(),
            method: Some(method.into()),
        }
    }

    /// Creates a new authorization error
    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization {
            message: message.into(),
            resource: None,
        }
    }

    /// Creates a new authorization error with resource details
    pub fn authorization_with_resource<S: Into<String>, R: Into<String>>(
        message: S,
        resource: R,
    ) -> Self {
        Self::Authorization {
            message: message.into(),
            resource: Some(resource.into()),
        }
    }

    /// Creates a new rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit {
            message: message.into(),
            reset_after: None,
        }
    }

    /// Creates a new rate limit error with reset time
    pub fn rate_limit_with_reset<S: Into<String>>(message: S, reset_after: u64) -> Self {
        Self::RateLimit {
            message: message.into(),
            reset_after: Some(reset_after),
        }
    }

    /// Creates a new not found error
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::NotFound {
            message: message.into(),
            resource_type: None,
            resource_id: None,
        }
    }

    /// Creates a new not found error with resource details
    pub fn not_found_with_details<S: Into<String>, T: Into<String>, I: Into<String>>(
        message: S,
        resource_type: T,
        resource_id: I,
    ) -> Self {
        Self::NotFound {
            message: message.into(),
            resource_type: Some(resource_type.into()),
            resource_id: Some(resource_id.into()),
        }
    }

    /// Creates a new conflict error
    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
            resource: None,
        }
    }

    /// Creates a new conflict error with resource details
    pub fn conflict_with_resource<S: Into<String>, R: Into<String>>(
        message: S,
        resource: R,
    ) -> Self {
        Self::Conflict {
            message: message.into(),
            resource: Some(resource.into()),
        }
    }

    /// Creates a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
            error_id: None,
        }
    }

    /// Creates a new internal error with tracking ID
    pub fn internal_with_id<S: Into<String>, I: Into<String>>(message: S, error_id: I) -> Self {
        Self::Internal {
            message: message.into(),
            error_id: Some(error_id.into()),
        }
    }

    /// Returns the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Core(core_err) => match core_err {
                CoreError::Shared(shared_err) => match shared_err {
                    shared_types::SharedError::Validation { .. } => StatusCode::BAD_REQUEST,
                    shared_types::SharedError::Permission { .. } => StatusCode::FORBIDDEN,
                    shared_types::SharedError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
                    shared_types::SharedError::Configuration { .. } => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                CoreError::Module { .. } => StatusCode::NOT_FOUND,
                CoreError::Command { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                CoreError::Config { .. } => StatusCode::BAD_REQUEST,
                CoreError::Dependency { .. } => StatusCode::FAILED_DEPENDENCY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ApiError::Validation { .. } => StatusCode::BAD_REQUEST,
            ApiError::Authentication { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Authorization { .. } => StatusCode::FORBIDDEN,
            ApiError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Payload { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
            ApiError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns an error code suitable for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::Core(core_err) => core_err.error_code(),
            ApiError::Validation { .. } => "VALIDATION_ERROR",
            ApiError::Authentication { .. } => "AUTHENTICATION_ERROR",
            ApiError::Authorization { .. } => "AUTHORIZATION_ERROR",
            ApiError::RateLimit { .. } => "RATE_LIMIT_ERROR",
            ApiError::Payload { .. } => "PAYLOAD_ERROR",
            ApiError::NotFound { .. } => "NOT_FOUND_ERROR",
            ApiError::Conflict { .. } => "CONFLICT_ERROR",
            ApiError::Internal { .. } => "INTERNAL_ERROR",
        }
    }

    /// Returns the underlying core error if this is a core error
    pub fn as_core_error(&self) -> Option<&CoreError> {
        match self {
            ApiError::Core(core_err) => Some(core_err),
            _ => None,
        }
    }

    /// Returns error details for the response
    pub fn error_details(&self) -> Option<ErrorDetails> {
        match self {
            ApiError::Validation { field, rule, .. } => Some(ErrorDetails::Validation {
                field: field.clone(),
                rule: rule.clone(),
            }),
            ApiError::RateLimit { reset_after, .. } => Some(ErrorDetails::RateLimit {
                reset_after: *reset_after,
            }),
            ApiError::NotFound {
                resource_type,
                resource_id,
                ..
            } => Some(ErrorDetails::Resource {
                resource_type: resource_type.clone(),
                resource_id: resource_id.clone(),
            }),
            _ => None,
        }
    }

    /// Creates an ErrorResponse structure for this error
    pub fn to_error_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            details: self.error_details(),
            request_id,
        }
    }
}

// Implement IntoResponse for Axum integration
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response(None); // TODO: Add request ID from context

        (status, Json(error_response)).into_response()
    }
}

// Standard conversions for common error types
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::validation(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        let core_error = CoreError::from(shared_types::SharedError::from(err));
        ApiError::Core(core_error)
    }
}

impl From<shared_types::SharedError> for ApiError {
    fn from(err: shared_types::SharedError) -> Self {
        ApiError::Core(CoreError::Shared(err))
    }
}

// Axum-specific error conversions
impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(err: axum::extract::rejection::JsonRejection) -> Self {
        ApiError::validation(format!("Invalid JSON payload: {}", err))
    }
}

impl From<axum::extract::rejection::QueryRejection> for ApiError {
    fn from(err: axum::extract::rejection::QueryRejection) -> Self {
        ApiError::validation(format!("Invalid query parameters: {}", err))
    }
}

impl From<axum::extract::rejection::PathRejection> for ApiError {
    fn from(err: axum::extract::rejection::PathRejection) -> Self {
        ApiError::validation(format!("Invalid path parameters: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_lib::CoreError;
    use shared_types::SharedError;

    #[test]
    fn test_status_code_mapping() {
        assert_eq!(
            ApiError::validation("test").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::authentication("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::authorization("test").status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ApiError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::internal("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_response_creation() {
        let error = ApiError::validation_with_details("Invalid input", "email", "format");
        let response = error.to_error_response(Some("req-123".to_string()));

        assert_eq!(response.error, "VALIDATION_ERROR");
        assert!(response.message.contains("Invalid input"));
        assert!(response.details.is_some());
        assert_eq!(response.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_core_error_wrapping() {
        let shared_error = SharedError::permission("Access denied");
        let core_error = CoreError::Shared(shared_error);
        let api_error = ApiError::Core(core_error);

        assert_eq!(api_error.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(api_error.error_code(), "PERMISSION_ERROR");
    }

    #[test]
    fn test_error_details() {
        let validation_error =
            ApiError::validation_with_details("Invalid format", "email", "email_format");

        if let Some(ErrorDetails::Validation { field, rule }) = validation_error.error_details() {
            assert_eq!(field, Some("email".to_string()));
            assert_eq!(rule, Some("email_format".to_string()));
        } else {
            panic!("Expected validation error details");
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{ invalid json";
        let json_error: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let api_error: ApiError = json_error.unwrap_err().into();

        assert_eq!(api_error.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(api_error.error_code(), "VALIDATION_ERROR");
    }
}
