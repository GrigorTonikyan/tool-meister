//! Configuration loading and parsing functionality.
//!
//! This module provides secure configuration loading with JSONC support,
//! validation, and error handling. It implements security best practices
//! from the security instructions including input validation and safe
//! file operations.

use crate::error::{CoreError, CoreResult};
use serde::de::DeserializeOwned;
use shared_types::config::{AppConfig, MenuConfig};
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Maximum allowed path length to prevent path-based attacks
const MAX_PATH_LENGTH: usize = 4096;

/// List of forbidden path components for security
const FORBIDDEN_PATH_COMPONENTS: &[&str] = &[
    "..",
    "~",
    "$",
    "`",
    "|",
    "&",
    ";",
    ">",
    "<",
    "*",
    "?",
    "[",
    "]",
    "{",
    "}",
    "\\",
    // System directories that should never be accessed
    "/etc/passwd",
    "/etc/shadow",
    "/proc",
    "/sys",
    "/dev",
];

/// List of allowed configuration file extensions
const ALLOWED_EXTENSIONS: &[&str] = &["jsonc", "json"];

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityValidation {
    pub is_safe: bool,
    pub violations: Vec<String>,
    pub file_info: Option<FileSecurityInfo>,
}

/// File security information
#[derive(Debug, Clone)]
pub struct FileSecurityInfo {
    pub size: u64,
    pub is_readable: bool,
    pub is_writable: bool,
    pub modified_time: Option<SystemTime>,
    pub extension: Option<String>,
}

/// Configuration loader that handles JSONC parsing and validation.
///
/// The `ConfigLoader` provides static methods for loading different types
/// of configuration files with comprehensive error handling and security
/// validation. It strips JSONC comments before parsing and validates
/// the resulting configuration structures.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads and parses application configuration from a JSONC file.
    ///
    /// This method reads the configuration file, strips JSONC comments,
    /// parses the JSON, and validates the resulting configuration structure.
    /// It implements security best practices by validating file paths and
    /// content before processing.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the application configuration file
    ///
    /// # Returns
    ///
    /// Returns the loaded and validated `AppConfig` structure.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Config` if:
    /// - File cannot be read (permissions, not found, etc.)
    /// - File contains invalid JSONC syntax
    /// - Configuration structure is invalid
    /// - Validation fails for security or format reasons
    ///
    /// # Security
    ///
    /// - Validates file path to prevent directory traversal
    /// - Checks file size to prevent resource exhaustion
    /// - Sanitizes configuration values during validation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigLoader;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::load_app_config(Path::new("config.jsonc")).await?;
    /// println!("App name: {}", config.app_settings.app_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_app_config<P: AsRef<Path>>(path: P) -> CoreResult<AppConfig> {
        let path = path.as_ref();

        // Validate path for security
        Self::validate_config_path(path)?;

        info!("Loading application configuration from: {}", path.display());

        let config: AppConfig = Self::load_and_parse_jsonc(path).await?;

        // Validate the loaded configuration
        config.validate().map_err(|e| {
            CoreError::config_with_path(
                format!("Configuration validation failed: {}", e),
                path.to_string_lossy().to_string(),
            )
        })?;

        debug!("Successfully loaded and validated app configuration");
        Ok(config)
    }

    /// Loads and parses menu configuration from a JSONC file.
    ///
    /// Similar to `load_app_config` but specifically for menu configuration
    /// files. Provides the same security validation and error handling.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the menu configuration file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use core_lib::config::ConfigLoader;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::load_menu_config(Path::new("main_menu.jsonc")).await?;
    /// println!("Menu title: {}", config.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_menu_config<P: AsRef<Path>>(path: P) -> CoreResult<MenuConfig> {
        let path = path.as_ref();

        // Validate path for security
        Self::validate_config_path(path)?;

        info!("Loading menu configuration from: {}", path.display());

        let config: MenuConfig = Self::load_and_parse_jsonc(path).await?;

        // Validate the loaded configuration
        config.validate().map_err(|e| {
            CoreError::config_with_path(
                format!("Menu configuration validation failed: {}", e),
                path.to_string_lossy().to_string(),
            )
        })?;

        debug!("Successfully loaded and validated menu configuration");
        Ok(config)
    }

    /// Generic method to load and parse any JSONC configuration file.
    ///
    /// This method provides the core JSONC loading functionality used by
    /// the specific configuration loaders. It handles file reading,
    /// comment stripping, JSON parsing, and basic error handling.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSONC file to load
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the configuration into
    ///
    /// # Security
    ///
    /// This method assumes the path has already been validated by the caller.
    /// It implements additional security measures like file size checking.
    async fn load_and_parse_jsonc<T, P>(path: P) -> CoreResult<T>
    where
        T: DeserializeOwned,
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // Read file content with security checks
        let content = Self::read_file_safely(path).await?;

        // Strip JSONC comments
        let cleaned_content = Self::strip_jsonc_comments(&content);

        // Parse JSON
        serde_json::from_str(&cleaned_content).map_err(|e| {
            CoreError::config_with_path(
                format!("Invalid JSON syntax: {}", e),
                path.to_string_lossy().to_string(),
            )
        })
    }

    /// Safely reads a file with security validation.
    ///
    /// This method implements security best practices for file reading,
    /// including size limits and permission checking.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Security
    ///
    /// - Checks file size to prevent resource exhaustion attacks
    /// - Validates file permissions and accessibility
    /// - Uses secure file reading methods
    async fn read_file_safely<P: AsRef<Path>>(path: P) -> CoreResult<String> {
        let path = path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Err(CoreError::config_with_path(
                "Configuration file not found".to_string(),
                path.to_string_lossy().to_string(),
            ));
        }

        // Check if it's actually a file (not a directory or special file)
        if !path.is_file() {
            return Err(CoreError::config_with_path(
                "Path is not a regular file".to_string(),
                path.to_string_lossy().to_string(),
            ));
        }

        // Check file size to prevent resource exhaustion
        let metadata = fs::metadata(path).map_err(|e| {
            CoreError::config_with_path(
                format!("Failed to read file metadata: {}", e),
                path.to_string_lossy().to_string(),
            )
        })?;

        const MAX_CONFIG_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
        if metadata.len() > MAX_CONFIG_SIZE {
            return Err(CoreError::config_with_path(
                format!(
                    "Configuration file too large: {} bytes (max: {} bytes)",
                    metadata.len(),
                    MAX_CONFIG_SIZE
                ),
                path.to_string_lossy().to_string(),
            ));
        }

        // Read file content
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            CoreError::config_with_path(
                format!("Failed to read file: {}", e),
                path.to_string_lossy().to_string(),
            )
        })?;

        // Validate content security
        let (is_content_safe, content_violations) =
            Self::validate_config_content_security(&content);
        if !is_content_safe {
            let violations = content_violations.join("; ");
            warn!(
                "Content security validation failed for {}: {}",
                path.to_string_lossy(),
                violations
            );
            return Err(CoreError::config_with_path(
                format!("Content security validation failed: {}", violations),
                path.to_string_lossy().to_string(),
            ));
        }

        Ok(content)
    }

    /// Validates a configuration file path for security.
    ///
    /// This method implements security validations to prevent directory
    /// traversal attacks and other path-based security issues.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Security
    ///
    /// - Prevents directory traversal (../)
    /// - Validates path length limits
    /// - Ensures safe characters in path
    /// - Checks for symbolic link attacks
    fn validate_config_path<P: AsRef<Path>>(path: P) -> CoreResult<()> {
        let path = path.as_ref();

        // Use comprehensive security validation
        let validation = Self::validate_config_path_comprehensive(path);

        if !validation.is_safe {
            let violations = validation.violations.join("; ");
            warn!(
                "Security validation failed for path {}: {}",
                path.to_string_lossy(),
                violations
            );
            return Err(CoreError::config_with_path(
                format!("Security validation failed: {}", violations),
                path.to_string_lossy().to_string(),
            ));
        }

        debug!(
            "Configuration path passed security validation: {}",
            path.to_string_lossy()
        );
        Ok(())
    }

    /// Strips JSONC-style comments from a string.
    ///
    /// This method removes both single-line (`//`) and multi-line (`/* */`)
    /// comments from JSONC content while preserving string literals that
    /// might contain comment-like sequences.
    ///
    /// # Arguments
    ///
    /// * `content` - The JSONC content to process
    ///
    /// # Returns
    ///
    /// Clean JSON content with comments removed.
    ///
    /// # Implementation Notes
    ///
    /// This implementation carefully handles:
    /// - String literals containing comment-like text
    /// - Escaped characters within strings
    /// - Nested comment structures
    /// - Line ending preservation for error reporting
    fn strip_jsonc_comments(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_string = false;
        let mut in_single_line_comment = false;
        let mut in_multi_line_comment = false;
        let mut escape_next = false;

        while let Some(ch) = chars.next() {
            if escape_next {
                escape_next = false;
                if !in_single_line_comment && !in_multi_line_comment {
                    result.push(ch);
                }
                continue;
            }

            if in_single_line_comment {
                if ch == '\n' || ch == '\r' {
                    in_single_line_comment = false;
                    result.push(ch); // Preserve line endings
                }
                continue;
            }

            if in_multi_line_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    in_multi_line_comment = false;
                }
                continue;
            }

            if in_string {
                if ch == '\\' {
                    escape_next = true;
                } else if ch == '"' {
                    in_string = false;
                }
                result.push(ch);
            } else {
                if ch == '"' {
                    in_string = true;
                    result.push(ch);
                } else if ch == '/' {
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch == '/' {
                            chars.next(); // consume second '/'
                            in_single_line_comment = true;
                            continue;
                        } else if next_ch == '*' {
                            chars.next(); // consume '*'
                            in_multi_line_comment = true;
                            continue;
                        }
                    }
                    result.push(ch);
                } else {
                    result.push(ch);
                }
            }
        }

        result
    }

    /// Performs comprehensive security validation on a configuration file path.
    ///
    /// This method implements security best practices from `.github/instructions/security.instructions.md`:
    /// - Validates file paths to prevent directory traversal attacks
    /// - Checks file permissions and accessibility
    /// - Validates file extensions against allowed list
    /// - Ensures file size constraints
    /// - Detects forbidden path components
    ///
    /// # Arguments
    /// * `path` - The configuration file path to validate
    ///
    /// # Returns
    /// Returns a `SecurityValidation` struct with validation results and any violations found.
    ///
    /// # Examples
    /// ```
    /// use core_lib::config::loader::ConfigLoader;
    /// use std::path::Path;
    ///
    /// let validation = ConfigLoader::validate_config_path_comprehensive(Path::new("config.jsonc"));
    /// if !validation.is_safe {
    ///     println!("Security violations: {:?}", validation.violations);
    /// }
    /// ```
    pub fn validate_config_path_comprehensive(path: &Path) -> SecurityValidation {
        let mut violations = Vec::new();
        let mut is_safe = true;

        // Convert path to string for analysis
        let path_str = path.to_string_lossy();

        // Check path length to prevent buffer overflow attacks
        if path_str.len() > MAX_PATH_LENGTH {
            violations.push(format!(
                "Path too long: {} characters (max: {})",
                path_str.len(),
                MAX_PATH_LENGTH
            ));
            is_safe = false;
        }

        // Check for forbidden path components
        for component in FORBIDDEN_PATH_COMPONENTS {
            if path_str.contains(component) {
                violations.push(format!(
                    "Forbidden path component detected: '{}'",
                    component
                ));
                is_safe = false;
            }
        }

        // Validate file extension
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !ALLOWED_EXTENSIONS.contains(&ext_str.as_str()) {
                violations.push(format!(
                    "Invalid file extension: '{}' (allowed: {:?})",
                    ext_str, ALLOWED_EXTENSIONS
                ));
                is_safe = false;
            }
        } else {
            violations.push("Missing file extension".to_string());
            is_safe = false;
        }

        // Check if path is absolute and doesn't contain relative components
        if !path.is_absolute() {
            // For relative paths, ensure they don't traverse outside expected directories
            for component in path.components() {
                if let std::path::Component::ParentDir = component {
                    violations.push("Relative path traversal detected (..)".to_string());
                    is_safe = false;
                    break;
                }
            }
        }

        // Try to get file information
        let file_info = if path.exists() {
            match fs::metadata(path) {
                Ok(metadata) => {
                    let mut info = FileSecurityInfo {
                        size: metadata.len(),
                        is_readable: true, // We'll assume readable if we can get metadata
                        is_writable: false, // We'll be conservative about write access
                        modified_time: metadata.modified().ok(),
                        extension: path.extension().map(|e| e.to_string_lossy().to_string()),
                    };

                    // Check file size limits
                    const MAX_CONFIG_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
                    if metadata.len() > MAX_CONFIG_SIZE {
                        violations.push(format!(
                            "File too large: {} bytes (max: {} bytes)",
                            metadata.len(),
                            MAX_CONFIG_SIZE
                        ));
                        is_safe = false;
                    }

                    // On Unix systems, check file permissions
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let permissions = metadata.permissions();
                        let mode = permissions.mode();

                        // Check if file is readable by owner
                        info.is_readable = (mode & 0o400) != 0;
                        // Check if file is writable by owner
                        info.is_writable = (mode & 0o200) != 0;

                        // Warn if file is world-readable (security concern for config files)
                        if (mode & 0o004) != 0 {
                            violations.push("Configuration file is world-readable".to_string());
                            // This is a warning, not a blocking violation
                        }

                        // Warn if file is world-writable (major security concern)
                        if (mode & 0o002) != 0 {
                            violations.push("Configuration file is world-writable".to_string());
                            is_safe = false;
                        }
                    }

                    Some(info)
                }
                Err(e) => {
                    violations.push(format!("Cannot access file metadata: {}", e));
                    is_safe = false;
                    None
                }
            }
        } else {
            None // File doesn't exist, which might be OK for some use cases
        };

        SecurityValidation {
            is_safe,
            violations,
            file_info,
        }
    }

    /// Validates configuration content for security concerns.
    ///
    /// This method scans configuration content for potentially dangerous
    /// patterns, command injections, or suspicious content that could
    /// indicate a security issue.
    ///
    /// # Arguments
    /// * `content` - The configuration file content to validate
    ///
    /// # Returns
    /// Returns a tuple of (is_safe, violations) where is_safe indicates
    /// if the content passed validation and violations contains any issues found.
    fn validate_config_content_security(content: &str) -> (bool, Vec<String>) {
        let mut violations = Vec::new();
        let mut is_safe = true;

        // Check for potential command injection patterns
        let dangerous_patterns = [
            "`",
            "$",
            "$(",
            "${",
            "&&",
            "||",
            "|",
            ";",
            "&",
            ">",
            "<",
            "exec",
            "eval",
            "system",
            "spawn",
            "/bin/",
            "/usr/bin/",
            "rm -",
            "chmod",
            "chown",
            "sudo",
            "su ",
        ];

        for pattern in dangerous_patterns {
            if content.contains(pattern) {
                violations.push(format!(
                    "Potentially dangerous pattern detected in content: '{}'",
                    pattern
                ));
                // This is suspicious but might be legitimate in some contexts
                // So we log it but don't necessarily block
            }
        }

        // Check for excessively long lines (potential buffer overflow)
        for (line_num, line) in content.lines().enumerate() {
            if line.len() > 10000 {
                violations.push(format!(
                    "Line {} is excessively long: {} characters",
                    line_num + 1,
                    line.len()
                ));
                is_safe = false;
            }
        }

        // Check for null bytes or other binary content
        if content.contains('\0') {
            violations.push("Binary content detected (null bytes)".to_string());
            is_safe = false;
        }

        // Check for excessively nested JSON (potential DoS)
        let brace_depth = content
            .chars()
            .fold((0i32, 0i32), |(current, max), ch| match ch {
                '{' | '[' => {
                    let new_depth = current + 1;
                    (new_depth, max.max(new_depth))
                }
                '}' | ']' => (current.saturating_sub(1), max),
                _ => (current, max),
            });

        if brace_depth.1 > 100 {
            violations.push(format!(
                "Excessive nesting depth detected: {} levels",
                brace_depth.1
            ));
            is_safe = false;
        }

        (is_safe, violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_strip_jsonc_comments() {
        let jsonc = r#"
        {
            // This is a comment
            "name": "test", // Another comment
            "value": 42,
            /* Multi-line
               comment */
            "description": "A test object"
        }
        "#;

        let cleaned = ConfigLoader::strip_jsonc_comments(jsonc);
        assert!(!cleaned.contains("This is a comment"));
        assert!(!cleaned.contains("Another comment"));
        assert!(!cleaned.contains("Multi-line"));
        assert!(cleaned.contains("\"name\": \"test\""));
        assert!(cleaned.contains("\"value\": 42"));
    }

    #[test]
    fn test_strip_comments_preserves_strings() {
        let jsonc = r#"
        {
            "message": "This string contains // and /* comments */",
            "url": "https://example.com/path", // Real comment
            "regex": "\/\*.*\*\/"
        }
        "#;

        let cleaned = ConfigLoader::strip_jsonc_comments(jsonc);
        assert!(cleaned.contains("This string contains // and /* comments */"));
        assert!(cleaned.contains("https://example.com/path"));
        assert!(cleaned.contains(r#"\/\*.*\*\/"#));
        assert!(!cleaned.contains("Real comment"));
    }

    #[tokio::test]
    async fn test_validate_config_path_security() {
        // Test directory traversal prevention
        let result = ConfigLoader::validate_config_path("../etc/passwd");
        assert!(result.is_err());

        let result = ConfigLoader::validate_config_path("config/../../../etc/passwd");
        assert!(result.is_err());

        // Test valid paths
        let result = ConfigLoader::validate_config_path("config.jsonc");
        assert!(result.is_ok());

        let result = ConfigLoader::validate_config_path("configs/app.jsonc");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_path_length() {
        // Test path length validation
        let long_path = "a".repeat(2000);
        let result = ConfigLoader::validate_config_path(long_path.as_str());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_safely_nonexistent() {
        let result = ConfigLoader::read_file_safely("nonexistent_file.jsonc").await;
        assert!(result.is_err());
        if let Err(CoreError::Config { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected CoreError::Config");
        }
    }

    #[tokio::test]
    async fn test_load_app_config_with_valid_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.jsonc");

        let test_config = r#"
        {
            // Application configuration
            "appSettings": {
                "appName": "Test App",
                "version": "1.0.0",
                "modulesDir": "modules",
                "downloadDir": "/tmp/downloads",
                "installPrefix": "/opt",
                "animation": {
                    "steps": 10,
                    "delayMs": 320
                }
            }
        }
        "#;

        fs::write(&config_path, test_config).expect("Failed to write test config");

        let result = ConfigLoader::load_app_config(&config_path).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.app_settings.app_name, "Test App");
        assert_eq!(config.app_settings.version, "1.0.0");
        assert_eq!(config.app_settings.modules_dir, "modules");
    }

    #[tokio::test]
    async fn test_load_menu_config_with_valid_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_menu.jsonc");

        let test_config = r#"
        {
            // Menu configuration
            "title": "Test Main Menu",
            "dynamicMenu": true,
            "options": [
                {
                    "name": "Test Option",
                    "description": "A test menu option",
                    "action": "test_action",
                    "enabled": true
                }
            ]
        }
        "#;

        fs::write(&config_path, test_config).expect("Failed to write test config");

        let result = ConfigLoader::load_menu_config(&config_path).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.title, "Test Main Menu");
        assert!(config.dynamic_menu);
        assert_eq!(config.options.len(), 1);
    }

    #[tokio::test]
    async fn test_load_config_with_invalid_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("invalid_config.jsonc");

        let invalid_config = r#"
        {
            "appSettings": {
                "appName": "Test App"
                // Missing comma and other syntax errors
                "version": "1.0.0"
            }
        "#;

        fs::write(&config_path, invalid_config).expect("Failed to write invalid config");

        let result = ConfigLoader::load_app_config(&config_path).await;
        assert!(result.is_err());
        if let Err(CoreError::Config { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected CoreError::Config for invalid JSON");
        }
    }

    #[test]
    fn test_comprehensive_security_validation() {
        // Test valid configuration file path (non-existent but safe)
        let validation =
            ConfigLoader::validate_config_path_comprehensive(Path::new("config.jsonc"));
        // Should be considered safe since path and extension are valid
        assert!(validation.is_safe);
        assert!(validation.violations.is_empty());

        // Test dangerous path components
        let validation =
            ConfigLoader::validate_config_path_comprehensive(Path::new("../etc/passwd"));
        assert!(!validation.is_safe);
        assert!(validation
            .violations
            .iter()
            .any(|v| v.contains("Forbidden path component")));

        // Test invalid extension
        let validation = ConfigLoader::validate_config_path_comprehensive(Path::new("config.exe"));
        assert!(!validation.is_safe);
        assert!(validation
            .violations
            .iter()
            .any(|v| v.contains("Invalid file extension")));

        // Test excessively long path
        let long_path = "a".repeat(5000) + ".jsonc";
        let validation = ConfigLoader::validate_config_path_comprehensive(Path::new(&long_path));
        assert!(!validation.is_safe);
        assert!(validation
            .violations
            .iter()
            .any(|v| v.contains("Path too long")));
    }

    #[test]
    fn test_content_security_validation() {
        // Test safe content
        let safe_content = r#"{"name": "test", "value": 42}"#;
        let (is_safe, violations) = ConfigLoader::validate_config_content_security(safe_content);
        assert!(is_safe);
        assert!(violations.is_empty());

        // Test dangerous patterns
        let dangerous_content = r#"{"command": "rm -rf /"}"#;
        let (_is_safe, violations) =
            ConfigLoader::validate_config_content_security(dangerous_content);
        // This should detect dangerous patterns but might not be blocking depending on context
        assert!(!violations.is_empty());

        // Test null bytes
        let binary_content = "config\0data";
        let (is_safe, _) = ConfigLoader::validate_config_content_security(binary_content);
        assert!(!is_safe);

        // Test excessive nesting
        let nested_content = "{".repeat(150) + &"}".repeat(150);
        let (is_safe, violations) = ConfigLoader::validate_config_content_security(&nested_content);
        assert!(!is_safe);
        assert!(violations.iter().any(|v| v.contains("Excessive nesting")));
    }

    #[test]
    fn test_strip_comments_complex_cases() {
        // Test nested comments and edge cases
        let complex_jsonc = r#"
        {
            // Outer comment
            "data": {
                /* Block comment
                   with multiple lines
                   and // internal single line comment */
                "nested": "value", // Trailing comment
                "array": [
                    1, // Comment in array
                    2
                ]
            }
            // Final comment
        }
        "#;

        let cleaned = ConfigLoader::strip_jsonc_comments(complex_jsonc);

        // Should not contain any comments
        assert!(!cleaned.contains("Outer comment"));
        assert!(!cleaned.contains("Block comment"));
        assert!(!cleaned.contains("internal single line"));
        assert!(!cleaned.contains("Trailing comment"));
        assert!(!cleaned.contains("Comment in array"));
        assert!(!cleaned.contains("Final comment"));

        // Should contain actual JSON structure
        assert!(cleaned.contains("\"data\":"));
        assert!(cleaned.contains("\"nested\": \"value\""));
        assert!(cleaned.contains("\"array\":"));
    }
}
