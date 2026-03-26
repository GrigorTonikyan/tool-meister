//! Security Validation Module
//!
//! This module implements security checks and input sanitization for command execution.
//! It follows the security guidelines defined in `.github/instructions/security.instructions.md`
//! to prevent injection attacks, validate inputs, and ensure safe command execution.

use crate::error::{CoreError, CoreResult};
use regex::Regex;
use shared_types::module::ModuleFunction;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, instrument, warn};

use super::ExecutionContext;

/// Security validation configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Allow commands that require sudo privileges
    pub allow_sudo: bool,
    /// Allow execution of scripts from temporary directories
    pub allow_temp_execution: bool,
    /// Maximum command length allowed
    pub max_command_length: usize,
    /// Maximum number of arguments
    pub max_args: usize,
    /// Allowed working directories (empty means any)
    pub allowed_working_dirs: Vec<String>,
    /// Blocked commands that should never be executed
    pub blocked_commands: HashSet<String>,
    /// Blocked patterns in commands
    pub blocked_patterns: Vec<Regex>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut blocked_commands = HashSet::new();
        blocked_commands.insert("rm".to_string());
        blocked_commands.insert("rmdir".to_string());
        blocked_commands.insert("dd".to_string());
        blocked_commands.insert("mkfs".to_string());
        blocked_commands.insert("fdisk".to_string());
        blocked_commands.insert("parted".to_string());
        blocked_commands.insert("shutdown".to_string());
        blocked_commands.insert("reboot".to_string());
        blocked_commands.insert("halt".to_string());
        blocked_commands.insert("poweroff".to_string());

        let blocked_patterns = vec![
            Regex::new(r"rm\s+-r?f").unwrap(),  // rm -rf patterns
            Regex::new(r">\s*/dev/").unwrap(),  // Writing to device files
            Regex::new(r";\s*rm\s+").unwrap(),  // Command injection with rm
            Regex::new(r"\|\s*rm\s+").unwrap(), // Piped rm commands
            Regex::new(r"&&\s*rm\s+").unwrap(), // Chained rm commands
            Regex::new(r"\$\(.*\)").unwrap(),   // Command substitution
            Regex::new(r"`.*`").unwrap(),       // Backtick command substitution
        ];

        Self {
            allow_sudo: false,
            allow_temp_execution: false,
            max_command_length: 1000,
            max_args: 50,
            allowed_working_dirs: vec![],
            blocked_commands,
            blocked_patterns,
        }
    }
}

/// Security validator for command execution
#[derive(Debug)]
pub struct SecurityValidator {
    config: SecurityConfig,
    /// Cache of previously validated commands
    validation_cache: HashMap<String, bool>,
}

impl SecurityValidator {
    /// Create a new security validator with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            validation_cache: HashMap::new(),
        }
    }

    /// Create a new security validator with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            validation_cache: HashMap::new(),
        }
    }

    /// Validate a module function before execution
    #[instrument(skip(self, function, context))]
    pub async fn validate_function(
        &self,
        function: &ModuleFunction,
        context: &ExecutionContext,
    ) -> CoreResult<()> {
        debug!(
            "Validating function '{}' for security",
            context.function_name
        );

        // Basic input validation
        self.validate_basic_inputs(&function.code, &context.args)?;

        // Check for blocked commands and patterns
        self.check_blocked_content(&function.code)?;

        // Validate command structure
        self.validate_command_structure(&function.code)?;

        // Check working directory if specified
        if let Some(ref wd) = context.working_dir {
            self.validate_working_directory(wd)?;
        }

        // Validate environment variables
        self.validate_environment_variables(&context.env_vars)?;

        // Special validation for sudo commands
        if function.code.contains("sudo") && !self.config.allow_sudo {
            return Err(CoreError::from(shared_types::SharedError::security(
                "Sudo commands are not allowed by security policy".to_string(),
            )));
        }

        debug!(
            "Function '{}' passed security validation",
            context.function_name
        );
        Ok(())
    }

    /// Validate a direct command before execution
    #[instrument(skip(self, command, args, context))]
    pub async fn validate_command(
        &self,
        command: &str,
        args: &[String],
        context: &ExecutionContext,
    ) -> CoreResult<()> {
        debug!("Validating direct command '{}' for security", command);

        // Create full command string for validation
        let full_command = format!("{} {}", command, args.join(" "));

        // Basic input validation
        self.validate_basic_inputs(&full_command, args)?;

        // Check for blocked commands and patterns
        self.check_blocked_content(&full_command)?;

        // Validate command structure
        self.validate_command_structure(&full_command)?;

        // Check working directory if specified
        if let Some(ref wd) = context.working_dir {
            self.validate_working_directory(wd)?;
        }

        // Validate environment variables
        self.validate_environment_variables(&context.env_vars)?;

        // Special validation for sudo commands
        if command == "sudo" || full_command.contains("sudo") {
            if !self.config.allow_sudo {
                return Err(CoreError::from(shared_types::SharedError::security(
                    "Sudo commands are not allowed by security policy".to_string(),
                )));
            }
        }

        debug!("Direct command '{}' passed security validation", command);
        Ok(())
    }

    /// Sanitize user input to prevent injection attacks
    pub fn sanitize_input(&self, input: &str) -> String {
        // Remove null bytes
        let sanitized = input.replace('\0', "");

        // Remove dangerous characters that could be used for injection
        let dangerous_chars = ['|', '&', ';', '(', ')', '<', '>', '`'];
        let mut result = sanitized;

        for ch in dangerous_chars {
            result = result.replace(ch, "");
        }

        // Limit length
        if result.len() > self.config.max_command_length {
            result.truncate(self.config.max_command_length);
        }

        result
    }

    /// Validate file paths to prevent directory traversal attacks
    pub fn validate_file_path(&self, path: &str) -> CoreResult<()> {
        // Check for directory traversal patterns
        if path.contains("..") {
            return Err(CoreError::from(shared_types::SharedError::security(
                "Directory traversal patterns are not allowed".to_string(),
            )));
        }

        // Check for absolute paths to sensitive directories
        let sensitive_paths = [
            "/etc/passwd",
            "/etc/shadow",
            "/root/",
            "/sys/",
            "/proc/",
            "/dev/",
        ];

        for sensitive in &sensitive_paths {
            if path.starts_with(sensitive) {
                return Err(CoreError::from(shared_types::SharedError::security(
                    format!("Access to sensitive path '{}' is not allowed", sensitive),
                )));
            }
        }

        // Ensure path is within expected boundaries
        let canonical_path = Path::new(path).canonicalize().map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to canonicalize path '{}': {}",
                path, e
            )))
        })?;

        // Check if path is in allowed working directories (if configured)
        if !self.config.allowed_working_dirs.is_empty() {
            let path_str = canonical_path.to_string_lossy();
            let is_allowed = self
                .config
                .allowed_working_dirs
                .iter()
                .any(|allowed| path_str.starts_with(allowed));

            if !is_allowed {
                return Err(CoreError::from(shared_types::SharedError::security(
                    format!("Path '{}' is not in allowed working directories", path),
                )));
            }
        }

        Ok(())
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
        // Clear cache when config changes
        self.validation_cache.clear();
    }

    /// Get current security configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    // Private helper methods

    /// Validate basic input parameters
    fn validate_basic_inputs(&self, command: &str, args: &[String]) -> CoreResult<()> {
        // Check command length
        if command.len() > self.config.max_command_length {
            return Err(CoreError::from(shared_types::SharedError::validation(
                format!(
                    "Command length {} exceeds maximum allowed {}",
                    command.len(),
                    self.config.max_command_length
                ),
            )));
        }

        // Check number of arguments
        if args.len() > self.config.max_args {
            return Err(CoreError::from(shared_types::SharedError::validation(
                format!(
                    "Number of arguments {} exceeds maximum allowed {}",
                    args.len(),
                    self.config.max_args
                ),
            )));
        }

        // Check for null bytes in command
        if command.contains('\0') {
            return Err(CoreError::from(shared_types::SharedError::security(
                "Null bytes are not allowed in commands".to_string(),
            )));
        }

        // Check for null bytes in arguments
        for (i, arg) in args.iter().enumerate() {
            if arg.contains('\0') {
                return Err(CoreError::from(shared_types::SharedError::security(
                    format!("Null bytes are not allowed in argument {}", i),
                )));
            }
        }

        Ok(())
    }

    /// Check for blocked commands and patterns
    fn check_blocked_content(&self, command: &str) -> CoreResult<()> {
        // Check blocked commands
        let command_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(base_command) = command_parts.first() {
            let base_cmd = Path::new(base_command)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(base_command);

            if self.config.blocked_commands.contains(base_cmd) {
                return Err(CoreError::from(shared_types::SharedError::security(
                    format!("Command '{}' is blocked by security policy", base_cmd),
                )));
            }
        }

        // Check blocked patterns
        for pattern in &self.config.blocked_patterns {
            if pattern.is_match(command) {
                return Err(CoreError::from(shared_types::SharedError::security(
                    format!("Command contains blocked pattern: {}", pattern.as_str()),
                )));
            }
        }

        Ok(())
    }

    /// Validate command structure for security issues
    fn validate_command_structure(&self, command: &str) -> CoreResult<()> {
        // Check for command injection patterns
        let injection_patterns = ["&&", "||", ";", "|", "<", ">", "$(", "`"];

        for pattern in &injection_patterns {
            if command.contains(pattern) {
                warn!("Potentially unsafe pattern '{}' found in command", pattern);
                // For now, we'll log a warning but still allow execution
                // In stricter mode, this could be an error
            }
        }

        // Check for script execution in temporary directories
        if !self.config.allow_temp_execution {
            let temp_patterns = ["/tmp/", "/var/tmp/", "/dev/shm/"];
            for pattern in &temp_patterns {
                if command.contains(pattern) {
                    return Err(CoreError::from(shared_types::SharedError::security(
                        format!(
                            "Execution from temporary directory '{}' is not allowed",
                            pattern
                        ),
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate working directory
    fn validate_working_directory(&self, working_dir: &str) -> CoreResult<()> {
        self.validate_file_path(working_dir)?;

        // Additional checks for working directory
        let path = Path::new(working_dir);
        if !path.exists() {
            return Err(CoreError::from(shared_types::SharedError::validation(
                format!("Working directory '{}' does not exist", working_dir),
            )));
        }

        if !path.is_dir() {
            return Err(CoreError::from(shared_types::SharedError::validation(
                format!("Working directory '{}' is not a directory", working_dir),
            )));
        }

        Ok(())
    }

    /// Validate environment variables for security issues
    fn validate_environment_variables(&self, env_vars: &HashMap<String, String>) -> CoreResult<()> {
        // Check for dangerous environment variables
        let dangerous_vars = ["LD_PRELOAD", "LD_LIBRARY_PATH", "PATH", "SHELL", "IFS"];

        for (key, value) in env_vars {
            // Check for dangerous variable names
            if dangerous_vars.contains(&key.as_str()) {
                warn!(
                    "Setting potentially dangerous environment variable: {}",
                    key
                );
            }

            // Check for null bytes
            if key.contains('\0') || value.contains('\0') {
                return Err(CoreError::from(shared_types::SharedError::security(
                    "Null bytes are not allowed in environment variables".to_string(),
                )));
            }

            // Check for command injection in values
            if value.contains("$(") || value.contains("`") {
                return Err(CoreError::from(shared_types::SharedError::security(
                    "Command substitution patterns are not allowed in environment variables"
                        .to_string(),
                )));
            }
        }

        Ok(())
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_command_validation() {
        let validator = SecurityValidator::new();
        let context = ExecutionContext {
            module_name: "test".to_string(),
            function_name: "test".to_string(),
            args: vec!["arg1".to_string()],
            dry_run: true,
            timeout: std::time::Duration::from_secs(10),
            working_dir: None,
            env_vars: HashMap::new(),
        };

        // Valid command should pass
        let result = validator
            .validate_command("echo", &["hello".to_string()], &context)
            .await;
        assert!(result.is_ok());

        // Blocked command should fail
        let result = validator
            .validate_command("rm", &["-rf".to_string(), "/".to_string()], &context)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_input_sanitization() {
        let validator = SecurityValidator::new();

        // Test removal of dangerous characters
        let sanitized = validator.sanitize_input("echo hello; rm -rf /");
        assert!(!sanitized.contains(';'));

        // Test null byte removal
        let sanitized = validator.sanitize_input("echo\0hello");
        assert!(!sanitized.contains('\0'));
    }

    #[test]
    fn test_file_path_validation() {
        let validator = SecurityValidator::new();

        // Directory traversal should fail
        let result = validator.validate_file_path("../../../etc/passwd");
        assert!(result.is_err());

        // Sensitive path should fail
        let result = validator.validate_file_path("/etc/shadow");
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_variable_validation() {
        let validator = SecurityValidator::new();

        let mut env_vars = HashMap::new();
        env_vars.insert("SAFE_VAR".to_string(), "safe_value".to_string());

        // Safe environment variables should pass
        let result = validator.validate_environment_variables(&env_vars);
        assert!(result.is_ok());

        // Null bytes should fail
        env_vars.insert("BAD_VAR".to_string(), "bad\0value".to_string());
        let result = validator.validate_environment_variables(&env_vars);
        assert!(result.is_err());
    }
}
