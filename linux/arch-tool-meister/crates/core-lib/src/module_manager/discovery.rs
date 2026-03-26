//! Module discovery and validation service.
//!
//! This service scans directories for module configurations, validates them,
//! and creates ModuleInfo instances for the module management system.

use crate::error::CoreError;
use serde_json::Value;
use shared_types::module::{ModuleDependency, ModuleInfo};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, io::AsyncReadExt};
use tracing::{debug, info, instrument, warn};

/// Default module search paths
const DEFAULT_MODULE_PATHS: &[&str] = &[
    "./modules",
    "/usr/share/arch-tool-meister/modules",
    "/opt/arch-tool-meister/modules",
];

/// Service for discovering and validating modules from the filesystem.
#[derive(Debug)]
pub struct ModuleDiscovery {
    /// Base directories to scan for modules
    scan_directories: Vec<PathBuf>,
    /// Cache of discovered modules
    discovered_modules: Arc<tokio::sync::RwLock<HashMap<String, ModuleInfo>>>,
}

impl ModuleDiscovery {
    /// Create a new module discovery service
    pub fn new() -> Self {
        let mut discovery = Self {
            scan_directories: Vec::new(),
            discovered_modules: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        };

        // Add default search paths
        for path_str in DEFAULT_MODULE_PATHS {
            let path = PathBuf::from(path_str);
            if path.exists() {
                discovery.scan_directories.push(path);
            }
        }

        // Add user-specific module path if it exists
        if let Some(home_dir) = std::env::var_os("HOME") {
            let user_modules = PathBuf::from(home_dir)
                .join(".config")
                .join("arch-tool-meister")
                .join("modules");
            if user_modules.exists() {
                discovery.scan_directories.push(user_modules);
            }
        }

        discovery
    }

    /// Create discovery service with custom scan directories
    pub fn with_directories(directories: Vec<PathBuf>) -> Self {
        Self {
            scan_directories: directories,
            discovered_modules: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Add a search path for module discovery
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.scan_directories.contains(&path) {
            debug!("Adding module search path: {:?}", path);
            self.scan_directories.push(path);
        }
    }

    /// Scan all configured directories for modules
    #[instrument(skip(self))]
    pub async fn discover_all_modules(&self) -> Result<HashMap<String, ModuleInfo>, CoreError> {
        let mut all_modules = HashMap::new();

        info!(
            "Starting module discovery in {} paths",
            self.scan_directories.len()
        );

        for directory in &self.scan_directories {
            if !directory.exists() {
                warn!("Module directory does not exist: {}", directory.display());
                continue;
            }

            match self.scan_directory(directory).await {
                Ok(modules) => {
                    info!(
                        "Discovered {} modules in {}",
                        modules.len(),
                        directory.display()
                    );
                    all_modules.extend(modules);
                }
                Err(e) => {
                    warn!("Failed to scan directory {}: {}", directory.display(), e);
                }
            }
        }

        // Update cache
        let mut cache = self.discovered_modules.write().await;
        *cache = all_modules.clone();

        info!("Total modules discovered: {}", all_modules.len());
        Ok(all_modules)
    }

    /// Scan a specific directory for modules
    #[instrument(skip(self))]
    async fn scan_directory(
        &self,
        directory: &Path,
    ) -> Result<HashMap<String, ModuleInfo>, CoreError> {
        let mut modules = HashMap::new();

        debug!("Scanning directory: {:?}", directory);

        let mut entries = fs::read_dir(directory).await.map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to read directory: {}: {}",
                directory.display(),
                e
            )))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to read directory entry in: {}: {}",
                directory.display(),
                e
            )))
        })? {
            let path = entry.path();
            if path.is_dir() {
                // Check for module config file
                let module_config_path = path.join("module.jsonc");
                let alt_config_path = path.join("module.json");

                let config_path = if module_config_path.exists() {
                    module_config_path
                } else if alt_config_path.exists() {
                    alt_config_path
                } else {
                    // Also recursively scan subdirectories
                    if let Ok(subdir_modules) = Box::pin(self.scan_directory(&path)).await {
                        modules.extend(subdir_modules);
                    } else {
                        warn!("Failed to scan subdirectory {:?}", path);
                    }
                    continue;
                };

                match self.load_module_config(&config_path).await {
                    Ok(module_info) => {
                        debug!("Loaded module: {}", module_info.name);
                        modules.insert(module_info.name.clone(), module_info);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load module config from {}: {}",
                            config_path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(modules)
    }

    /// Load and validate a module configuration file
    async fn load_module_config(&self, config_path: &Path) -> Result<ModuleInfo, CoreError> {
        debug!("Loading module from: {:?}", config_path);

        // Read the JSONC file
        let content = self.read_config_file(config_path).await?;

        // Parse JSONC (JSON with comments)
        let config_value = self.parse_jsonc(&content)?;

        // Convert to ModuleInfo
        let mut module_info = self.parse_module_config(config_value)?;

        // Set additional metadata
        if let Some(parent_dir) = config_path.parent() {
            if let Some(module_name) = parent_dir.file_name().and_then(|n| n.to_str()) {
                // Use directory name as module name if not specified
                if module_info.name.is_empty() {
                    module_info.name = module_name.to_string();
                }
            }
        }

        // Validate the module
        self.validate_module(&module_info)?;

        Ok(module_info)
    }

    /// Read configuration file content
    async fn read_config_file(&self, path: &Path) -> Result<String, CoreError> {
        let mut file = fs::File::open(path).await.map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to open module config {:?}: {}",
                path, e
            )))
        })?;

        let mut content = String::new();
        file.read_to_string(&mut content).await.map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to read module config {:?}: {}",
                path, e
            )))
        })?;

        Ok(content)
    }

    /// Parse JSONC (JSON with comments) content
    fn parse_jsonc(&self, content: &str) -> Result<Value, CoreError> {
        // Remove comments for basic JSONC support
        let cleaned = self.remove_jsonc_comments(content);

        serde_json::from_str(&cleaned).map_err(|e| {
            CoreError::from(shared_types::SharedError::parse(format!(
                "Invalid JSON in module config: {}",
                e
            )))
        })
    }

    /// Simple JSONC comment removal (line comments only)
    fn remove_jsonc_comments(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| {
                // Find the first occurrence of "//" not inside quotes
                let mut in_string = false;
                let mut escape_next = false;

                for (i, char) in line.char_indices() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }

                    match char {
                        '"' if !escape_next => in_string = !in_string,
                        '\\' if in_string => escape_next = true,
                        '/' if !in_string && line.chars().nth(i + 1) == Some('/') => {
                            return &line[..i];
                        }
                        _ => {}
                    }
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse module configuration from JSON value
    fn parse_module_config(&self, value: Value) -> Result<ModuleInfo, CoreError> {
        let obj = value.as_object().ok_or_else(|| {
            CoreError::from(shared_types::SharedError::parse(
                "Module config must be a JSON object".to_string(),
            ))
        })?;

        // Extract required fields
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let version = obj
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

        // Parse settings
        let settings = obj
            .get("settings")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_else(HashMap::new);

        // Parse menu
        let menu = if let Some(menu_value) = obj.get("menu") {
            self.parse_module_menu(menu_value)?
        } else {
            shared_types::module::ModuleMenu::default()
        };

        // Parse commands (old format as hashmap)
        let commands = if let Some(commands_value) = obj.get("commands") {
            self.parse_module_commands_hashmap(commands_value)?
        } else {
            HashMap::new()
        };

        // Parse functions
        let functions = if let Some(functions_value) = obj.get("functions") {
            self.parse_module_functions(functions_value)?
        } else {
            HashMap::new()
        };

        Ok(ModuleInfo {
            name,
            description,
            version,
            enabled,
            settings,
            menu,
            commands,
            functions,
        })
    }

    /// Parse module menu from JSON value
    fn parse_module_menu(
        &self,
        value: &Value,
    ) -> Result<shared_types::module::ModuleMenu, CoreError> {
        if let Some(menu_obj) = value.as_object() {
            let title = menu_obj
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Module Menu")
                .to_string();

            let main_menu_entry = menu_obj
                .get("mainMenuEntry")
                .and_then(|v| v.as_str())
                .unwrap_or("Module")
                .to_string();

            let options =
                if let Some(options_array) = menu_obj.get("options").and_then(|v| v.as_array()) {
                    self.parse_menu_options(options_array)?
                } else {
                    vec![shared_types::module::MenuOption {
                        text: "Return to Main Menu".to_string(),
                        option_type: shared_types::module::MenuOptionType::Return,
                        function_name: None,
                    }]
                };

            Ok(shared_types::module::ModuleMenu {
                title,
                main_menu_entry,
                options,
            })
        } else {
            Ok(shared_types::module::ModuleMenu::default())
        }
    }

    /// Parse menu options from JSON array
    fn parse_menu_options(
        &self,
        options_array: &[Value],
    ) -> Result<Vec<shared_types::module::MenuOption>, CoreError> {
        let mut options = Vec::new();

        for option_value in options_array {
            if let Some(option_obj) = option_value.as_object() {
                let text = option_obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let option_type_str = option_obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("return");

                let option_type = match option_type_str {
                    "scriptFunction" => shared_types::module::MenuOptionType::ScriptFunction,
                    "return" => shared_types::module::MenuOptionType::Return,
                    "navigate" => shared_types::module::MenuOptionType::Navigate,
                    other => shared_types::module::MenuOptionType::Custom(other.to_string()),
                };

                let function_name = option_obj
                    .get("functionName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                options.push(shared_types::module::MenuOption {
                    text,
                    option_type,
                    function_name,
                });
            }
        }

        Ok(options)
    }

    /// Parse module commands from JSON value as HashMap (old format)
    fn parse_module_commands_hashmap(
        &self,
        value: &Value,
    ) -> Result<HashMap<String, shared_types::module::ModuleCommand>, CoreError> {
        let mut commands = HashMap::new();

        if let Some(commands_obj) = value.as_object() {
            for (cmd_name, cmd_value) in commands_obj {
                if let Some(cmd_obj) = cmd_value.as_object() {
                    let description = cmd_obj
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let function = cmd_obj
                        .get("function")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let args = cmd_obj
                        .get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_else(Vec::new);

                    commands.insert(
                        cmd_name.clone(),
                        shared_types::module::ModuleCommand {
                            description,
                            function,
                            args,
                        },
                    );
                }
            }
        }

        Ok(commands)
    }

    /// Parse module functions from JSON value
    fn parse_module_functions(
        &self,
        value: &Value,
    ) -> Result<HashMap<String, shared_types::module::ModuleFunction>, CoreError> {
        let mut functions = HashMap::new();

        if let Some(functions_obj) = value.as_object() {
            for (func_name, func_value) in functions_obj {
                if let Some(func_obj) = func_value.as_object() {
                    let code = func_obj
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    functions.insert(
                        func_name.clone(),
                        shared_types::module::ModuleFunction { code },
                    );
                } else if let Some(code_str) = func_value.as_str() {
                    // Handle simple string function definition
                    functions.insert(
                        func_name.clone(),
                        shared_types::module::ModuleFunction {
                            code: code_str.to_string(),
                        },
                    );
                }
            }
        }

        Ok(functions)
    }

    /// Parse module dependencies from JSON value (legacy support)
    fn parse_module_dependencies(&self, value: &Value) -> Result<Vec<ModuleDependency>, CoreError> {
        let mut dependencies = Vec::new();

        if let Some(deps_array) = value.as_array() {
            for dep_value in deps_array {
                if let Some(dep_str) = dep_value.as_str() {
                    // Simple string dependency - assume it's a system package
                    dependencies.push(ModuleDependency::system_package(dep_str));
                } else if let Some(dep_obj) = dep_value.as_object() {
                    // Complex dependency object
                    let dep_type = dep_obj
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("package");

                    match dep_type {
                        "systemPackage" | "package" => {
                            let name = dep_obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    CoreError::from(shared_types::SharedError::parse(
                                        "System package dependency name is required".to_string(),
                                    ))
                                })?
                                .to_string();

                            dependencies.push(ModuleDependency::system_package(name));
                        }
                        "command" => {
                            let name = dep_obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    CoreError::from(shared_types::SharedError::parse(
                                        "Command dependency name is required".to_string(),
                                    ))
                                })?
                                .to_string();

                            let check_command = dep_obj
                                .get("checkCommand")
                                .or_else(|| dep_obj.get("check_command"))
                                .and_then(|v| v.as_str())
                                .unwrap_or(&format!("which {}", name))
                                .to_string();

                            dependencies.push(ModuleDependency::command(name, check_command));
                        }
                        "file" => {
                            let path =
                                dep_obj
                                    .get("path")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        CoreError::from(shared_types::SharedError::parse(
                                            "File dependency path is required".to_string(),
                                        ))
                                    })?;

                            let required = dep_obj
                                .get("required")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true);

                            dependencies.push(ModuleDependency::file(path, required));
                        }
                        _ => {
                            warn!("Unknown dependency type: {}", dep_type);
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Validate a module configuration
    fn validate_module(&self, module: &ModuleInfo) -> Result<(), CoreError> {
        // Basic validation
        if module.name.trim().is_empty() {
            return Err(CoreError::from(shared_types::SharedError::validation(
                "Module name cannot be empty".to_string(),
            )));
        }

        // Security validation
        self.validate_security(module)?;

        // Command validation
        self.validate_commands(module)?;

        Ok(())
    }

    /// Perform security validation on module
    fn validate_security(&self, module: &ModuleInfo) -> Result<(), CoreError> {
        // Check functions for dangerous patterns
        for (func_name, function) in &module.functions {
            let code = &function.code;

            // Basic security checks
            if code.contains("rm -rf /") {
                return Err(CoreError::from(shared_types::SharedError::validation(
                    format!(
                        "Module '{}' function '{}' contains dangerous command pattern",
                        module.name, func_name
                    ),
                )));
            }

            if code.contains("sudo ") && !code.contains("sudo -n") {
                warn!(
                    "Module '{}' function '{}' uses sudo without -n flag",
                    module.name, func_name
                );
            }

            // Check for network operations
            if code.contains("curl") || code.contains("wget") {
                debug!(
                    "Module '{}' function '{}' performs network operations",
                    module.name, func_name
                );
            }
        }

        Ok(())
    }

    /// Validate module commands
    fn validate_commands(&self, module: &ModuleInfo) -> Result<(), CoreError> {
        for (cmd_name, command) in &module.commands {
            if cmd_name.trim().is_empty() {
                return Err(CoreError::from(shared_types::SharedError::validation(
                    format!("Module '{}' has command with empty name", module.name),
                )));
            }

            if command.function.trim().is_empty() {
                return Err(CoreError::from(shared_types::SharedError::validation(
                    format!(
                        "Module '{}' command '{}' has empty function reference",
                        module.name, cmd_name
                    ),
                )));
            }

            // Check if the referenced function exists
            if !module.functions.contains_key(&command.function) {
                return Err(CoreError::from(shared_types::SharedError::validation(
                    format!(
                        "Module '{}' command '{}' references non-existent function '{}'",
                        module.name, cmd_name, command.function
                    ),
                )));
            }
        }

        Ok(())
    }

    /// Get cached discovered modules
    pub async fn get_cached_modules(&self) -> HashMap<String, ModuleInfo> {
        let cache = self.discovered_modules.read().await;
        cache.clone()
    }

    /// Find a specific module by name
    pub async fn find_module(&self, name: &str) -> Option<ModuleInfo> {
        let cache = self.discovered_modules.read().await;
        cache.get(name).cloned()
    }

    /// Get modules by category or tag
    pub async fn filter_modules<F>(&self, predicate: F) -> Vec<ModuleInfo>
    where
        F: Fn(&ModuleInfo) -> bool,
    {
        let cache = self.discovered_modules.read().await;
        cache.values().filter(|m| predicate(m)).cloned().collect()
    }

    /// Refresh module discovery (re-scan directories)
    pub async fn refresh(&self) -> Result<(), CoreError> {
        info!("Refreshing module discovery");
        self.discover_all_modules().await?;
        Ok(())
    }

    /// Clear the discovery cache
    pub fn clear_cache(&mut self) {
        // We can't clear the cache directly since it's async, but we can implement this
        // by having a method that clears on next operation
    }

    /// Get the current search paths
    pub fn get_search_paths(&self) -> &[PathBuf] {
        &self.scan_directories
    }
}
