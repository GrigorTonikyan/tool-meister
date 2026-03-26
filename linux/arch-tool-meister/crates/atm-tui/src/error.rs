//! TUI-specific error types that wrap CoreError and add UI-specific context.
//!
//! This module defines error types specific to the terminal user interface,
//! wrapping the core library errors with additional UI context and user-friendly
//! messages for terminal display.

use core_lib::error::CoreError;
use thiserror::Error;

/// TUI-specific error type that wraps CoreError and adds UI-specific context.
///
/// This error type represents all errors that can occur within the TUI application.
/// It wraps CoreError to maintain the error chain while adding UI-specific variants
/// and user-friendly display messages suitable for terminal interfaces.
///
/// # Design Principles
/// - Wraps CoreError for consistent error handling across layers
/// - Provides user-friendly error messages suitable for terminal display
/// - Adds UI-specific error variants for rendering and input handling
/// - Maintains error chain for debugging while providing actionable feedback
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TuiError {
    /// Core library error wrapped with TUI context
    #[error(transparent)]
    Core(#[from] CoreError),

    /// Terminal rendering and display errors
    #[error("Terminal rendering error: {message}")]
    Rendering {
        /// Human-readable error message
        message: String,
        /// Component that failed to render
        component: Option<String>,
    },

    /// User input and interaction errors
    #[error("User input error: {message}")]
    Input {
        /// Human-readable error message
        message: String,
        /// Input key or sequence that caused the error
        input: Option<String>,
    },

    /// Terminal initialization and setup errors
    #[error("Terminal initialization error: {message}")]
    Terminal {
        /// Human-readable error message
        message: String,
        /// Terminal capability that failed
        capability: Option<String>,
    },

    /// UI state management errors
    #[error("UI state error: {message}")]
    State {
        /// Human-readable error message
        message: String,
        /// UI state that caused the error
        state: Option<String>,
    },

    /// Theme and styling errors
    #[error("Theme error: {message}")]
    Theme {
        /// Human-readable error message
        message: String,
        /// Theme element that failed
        element: Option<String>,
    },

    /// Layout and sizing errors
    #[error("Layout error: {message}")]
    Layout {
        /// Human-readable error message
        message: String,
        /// Layout constraint or area that failed
        constraint: Option<String>,
    },
}

/// Result type alias for TUI operations
pub type TuiResult<T> = Result<T, TuiError>;

impl TuiError {
    /// Creates a new rendering error with a message
    pub fn rendering<S: Into<String>>(message: S) -> Self {
        Self::Rendering {
            message: message.into(),
            component: None,
        }
    }

    /// Creates a new rendering error with a message and component
    pub fn rendering_with_component<S: Into<String>, C: Into<String>>(
        message: S,
        component: C,
    ) -> Self {
        Self::Rendering {
            message: message.into(),
            component: Some(component.into()),
        }
    }

    /// Creates a new input error with a message
    pub fn input<S: Into<String>>(message: S) -> Self {
        Self::Input {
            message: message.into(),
            input: None,
        }
    }

    /// Creates a new input error with a message and input details
    pub fn input_with_details<S: Into<String>, I: Into<String>>(message: S, input: I) -> Self {
        Self::Input {
            message: message.into(),
            input: Some(input.into()),
        }
    }

    /// Creates a new terminal error with a message
    pub fn terminal<S: Into<String>>(message: S) -> Self {
        Self::Terminal {
            message: message.into(),
            capability: None,
        }
    }

    /// Creates a new terminal error with a message and capability
    pub fn terminal_with_capability<S: Into<String>, C: Into<String>>(
        message: S,
        capability: C,
    ) -> Self {
        Self::Terminal {
            message: message.into(),
            capability: Some(capability.into()),
        }
    }

    /// Creates a new state error with a message
    pub fn state<S: Into<String>>(message: S) -> Self {
        Self::State {
            message: message.into(),
            state: None,
        }
    }

    /// Creates a new state error with a message and state details
    pub fn state_with_details<S: Into<String>, St: Into<String>>(message: S, state: St) -> Self {
        Self::State {
            message: message.into(),
            state: Some(state.into()),
        }
    }

    /// Creates a new theme error with a message
    pub fn theme<S: Into<String>>(message: S) -> Self {
        Self::Theme {
            message: message.into(),
            element: None,
        }
    }

    /// Creates a new theme error with a message and element
    pub fn theme_with_element<S: Into<String>, E: Into<String>>(message: S, element: E) -> Self {
        Self::Theme {
            message: message.into(),
            element: Some(element.into()),
        }
    }

    /// Creates a new layout error with a message
    pub fn layout<S: Into<String>>(message: S) -> Self {
        Self::Layout {
            message: message.into(),
            constraint: None,
        }
    }

    /// Creates a new layout error with a message and constraint
    pub fn layout_with_constraint<S: Into<String>, C: Into<String>>(
        message: S,
        constraint: C,
    ) -> Self {
        Self::Layout {
            message: message.into(),
            constraint: Some(constraint.into()),
        }
    }

    /// Returns true if the error represents a retryable operation
    pub fn is_retryable(&self) -> bool {
        match self {
            TuiError::Core(core_err) => core_err.is_retryable(),
            TuiError::Rendering { .. } => true, // Rendering might be retryable
            TuiError::Input { .. } => false,    // Input errors are not retryable
            TuiError::Terminal { .. } => false, // Terminal errors are not retryable
            TuiError::State { .. } => true,     // State errors might be retryable
            TuiError::Theme { .. } => false,    // Theme errors are not retryable
            TuiError::Layout { .. } => true,    // Layout errors might be retryable
        }
    }

    /// Returns true if the error represents a user input error
    pub fn is_user_error(&self) -> bool {
        match self {
            TuiError::Core(core_err) => core_err.is_user_error(),
            TuiError::Input { .. } => true, // Input errors are user errors
            TuiError::Theme { .. } => true, // Theme errors might be user errors
            _ => false,
        }
    }

    /// Returns true if the error should be shown to the user in the UI
    pub fn should_display_to_user(&self) -> bool {
        match self {
            TuiError::Input { .. } => true,
            TuiError::Theme { .. } => true,
            TuiError::Layout { .. } => false, // Layout errors are usually internal
            TuiError::Core(core_err) => core_err.is_user_error(),
            _ => true, // Most TUI errors should be shown to the user
        }
    }

    /// Returns an error code suitable for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            TuiError::Core(core_err) => core_err.error_code(),
            TuiError::Rendering { .. } => "TUI_RENDERING_ERROR",
            TuiError::Input { .. } => "TUI_INPUT_ERROR",
            TuiError::Terminal { .. } => "TUI_TERMINAL_ERROR",
            TuiError::State { .. } => "TUI_STATE_ERROR",
            TuiError::Theme { .. } => "TUI_THEME_ERROR",
            TuiError::Layout { .. } => "TUI_LAYOUT_ERROR",
        }
    }

    /// Returns the underlying core error if this is a core error
    pub fn as_core_error(&self) -> Option<&CoreError> {
        match self {
            TuiError::Core(core_err) => Some(core_err),
            _ => None,
        }
    }

    /// Returns a user-friendly message suitable for display in the terminal
    pub fn user_message(&self) -> String {
        match self {
            TuiError::Core(core_err) => match core_err {
                CoreError::Module {
                    message,
                    module_name,
                } => {
                    format!("Module Error: {}", message)
                        + &module_name
                            .as_ref()
                            .map(|name| format!(" ({})", name))
                            .unwrap_or_default()
                }
                CoreError::Command {
                    message, command, ..
                } => {
                    format!("Command Failed: {}", message)
                        + &command
                            .as_ref()
                            .map(|cmd| format!(" ({})", cmd))
                            .unwrap_or_default()
                }
                _ => core_err.to_string(),
            },
            TuiError::Rendering { message, component } => {
                format!("Display Error: {}", message)
                    + &component
                        .as_ref()
                        .map(|comp| format!(" ({})", comp))
                        .unwrap_or_default()
            }
            TuiError::Input { message, input } => {
                format!("Input Error: {}", message)
                    + &input
                        .as_ref()
                        .map(|inp| format!(" ({})", inp))
                        .unwrap_or_default()
            }
            TuiError::Terminal {
                message,
                capability,
            } => {
                format!("Terminal Error: {}", message)
                    + &capability
                        .as_ref()
                        .map(|cap| format!(" ({})", cap))
                        .unwrap_or_default()
            }
            _ => self.to_string(),
        }
    }

    /// Returns suggested recovery actions for the user
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            TuiError::Core(core_err) => match core_err {
                CoreError::Module { .. } => vec![
                    "Check module configuration files".to_string(),
                    "Verify module directory structure".to_string(),
                    "Review module documentation".to_string(),
                ],
                CoreError::Command { .. } => vec![
                    "Check if the command exists".to_string(),
                    "Verify command permissions".to_string(),
                    "Try running the command manually".to_string(),
                ],
                CoreError::Config { .. } => vec![
                    "Check configuration file syntax".to_string(),
                    "Verify file permissions".to_string(),
                    "Reset to default configuration".to_string(),
                ],
                _ => vec!["Check application logs for more details".to_string()],
            },
            TuiError::Terminal { .. } => vec![
                "Check terminal compatibility".to_string(),
                "Try a different terminal emulator".to_string(),
                "Verify terminal size requirements".to_string(),
            ],
            TuiError::Input { .. } => vec![
                "Review available key bindings".to_string(),
                "Check input method configuration".to_string(),
            ],
            TuiError::Rendering { .. } => vec![
                "Try resizing the terminal".to_string(),
                "Check terminal color support".to_string(),
                "Verify theme configuration".to_string(),
            ],
            _ => vec!["Restart the application".to_string()],
        }
    }
}

// Ergonomic error conversion implementations
impl From<shared_types::SharedError> for TuiError {
    fn from(err: shared_types::SharedError) -> Self {
        TuiError::Core(CoreError::Shared(err))
    }
}

impl From<std::io::Error> for TuiError {
    fn from(err: std::io::Error) -> Self {
        let shared_error = shared_types::SharedError::from(err);
        TuiError::from(shared_error)
    }
}

impl From<serde_json::Error> for TuiError {
    fn from(err: serde_json::Error) -> Self {
        let shared_error = shared_types::SharedError::from(err);
        TuiError::from(shared_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_lib::error::CoreError;
    use shared_types::SharedError;

    #[test]
    fn test_error_creation() {
        let rendering_error = TuiError::rendering_with_component("Failed to render", "menu");
        assert_eq!(rendering_error.error_code(), "TUI_RENDERING_ERROR");
        assert!(rendering_error.is_retryable());
        assert!(rendering_error.should_display_to_user());

        let input_error = TuiError::input_with_details("Invalid key", "Ctrl+X");
        assert_eq!(input_error.error_code(), "TUI_INPUT_ERROR");
        assert!(!input_error.is_retryable());
        assert!(input_error.is_user_error());
    }

    #[test]
    fn test_core_error_wrapping() {
        let core_error = CoreError::module_with_name("Invalid module", "test-module");
        let tui_error: TuiError = core_error.into();

        assert!(matches!(tui_error, TuiError::Core(_)));
        assert_eq!(tui_error.error_code(), "MODULE_ERROR");
        assert_eq!(
            tui_error.as_core_error().unwrap().error_code(),
            "MODULE_ERROR"
        );
    }

    #[test]
    fn test_shared_error_chain() {
        let shared_error = SharedError::io("File not found");
        let core_error = CoreError::Shared(shared_error);
        let tui_error = TuiError::Core(core_error);

        assert_eq!(tui_error.error_code(), "IO_ERROR");
        assert!(tui_error.is_retryable());
    }

    #[test]
    fn test_user_message_formatting() {
        let error = TuiError::rendering_with_component("Widget overflow", "status_bar");
        let message = error.user_message();
        assert!(message.contains("Display Error"));
        assert!(message.contains("Widget overflow"));
        assert!(message.contains("status_bar"));
    }

    #[test]
    fn test_recovery_suggestions() {
        let terminal_error = TuiError::terminal("Unsupported terminal");
        let suggestions = terminal_error.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("terminal")));
    }

    #[test]
    fn test_error_classification() {
        assert!(TuiError::input("Invalid input").is_user_error());
        assert!(!TuiError::layout("Layout calculation failed").should_display_to_user());
        assert!(TuiError::state("State error").is_retryable());
    }
}
