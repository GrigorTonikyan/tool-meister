use std::env;
use std::fmt;

use color_eyre::Result;
use tracing::error;

/// Application-specific error types with user-friendly messages
#[derive(Debug)]
pub enum AtmError {
    /// Configuration file related errors
    ConfigError {
        file_path: String,
        message: String,
        suggestion: String,
    },
    /// Module related errors
    ModuleError {
        module_name: String,
        message: String,
        suggestion: String,
    },
    /// Command execution errors
    CommandError {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
        suggestion: String,
    },
    /// File I/O errors
    FileError {
        operation: String,
        file_path: String,
        message: String,
        suggestion: String,
    },
    /// JSON parsing errors
    JsonError {
        file_path: String,
        line: Option<usize>,
        message: String,
        suggestion: String,
    },
    /// TUI related errors
    TuiError { message: String, suggestion: String },
}

impl fmt::Display for AtmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtmError::ConfigError {
                file_path,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "Configuration Error in '{}': {}\n💡 Suggestion: {}",
                    file_path, message, suggestion
                )
            }
            AtmError::ModuleError {
                module_name,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "Module Error in '{}': {}\n💡 Suggestion: {}",
                    module_name, message, suggestion
                )
            }
            AtmError::CommandError {
                command,
                exit_code,
                stderr,
                suggestion,
            } => {
                let exit_info = match exit_code {
                    Some(code) => format!(" (exit code: {})", code),
                    None => String::new(),
                };
                write!(
                    f,
                    "Command Failed: '{}'{}\nError output: {}\n💡 Suggestion: {}",
                    command, exit_info, stderr, suggestion
                )
            }
            AtmError::FileError {
                operation,
                file_path,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "File Error during '{}' on '{}': {}\n💡 Suggestion: {}",
                    operation, file_path, message, suggestion
                )
            }
            AtmError::JsonError {
                file_path,
                line,
                message,
                suggestion,
            } => {
                let line_info = match line {
                    Some(l) => format!(" at line {}", l),
                    None => String::new(),
                };
                write!(
                    f,
                    "JSON Parsing Error in '{}'{}: {}\n💡 Suggestion: {}",
                    file_path, line_info, message, suggestion
                )
            }
            AtmError::TuiError {
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "Terminal UI Error: {}\n💡 Suggestion: {}",
                    message, suggestion
                )
            }
        }
    }
}

impl std::error::Error for AtmError {}

// Implement conversion from common error types
impl From<std::io::Error> for AtmError {
    fn from(err: std::io::Error) -> Self {
        AtmError::FileError {
            operation: "file operation".to_string(),
            file_path: "unknown".to_string(),
            message: err.to_string(),
            suggestion: "Check if the file exists and you have proper permissions".to_string(),
        }
    }
}

impl From<serde_json::Error> for AtmError {
    fn from(err: serde_json::Error) -> Self {
        AtmError::JsonError {
            file_path: "unknown".to_string(),
            line: Some(err.line()),
            message: err.to_string(),
            suggestion: "Check JSON syntax - ensure all brackets and quotes are properly closed"
                .to_string(),
        }
    }
}

/// Helper functions for creating user-friendly errors
impl AtmError {
    pub fn config_not_found(file_path: &str) -> Self {
        Self::ConfigError {
            file_path: file_path.to_string(),
            message: "Configuration file not found".to_string(),
            suggestion: format!("Ensure the configuration file exists at '{}' or run the application from the correct directory", file_path),
        }
    }

    pub fn config_invalid_json(file_path: &str, json_error: &str) -> Self {
        Self::JsonError {
            file_path: file_path.to_string(),
            line: None,
            message: format!("Invalid JSON format: {}", json_error),
            suggestion: "Check the JSON syntax, ensure all brackets and quotes are properly closed, and validate against a JSON checker".to_string(),
        }
    }

    pub fn module_not_found(module_name: &str) -> Self {
        Self::ModuleError {
            module_name: module_name.to_string(),
            message: "Module configuration not found".to_string(),
            suggestion: format!("Ensure the module '{}' exists in the modules directory and has a valid config.jsonc file", module_name),
        }
    }

    pub fn command_execution_failed(command: &str, exit_code: Option<i32>, stderr: &str) -> Self {
        let suggestion = match command {
            cmd if cmd.contains("code") => "Ensure VS Code is installed and in your PATH. Try 'which code' to verify the installation".to_string(),
            cmd if cmd.contains("git") => "Ensure Git is installed and configured. Check 'git --version' and 'git config --list'".to_string(),
            cmd if cmd.contains("pacman") || cmd.contains("paru") || cmd.contains("yay") => 
                "Ensure you have the necessary permissions and the package manager is available".to_string(),
            _ => "Check if the command exists in your PATH and you have the necessary permissions".to_string(),
        };

        Self::CommandError {
            command: command.to_string(),
            exit_code,
            stderr: stderr.to_string(),
            suggestion,
        }
    }

    pub fn file_operation_failed(
        operation: &str,
        file_path: &str,
        io_error: &std::io::Error,
    ) -> Self {
        let suggestion = match io_error.kind() {
            std::io::ErrorKind::NotFound => {
                format!("Ensure the file or directory '{}' exists", file_path)
            }
            std::io::ErrorKind::PermissionDenied => format!(
                "Check file permissions for '{}'. You may need to run with elevated privileges",
                file_path
            ),
            std::io::ErrorKind::AlreadyExists => format!(
                "File '{}' already exists. Choose a different name or remove the existing file",
                file_path
            ),
            _ => "Check the file path and your system's file access permissions".to_string(),
        };

        Self::FileError {
            operation: operation.to_string(),
            file_path: file_path.to_string(),
            message: io_error.to_string(),
            suggestion,
        }
    }

    pub fn tui_initialization_failed(tui_error: &str) -> Self {
        Self::TuiError {
            message: format!("Failed to initialize terminal interface: {}", tui_error),
            suggestion: "Ensure your terminal supports the required features. Try running in a different terminal or check TERM environment variable".to_string(),
        }
    }

    /// Get recovery suggestions based on error type and context
    pub fn get_recovery_steps(&self) -> Vec<String> {
        match self {
            AtmError::ConfigError { file_path, .. } => {
                vec![
                    "1. Check if the configuration file exists".to_string(),
                    format!("2. Verify permissions for: {}", file_path),
                    "3. Validate JSON syntax with 'jsonlint' or online validator".to_string(),
                    "4. Compare with working example in repository".to_string(),
                    "5. Reset to default configuration if needed".to_string(),
                ]
            }
            AtmError::ModuleError { module_name, .. } => {
                vec![
                    format!("1. Verify module '{}' directory exists", module_name),
                    "2. Check for required files: config.jsonc, menu.jsonc, commands.jsonc"
                        .to_string(),
                    "3. Validate JSON syntax in all module files".to_string(),
                    "4. Ensure module is properly configured and enabled".to_string(),
                    "5. Try running: './arch-tool-meister --list-modules'".to_string(),
                ]
            }
            AtmError::CommandError { command, .. } => {
                if command.contains("code") {
                    vec![
                        "1. Install VS Code: Download from https://code.visualstudio.com/"
                            .to_string(),
                        "2. Add VS Code to PATH: export PATH=\"$PATH:/usr/bin\"".to_string(),
                        "3. Verify installation: 'which code' or 'code --version'".to_string(),
                        "4. For Arch Linux: 'sudo pacman -S code'".to_string(),
                    ]
                } else if command.contains("git") {
                    vec![
                        "1. Install Git: 'sudo pacman -S git'".to_string(),
                        "2. Configure Git: 'git config --global user.name \"Your Name\"'".to_string(),
                        "3. Configure Git: 'git config --global user.email \"your.email@example.com\"'".to_string(),
                        "4. Verify setup: 'git --version' and 'git config --list'".to_string(),
                    ]
                } else {
                    vec![
                        format!(
                            "1. Check if '{}' is installed",
                            command.split_whitespace().next().unwrap_or("command")
                        ),
                        "2. Verify the command is in your PATH".to_string(),
                        "3. Check required permissions".to_string(),
                        "4. Try running the command manually to debug".to_string(),
                    ]
                }
            }
            AtmError::FileError {
                operation,
                file_path,
                ..
            } => {
                vec![
                    format!("1. Check if file/directory exists: {}", file_path),
                    format!(
                        "2. Verify permissions for {}: 'ls -la {}'",
                        operation, file_path
                    ),
                    "3. Ensure parent directories exist".to_string(),
                    "4. Check available disk space: 'df -h'".to_string(),
                    "5. Try running with elevated privileges if needed".to_string(),
                ]
            }
            AtmError::JsonError { file_path, .. } => {
                vec![
                    format!("1. Open file in editor: {}", file_path),
                    "2. Check for missing commas, brackets, or quotes".to_string(),
                    "3. Use JSON validator: 'jsonlint <file>' or online tool".to_string(),
                    "4. Compare with working JSON example".to_string(),
                    "5. Remove comments if in strict JSON mode".to_string(),
                ]
            }
            AtmError::TuiError { .. } => {
                vec![
                    "1. Check terminal compatibility: echo $TERM".to_string(),
                    "2. Try different terminal emulator".to_string(),
                    "3. Update terminal configuration".to_string(),
                    "4. Ensure sufficient terminal size".to_string(),
                    "5. Check for terminal multiplexer conflicts".to_string(),
                ]
            }
        }
    }

    /// Get relevant documentation links
    pub fn get_help_resources(&self) -> Vec<(String, String)> {
        match self {
            AtmError::ConfigError { .. } => {
                vec![
                    (
                        "Configuration Guide".to_string(),
                        "https://github.com/your-repo/wiki/Configuration".to_string(),
                    ),
                    (
                        "JSON Validator".to_string(),
                        "https://jsonlint.com/".to_string(),
                    ),
                ]
            }
            AtmError::ModuleError { .. } => {
                vec![
                    (
                        "Module Development".to_string(),
                        "https://github.com/your-repo/wiki/Modules".to_string(),
                    ),
                    (
                        "Example Modules".to_string(),
                        "https://github.com/your-repo/tree/main/modules".to_string(),
                    ),
                ]
            }
            AtmError::CommandError { command, .. } => {
                if command.contains("code") {
                    vec![
                        (
                            "VS Code Download".to_string(),
                            "https://code.visualstudio.com/Download".to_string(),
                        ),
                        (
                            "VS Code Linux Setup".to_string(),
                            "https://code.visualstudio.com/docs/setup/linux".to_string(),
                        ),
                    ]
                } else if command.contains("git") {
                    vec![
                        (
                            "Git Documentation".to_string(),
                            "https://git-scm.com/doc".to_string(),
                        ),
                        (
                            "Git Configuration".to_string(),
                            "https://git-scm.com/book/en/v2/Getting-Started-First-Time-Git-Setup"
                                .to_string(),
                        ),
                    ]
                } else {
                    vec![
                        (
                            "Arch Linux Packages".to_string(),
                            "https://archlinux.org/packages/".to_string(),
                        ),
                        (
                            "AUR Repository".to_string(),
                            "https://aur.archlinux.org/".to_string(),
                        ),
                    ]
                }
            }
            _ => {
                vec![
                    (
                        "Project Documentation".to_string(),
                        "https://github.com/your-repo/wiki".to_string(),
                    ),
                    (
                        "Issue Tracker".to_string(),
                        "https://github.com/your-repo/issues".to_string(),
                    ),
                ]
            }
        }
    }
}

pub fn init() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        // Ensure terminal is properly restored
        restore_terminal();

        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let metadata = metadata!();
            let file_path = handle_dump(&metadata, panic_info);
            // prints human-panic message
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));
    Ok(())
}

/// Restore terminal to a clean state
/// This function can be called from panic handlers or signal handlers
pub fn restore_terminal() {
    use crossterm::{
        cursor,
        event::{DisableBracketedPaste, DisableMouseCapture},
        execute,
        terminal::{disable_raw_mode, is_raw_mode_enabled, LeaveAlternateScreen},
    };
    use std::io::stdout;

    // Try to restore terminal state
    if let Ok(true) = is_raw_mode_enabled() {
        let _ = execute!(
            stdout(),
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        );
        let _ = disable_raw_mode();
    }
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
        (target: $target:expr, level: $level:expr, $ex:expr) => {
            {
                match $ex {
                        value => {
                                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                                value
                        }
                }
            }
        };
        (level: $level:expr, $ex:expr) => {
                trace_dbg!(target: module_path!(), level: $level, $ex)
        };
        (target: $target:expr, $ex:expr) => {
                trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
        };
        ($ex:expr) => {
                trace_dbg!(level: tracing::Level::DEBUG, $ex)
        };
}
