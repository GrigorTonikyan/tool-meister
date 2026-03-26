//! Module-related types and data structures.
//!
//! This module defines the types used to represent modules, their metadata,
//! commands, dependencies, and status information based on the actual
//! module.jsonc structure used in the application.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module metadata and information matching module.jsonc structure.
///
/// This struct contains all the metadata associated with a module,
/// including its identification, description, configuration, and functionality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module name (unique identifier)
    pub name: String,
    /// Detailed description of what the module does
    pub description: String,
    /// Module version following semantic versioning
    pub version: String,
    /// Whether the module is enabled
    pub enabled: bool,
    /// Module-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Menu configuration for the module
    pub menu: ModuleMenu,
    /// Command definitions
    pub commands: HashMap<String, ModuleCommand>,
    /// Function implementations
    pub functions: HashMap<String, ModuleFunction>,
}

/// Menu configuration for a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleMenu {
    /// Menu title displayed when module is selected
    pub title: String,
    /// Entry text for the main menu
    #[serde(rename = "mainMenuEntry")]
    pub main_menu_entry: String,
    /// Menu options available in this module
    pub options: Vec<MenuOption>,
}

/// Individual menu option within a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuOption {
    /// Display text for the menu option
    pub text: String,
    /// Type of menu option
    #[serde(rename = "type")]
    pub option_type: MenuOptionType,
    /// Function name to execute (for scriptFunction type)
    #[serde(rename = "functionName")]
    pub function_name: Option<String>,
}

/// Type of menu option.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuOptionType {
    /// Execute a script function
    ScriptFunction,
    /// Return to previous menu
    Return,
    /// Navigate to another module
    Navigate,
    /// Custom action type
    Custom(String),
}

/// Command definition within a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCommand {
    /// Description of what the command does
    pub description: String,
    /// Function name to execute
    pub function: String,
    /// Arguments for the command
    pub args: Vec<String>,
}

/// Function implementation within a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleFunction {
    /// Shell code to execute
    pub code: String,
}

/// Module dependency specification.
///
/// Represents different types of dependencies that a module might require.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ModuleDependency {
    /// System package dependency (e.g., from pacman)
    SystemPackage {
        name: String,
        version: Option<String>,
        package_manager: Option<String>,
    },
    /// Command line tool dependency
    Command {
        name: String,
        check_command: String,
        install_command: Option<String>,
    },
    /// File or directory dependency
    File {
        path: String,
        required: bool,
        description: Option<String>,
    },
    /// Network service dependency
    Service {
        name: String,
        url: Option<String>,
        check_method: Option<String>,
    },
    /// Another module dependency
    Module {
        name: String,
        version: Option<String>,
        required: bool,
    },
    /// Environment variable dependency
    Environment {
        variable: String,
        expected_value: Option<String>,
        required: bool,
    },
}

/// Current status of a module.
///
/// Represents the current state and availability of a module in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleStatus {
    /// Module is available and ready to use
    Available,
    /// Module is installed and functional
    Installed,
    /// Module is installed but has updates available
    Outdated {
        current_version: String,
        latest_version: String,
    },
    /// Module has dependency issues
    DependencyMissing {
        missing_deps: Vec<String>,
        optional_missing: Vec<String>,
    },
    /// Module is disabled
    Disabled,
    /// Module is in an error state
    Error {
        message: String,
        error_code: Option<String>,
    },
    /// Module status is being checked
    Checking,
}

/// Builder for creating ModuleInfo instances.
pub struct ModuleInfoBuilder {
    module: ModuleInfo,
}

impl ModuleInfo {
    /// Create a new ModuleInfo with minimal required fields.
    pub fn new(name: String, description: String, version: String) -> Self {
        Self {
            name,
            description,
            version,
            enabled: true,
            settings: HashMap::new(),
            menu: ModuleMenu::default(),
            commands: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Create a builder for this module.
    pub fn builder(name: String, description: String, version: String) -> ModuleInfoBuilder {
        ModuleInfoBuilder {
            module: Self::new(name, description, version),
        }
    }

    /// Validate the module configuration.
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        // Validate required fields
        if self.name.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Module name cannot be empty",
                "name",
            ));
        }

        if self.description.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Module description cannot be empty",
                "description",
            ));
        }

        if self.version.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Module version cannot be empty",
                "version",
            ));
        }

        // Validate version format (basic semantic versioning)
        if !self.is_valid_semver(&self.version) {
            return Err(crate::error::SharedError::validation_with_field(
                "Module version must follow semantic versioning (e.g., 1.0.0)",
                "version",
            ));
        }

        // Validate menu configuration
        self.menu.validate()?;

        // Validate commands reference valid functions
        for (cmd_name, command) in &self.commands {
            if !self.functions.contains_key(&command.function) {
                return Err(crate::error::SharedError::validation_with_field(
                    format!(
                        "Command '{}' references undefined function '{}'",
                        cmd_name, command.function
                    ),
                    "commands",
                ));
            }
        }

        // Validate menu options reference valid commands
        for option in &self.menu.options {
            if let MenuOptionType::ScriptFunction = option.option_type {
                if let Some(ref func_name) = option.function_name {
                    if !self.commands.contains_key(func_name) {
                        return Err(crate::error::SharedError::validation_with_field(
                            format!("Menu option references undefined command '{}'", func_name),
                            "menu.options",
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if version follows basic semantic versioning
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// Get all available command names
    pub fn command_names(&self) -> Vec<&String> {
        self.commands.keys().collect()
    }

    /// Get all available function names
    pub fn function_names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// Check if the module has a specific command
    pub fn has_command(&self, command_name: &str) -> bool {
        self.commands.contains_key(command_name)
    }

    /// Check if the module has a specific function
    pub fn has_function(&self, function_name: &str) -> bool {
        self.functions.contains_key(function_name)
    }
}

impl ModuleInfoBuilder {
    /// Set the enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.module.enabled = enabled;
        self
    }

    /// Add a setting
    pub fn setting<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.module.settings.insert(key.into(), value.into());
        self
    }

    /// Set the menu configuration
    pub fn menu(mut self, menu: ModuleMenu) -> Self {
        self.module.menu = menu;
        self
    }

    /// Add a command
    pub fn command<K: Into<String>>(mut self, name: K, command: ModuleCommand) -> Self {
        self.module.commands.insert(name.into(), command);
        self
    }

    /// Add a function
    pub fn function<K: Into<String>>(mut self, name: K, function: ModuleFunction) -> Self {
        self.module.functions.insert(name.into(), function);
        self
    }

    /// Build the final ModuleInfo
    pub fn build(self) -> Result<ModuleInfo, crate::error::SharedError> {
        self.module.validate()?;
        Ok(self.module)
    }
}

impl Default for ModuleMenu {
    fn default() -> Self {
        Self {
            title: "Module Menu".to_string(),
            main_menu_entry: "Module".to_string(),
            options: vec![MenuOption {
                text: "Return to Main Menu".to_string(),
                option_type: MenuOptionType::Return,
                function_name: None,
            }],
        }
    }
}

impl ModuleMenu {
    /// Validate the menu configuration
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.title.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Menu title cannot be empty",
                "menu.title",
            ));
        }

        if self.main_menu_entry.is_empty() {
            return Err(crate::error::SharedError::validation_with_field(
                "Main menu entry cannot be empty",
                "menu.mainMenuEntry",
            ));
        }

        // Validate options
        for (index, option) in self.options.iter().enumerate() {
            option.validate().map_err(|e| {
                crate::error::SharedError::validation_with_field(
                    format!("Menu option {} validation failed: {}", index, e),
                    "menu.options",
                )
            })?;
        }

        Ok(())
    }

    /// Get options that are script functions
    pub fn script_function_options(&self) -> Vec<&MenuOption> {
        self.options
            .iter()
            .filter(|opt| matches!(opt.option_type, MenuOptionType::ScriptFunction))
            .collect()
    }

    /// Find option by function name
    pub fn find_option_by_function(&self, function_name: &str) -> Option<&MenuOption> {
        self.options.iter().find(|opt| {
            opt.function_name
                .as_ref()
                .map_or(false, |name| name == function_name)
        })
    }
}

impl MenuOption {
    /// Validate the menu option
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.text.is_empty() {
            return Err(crate::error::SharedError::validation(
                "Menu option text cannot be empty",
            ));
        }

        // Validate that scriptFunction type has a function name
        if matches!(self.option_type, MenuOptionType::ScriptFunction)
            && self.function_name.is_none()
        {
            return Err(crate::error::SharedError::validation(
                "ScriptFunction menu option must have a function name",
            ));
        }

        Ok(())
    }
}

impl ModuleCommand {
    /// Create a new command
    pub fn new<S: Into<String>>(description: S, function: S) -> Self {
        Self {
            description: description.into(),
            function: function.into(),
            args: Vec::new(),
        }
    }

    /// Add arguments to the command
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(|s| s.into()).collect();
        self
    }
}

impl ModuleFunction {
    /// Create a new function with code
    pub fn new<S: Into<String>>(code: S) -> Self {
        Self { code: code.into() }
    }

    /// Validate the function
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        if self.code.trim().is_empty() {
            return Err(crate::error::SharedError::validation(
                "Function code cannot be empty",
            ));
        }

        Ok(())
    }
}

impl ModuleDependency {
    /// Create a system package dependency
    pub fn system_package<S: Into<String>>(name: S) -> Self {
        Self::SystemPackage {
            name: name.into(),
            version: None,
            package_manager: None,
        }
    }

    /// Create a command dependency
    pub fn command<S: Into<String>>(name: S, check_command: S) -> Self {
        Self::Command {
            name: name.into(),
            check_command: check_command.into(),
            install_command: None,
        }
    }

    /// Create a file dependency
    pub fn file<S: Into<String>>(path: S, required: bool) -> Self {
        Self::File {
            path: path.into(),
            required,
            description: None,
        }
    }

    /// Validate the dependency
    pub fn validate(&self) -> Result<(), crate::error::SharedError> {
        match self {
            ModuleDependency::SystemPackage { name, .. } => {
                if name.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "System package name cannot be empty",
                    ));
                }
            }
            ModuleDependency::Command {
                name,
                check_command,
                ..
            } => {
                if name.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "Command name cannot be empty",
                    ));
                }
                if check_command.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "Check command cannot be empty",
                    ));
                }
            }
            ModuleDependency::File { path, .. } => {
                if path.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "File path cannot be empty",
                    ));
                }
            }
            ModuleDependency::Module { name, .. } => {
                if name.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "Module name cannot be empty",
                    ));
                }
            }
            ModuleDependency::Service { name, .. } => {
                if name.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "Service name cannot be empty",
                    ));
                }
            }
            ModuleDependency::Environment { variable, .. } => {
                if variable.is_empty() {
                    return Err(crate::error::SharedError::validation(
                        "Environment variable name cannot be empty",
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_info_creation() {
        let module = ModuleInfo::new(
            "test-module".to_string(),
            "A test module".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(module.name, "test-module");
        assert_eq!(module.description, "A test module");
        assert_eq!(module.version, "1.0.0");
        assert!(module.enabled);
        assert!(module.commands.is_empty());
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_module_builder() {
        let command = ModuleCommand::new("Test command", "test_function");
        let function = ModuleFunction::new("echo 'test'");

        let module = ModuleInfo::builder(
            "test".to_string(),
            "Test module".to_string(),
            "1.0.0".to_string(),
        )
        .enabled(false)
        .setting("key", "value")
        .command("test_cmd", command)
        .function("test_function", function)
        .build()
        .unwrap();

        assert_eq!(module.name, "test");
        assert!(!module.enabled);
        assert!(module.settings.contains_key("key"));
        assert!(module.commands.contains_key("test_cmd"));
        assert!(module.functions.contains_key("test_function"));
    }

    #[test]
    fn test_module_command_creation() {
        let command =
            ModuleCommand::new("Test command", "test_function").with_args(vec!["arg1", "arg2"]);

        assert_eq!(command.description, "Test command");
        assert_eq!(command.function, "test_function");
        assert_eq!(command.args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_validation_success() {
        let mut module = ModuleInfo::new(
            "valid-module".to_string(),
            "Valid module".to_string(),
            "1.0.0".to_string(),
        );

        // Add a valid command and function
        module.commands.insert(
            "test_cmd".to_string(),
            ModuleCommand::new("Test", "test_func"),
        );
        module
            .functions
            .insert("test_func".to_string(), ModuleFunction::new("echo 'test'"));

        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_validation_failures() {
        // Test empty name
        let mut module = ModuleInfo::new(
            String::new(),
            "Valid description".to_string(),
            "1.0.0".to_string(),
        );
        assert!(module.validate().is_err());

        // Test invalid version
        module = ModuleInfo::new(
            "valid-name".to_string(),
            "Valid description".to_string(),
            "invalid-version".to_string(),
        );
        assert!(module.validate().is_err());

        // Test command referencing non-existent function
        module = ModuleInfo::new(
            "valid-name".to_string(),
            "Valid description".to_string(),
            "1.0.0".to_string(),
        );
        module.commands.insert(
            "test_cmd".to_string(),
            ModuleCommand::new("Test", "non_existent_func"),
        );
        assert!(module.validate().is_err());
    }

    #[test]
    fn test_dependency_creation() {
        let sys_dep = ModuleDependency::system_package("git");
        assert!(matches!(sys_dep, ModuleDependency::SystemPackage { .. }));

        let cmd_dep = ModuleDependency::command("docker", "docker --version");
        assert!(matches!(cmd_dep, ModuleDependency::Command { .. }));

        let file_dep = ModuleDependency::file("/etc/hosts", true);
        assert!(matches!(file_dep, ModuleDependency::File { .. }));
    }

    #[test]
    fn test_menu_validation() {
        let mut menu = ModuleMenu::default();
        assert!(menu.validate().is_ok());

        // Test empty title
        menu.title = String::new();
        assert!(menu.validate().is_err());

        // Test empty main menu entry
        menu.title = "Valid Title".to_string();
        menu.main_menu_entry = String::new();
        assert!(menu.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let module = ModuleInfo::new(
            "test".to_string(),
            "Test module".to_string(),
            "1.0.0".to_string(),
        );

        let serialized = serde_json::to_string(&module).unwrap();
        let deserialized: ModuleInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(module, deserialized);
    }

    #[test]
    fn test_module_query_methods() {
        let mut module =
            ModuleInfo::new("test".to_string(), "Test".to_string(), "1.0.0".to_string());

        module
            .commands
            .insert("cmd1".to_string(), ModuleCommand::new("Command 1", "func1"));
        module
            .functions
            .insert("func1".to_string(), ModuleFunction::new("echo '1'"));

        assert!(module.has_command("cmd1"));
        assert!(!module.has_command("cmd2"));
        assert!(module.has_function("func1"));
        assert!(!module.has_function("func2"));

        let command_names = module.command_names();
        assert_eq!(command_names.len(), 1);
        assert!(command_names.contains(&&"cmd1".to_string()));
    }
}
