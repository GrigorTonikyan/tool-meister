use crate::error::{Error, Result};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoMetadata {
    pub settings: MetadataSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataSettings {
    pub defaults: DefaultSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultSettings {
    pub app_config_path: Vec<String>,
    pub config_file_name: String,
    pub manifests_dir: String,
    pub tools_dir: Vec<String>,
    pub tools_sources_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    /// Paths to search for tool manifests (local directories and URLs)
    pub manifest_sources: Vec<ManifestSource>,
    /// Base directory where tools should be installed/downloaded
    pub tools_dir: PathBuf,
    /// Default manifest directory
    pub default_manifest_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestSource {
    /// Type of source: "local", "git", "url"
    #[serde(rename = "type")]
    pub source_type: String,
    /// Path or URL to the source
    pub path: String,
    /// Optional branch for git sources
    pub branch: Option<String>,
    /// Whether this source should be updated automatically
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
}

fn default_auto_update() -> bool {
    true
}

impl Default for GlobalConfig {
    fn default() -> Self {
        // Try to load defaults from Cargo.toml metadata, fallback to hardcoded defaults
        match Self::load_from_cargo_metadata() {
            Ok(config) => config,
            Err(_) => {
                // Fallback to hardcoded defaults if metadata loading fails
                Self {
                    manifest_sources: vec![ManifestSource {
                        source_type: "local".to_string(),
                        path: "manifests".to_string(),
                        branch: None,
                        auto_update: false,
                    }],
                    tools_dir: PathBuf::from("tools"),

                    default_manifest_dir: PathBuf::from("manifests"),
                }
            }
        }
    }
}

impl GlobalConfig {
    pub fn load() -> Result<Self> {
        // Load from global config path only
        let config_path = Self::get_config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).with_context(|| {
                format!(
                    "Failed to read global config file: {}",
                    config_path.display()
                )
            })?;

            let config: GlobalConfig = toml::from_str(&content).with_context(|| {
                format!(
                    "Failed to parse global config file: {}",
                    config_path.display()
                )
            })?;

            Ok(config)
        } else {
            // Create default config file
            let default_config = GlobalConfig::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Io(e))?;
        }

        let toml_content = toml::to_string_pretty(self).map_err(|e| Error::TomlSer(e))?;

        std::fs::write(&config_path, toml_content).map_err(|e| Error::Io(e))?;

        Ok(())
    }

    pub fn get_config_path() -> PathBuf {
        const APP_NAME: &str = env!("CARGO_PKG_NAME");

        // Get config file name from metadata
        let config_file_name = Self::get_config_file_name();

        // Try to use XDG config directory, fall back to ~/.config
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
                .join(APP_NAME)
                .join(config_file_name)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join(APP_NAME)
                .join(config_file_name)
        } else {
            PathBuf::from(".config")
        }
    }

    fn get_config_file_name() -> String {
        // Use the metadata embedded at build time
        let metadata_json = env!("PACKAGE_METADATA_JSON");
        let metadata: DefaultSettings = serde_json::from_str(metadata_json)
            .expect("Failed to parse embedded package metadata for config file name");
        metadata.config_file_name
    }

    pub fn get_tools_directory(&self) -> &PathBuf {
        &self.tools_dir
    }

    pub fn find_tool_manifest(&self, tool_name: &str) -> Result<Option<PathBuf>> {
        for source in &self.manifest_sources {
            match source.source_type.as_str() {
                "local" => {
                    let manifest_path =
                        PathBuf::from(&source.path).join(format!("{}.jsonc", tool_name));
                    if manifest_path.exists() {
                        return Ok(Some(manifest_path));
                    }
                }
                "git" => {
                    // For git sources, check if already cloned locally
                    let local_path = PathBuf::from(".manifest-cache")
                        .join(Self::sanitize_url(&source.path))
                        .join(format!("{}.jsonc", tool_name));
                    if local_path.exists() {
                        return Ok(Some(local_path));
                    }
                }
                "url" => {
                    // For URL sources, check cached version
                    let local_path = PathBuf::from(".manifest-cache")
                        .join("url-manifests")
                        .join(format!("{}.jsonc", tool_name));
                    if local_path.exists() {
                        return Ok(Some(local_path));
                    }
                }
                _ => {
                    println!(
                        "Warning: Unknown manifest source type: {}",
                        source.source_type
                    );
                }
            }
        }
        Ok(None)
    }

    fn sanitize_url(url: &str) -> String {
        url.replace(['/', ':', '.'], "_")
    }

    pub fn add_manifest_source(
        &mut self,
        source_type: String,
        path: String,
        branch: Option<String>,
        auto_update: bool,
    ) -> Result<String> {
        // Validate source type
        match source_type.as_str() {
            "local" | "git" | "url" => {}
            _ => {
                return Err(crate::error::Error::Config(format!(
                    "Invalid source type '{}'. Must be one of: local, git, url",
                    source_type
                )));
            }
        }

        // Validate and resolve path based on source type
        let validated_path = match source_type.as_str() {
            "local" => {
                // Resolve to absolute path
                let path_buf = PathBuf::from(&path);
                let absolute_path = if path_buf.is_absolute() {
                    path_buf
                } else {
                    std::env::current_dir()
                        .context("Failed to get current directory")?
                        .join(path_buf)
                };

                // Canonicalize to resolve any .. or . components
                let canonical_path = absolute_path.canonicalize().with_context(|| {
                    format!(
                        "Path does not exist or cannot be accessed: {}",
                        absolute_path.display()
                    )
                })?;

                // Check if it's a directory
                if !canonical_path.is_dir() {
                    return Err(crate::error::Error::Config(format!(
                        "Local manifest source must be a directory: {}",
                        canonical_path.display()
                    )));
                }

                // Check if we can read the directory
                std::fs::read_dir(&canonical_path).with_context(|| {
                    format!("Cannot read directory: {}", canonical_path.display())
                })?;

                canonical_path.to_string_lossy().to_string()
            }
            "git" => {
                // For git URLs, do basic validation
                if !path.starts_with("http://")
                    && !path.starts_with("https://")
                    && !path.starts_with("git@")
                {
                    return Err(crate::error::Error::Config(format!(
                        "Git source must be a valid git URL (http://, https://, or git@): {}",
                        path
                    )));
                }
                path
            }
            "url" => {
                // For URLs, do basic validation
                if !path.starts_with("http://") && !path.starts_with("https://") {
                    return Err(crate::error::Error::Config(format!(
                        "URL source must be a valid HTTP/HTTPS URL: {}",
                        path
                    )));
                }
                path
            }
            _ => unreachable!(), // Already validated above
        };

        // Check if source already exists (using the validated path)
        let source_exists = self
            .manifest_sources
            .iter()
            .any(|source| source.source_type == source_type && source.path == validated_path);

        if source_exists {
            return Err(crate::error::Error::Config(format!(
                "Manifest source already exists: {} {}",
                source_type, validated_path
            )));
        }

        // Add the new source with validated path
        let new_source = ManifestSource {
            source_type,
            path: validated_path.clone(),
            branch,
            auto_update,
        };

        self.manifest_sources.push(new_source);
        Ok(validated_path)
    }

    fn load_from_cargo_metadata() -> Result<Self> {
        // Use the metadata embedded at build time
        let metadata_json = env!("PACKAGE_METADATA_JSON");
        let metadata: DefaultSettings = serde_json::from_str(metadata_json)
            .context("Failed to parse embedded package metadata")?;

        let package_name = env!("CARGO_PKG_NAME");

        // Parse app_config_path
        let resolved_config_dir =
            Self::resolve_config_path(&metadata.app_config_path, package_name)?;

        // Parse manifests_dir
        let manifests_dir = Self::resolve_template_path(
            &metadata.manifests_dir,
            package_name,
            &resolved_config_dir,
        )?;

        // Parse tools_dir
        let tools_dir = Self::resolve_tools_path(&metadata.tools_dir)?;

        Ok(Self {
            manifest_sources: vec![ManifestSource {
                source_type: "local".to_string(),
                path: manifests_dir.to_string_lossy().to_string(),
                branch: None,
                auto_update: false,
            }],
            tools_dir,
            default_manifest_dir: manifests_dir,
        })
    }
    fn resolve_config_path(paths: &[String], package_name: &str) -> Result<PathBuf> {
        for path_template in paths {
            let resolved_path = Self::expand_env_vars(path_template, package_name)?;

            // Skip paths that still contain unexpanded environment variables
            if resolved_path.contains('$') {
                continue;
            }

            let path = PathBuf::from(resolved_path);

            // Check if this path is usable (parent directories exist or can be created)
            if let Some(parent) = path.parent() {
                if parent.exists() || std::fs::create_dir_all(parent).is_ok() {
                    return Ok(path);
                }
            } else if path.exists() {
                return Ok(path);
            }
        }

        // If no paths work, fallback to current directory
        Ok(PathBuf::from("./"))
    }
    fn resolve_tools_path(paths: &[String]) -> Result<PathBuf> {
        for path_template in paths {
            let resolved_path = Self::expand_env_vars(path_template, "")?;

            // Skip paths that still contain unexpanded environment variables
            if resolved_path.contains('$') {
                continue;
            }

            let path = PathBuf::from(resolved_path);

            // Check if this path is usable
            if let Some(parent) = path.parent() {
                if parent.exists() || std::fs::create_dir_all(parent).is_ok() {
                    return Ok(path);
                }
            } else if path.exists() || std::fs::create_dir_all(&path).is_ok() {
                return Ok(path);
            }
        }

        // If no paths work, fallback to ./tools
        Ok(PathBuf::from("./tools"))
    }

    fn resolve_template_path(
        template: &str,
        package_name: &str,
        config_dir: &Path,
    ) -> Result<PathBuf> {
        // Replace template placeholders
        let expanded = template.replace(
            "<[package.metadata.settings.defaults.app_config_path]>",
            &config_dir.to_string_lossy(),
        );

        let expanded = Self::expand_env_vars(&expanded, package_name)?;
        Ok(PathBuf::from(expanded))
    }

    fn expand_env_vars(template: &str, package_name: &str) -> Result<String> {
        let mut result = template.to_string();

        // Replace package name placeholder
        result = result.replace("<[package.name]>", package_name);

        // Expand environment variables
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            result = result.replace("$XDG_CONFIG_HOME", &xdg_config);
        }

        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            result = result.replace("$XDG_DATA_HOME", &xdg_data);
        }

        if let Ok(home) = env::var("HOME") {
            result = result.replace("$HOME", &home);
        }

        // If we still have unexpanded variables, it means they're not set
        // For $XDG_DATA_HOME, fallback to $HOME/.local/share if $HOME is available
        if result.contains("$XDG_DATA_HOME") {
            if let Ok(home) = env::var("HOME") {
                result = result.replace("$XDG_DATA_HOME", &format!("{}/.local/share", home));
            }
        }

        // If we still have unexpanded variables, it means they're not available
        // Leave them unexpanded to signal an error in path resolution
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use tempfile::tempdir;

    #[test]
    fn test_expand_env_vars_with_package_name() {
        let result = GlobalConfig::expand_env_vars("<[package.name]>", "test-app").unwrap();
        assert_eq!(result, "test-app");
    }

    #[test]
    fn test_expand_env_vars_with_home() {
        unsafe { env::set_var("HOME", "/home/testuser") };
        let result = GlobalConfig::expand_env_vars("$HOME/.config", "").unwrap();
        assert_eq!(result, "/home/testuser/.config");
    }

    #[test]
    fn test_expand_env_vars_with_xdg_config_home() {
        unsafe { env::set_var("XDG_CONFIG_HOME", "/custom/config") };
        let result = GlobalConfig::expand_env_vars("$XDG_CONFIG_HOME/app", "").unwrap();
        assert_eq!(result, "/custom/config/app");
    }

    #[test]
    fn test_expand_env_vars_xdg_data_home_fallback() {
        unsafe { env::remove_var("XDG_DATA_HOME") };
        unsafe { env::set_var("HOME", "/home/testuser") };
        let result = GlobalConfig::expand_env_vars("$XDG_DATA_HOME/apps", "").unwrap();
        assert_eq!(result, "/home/testuser/.local/share/apps");
    }

    #[test]
    fn test_resolve_config_path_with_valid_path() {
        let temp_dir = tempdir().unwrap();
        let paths = vec![temp_dir.path().to_string_lossy().to_string()];

        let result = GlobalConfig::resolve_config_path(&paths, "test-app").unwrap();
        assert_eq!(result, temp_dir.path());
    }

    #[test]
    fn test_resolve_config_path_fallback() {
        let paths = vec!["$NONEXISTENT_VAR/config".to_string()];

        let result = GlobalConfig::resolve_config_path(&paths, "test-app").unwrap();
        assert_eq!(result, PathBuf::from("./"));
    }

    #[test]
    fn test_resolve_tools_path_with_valid_path() {
        let temp_dir = tempdir().unwrap();
        let paths = vec![temp_dir.path().to_string_lossy().to_string()];

        let result = GlobalConfig::resolve_tools_path(&paths).unwrap();
        assert_eq!(result, temp_dir.path());
    }

    #[test]
    fn test_resolve_tools_path_fallback() {
        let paths = vec!["$NONEXISTENT_VAR/tools".to_string()];

        let result = GlobalConfig::resolve_tools_path(&paths).unwrap();
        assert_eq!(result, PathBuf::from("./tools"));
    }

    #[test]
    fn test_add_manifest_source_local_valid() {
        let temp_dir = tempdir().unwrap();
        fs::create_dir_all(&temp_dir).unwrap();

        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "local".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
            None,
            true,
        );

        assert!(result.is_ok());
        let validated_path = result.unwrap();
        assert!(validated_path.starts_with('/'));
        assert_eq!(config.manifest_sources.len(), 2); // default + new one
    }

    #[test]
    fn test_add_manifest_source_local_nonexistent() {
        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "local".to_string(),
            "/nonexistent/path".to_string(),
            None,
            true,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_add_manifest_source_local_file_not_directory() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "local".to_string(),
            file_path.to_string_lossy().to_string(),
            None,
            true,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be a directory")
        );
    }

    #[test]
    fn test_add_manifest_source_git_valid() {
        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "git".to_string(),
            "https://github.com/example/repo.git".to_string(),
            Some("main".to_string()),
            true,
        );

        assert!(result.is_ok());
        let validated_path = result.unwrap();
        assert_eq!(validated_path, "https://github.com/example/repo.git");
        assert_eq!(config.manifest_sources.len(), 2);
    }

    #[test]
    fn test_add_manifest_source_git_invalid_url() {
        let mut config = GlobalConfig::default();
        let result =
            config.add_manifest_source("git".to_string(), "invalid-url".to_string(), None, true);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid git URL"));
    }

    #[test]
    fn test_add_manifest_source_url_valid() {
        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "url".to_string(),
            "https://example.com/manifests".to_string(),
            None,
            false,
        );

        assert!(result.is_ok());
        let validated_path = result.unwrap();
        assert_eq!(validated_path, "https://example.com/manifests");
        assert_eq!(config.manifest_sources.len(), 2);
    }

    #[test]
    fn test_add_manifest_source_url_invalid() {
        let mut config = GlobalConfig::default();
        let result = config.add_manifest_source(
            "url".to_string(),
            "ftp://example.com/manifests".to_string(),
            None,
            true,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTP/HTTPS URL"));
    }

    #[test]
    fn test_add_manifest_source_invalid_type() {
        let mut config = GlobalConfig::default();
        let result =
            config.add_manifest_source("invalid".to_string(), "/some/path".to_string(), None, true);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid source type")
        );
    }

    #[test]
    fn test_add_manifest_source_duplicate() {
        let temp_dir = tempdir().unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        let mut config = GlobalConfig::default();

        // Add source first time
        let result1 = config.add_manifest_source("local".to_string(), path.clone(), None, true);
        assert!(result1.is_ok());

        // Try to add same source again
        let result2 = config.add_manifest_source("local".to_string(), path, None, true);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_sanitize_url() {
        let result = GlobalConfig::sanitize_url("https://github.com/user/repo.git");
        assert_eq!(result, "https___github_com_user_repo_git");
    }

    #[test]
    fn test_find_tool_manifest_local_exists() {
        let temp_dir = tempdir().unwrap();
        let manifest_dir = temp_dir.path().join("manifests");
        fs::create_dir_all(&manifest_dir).unwrap();

        let manifest_content = r#"{"repo": {"name": "test"}, "actions": {}}"#;
        fs::write(manifest_dir.join("test-tool.jsonc"), manifest_content).unwrap();

        let mut config = GlobalConfig::default();
        config.manifest_sources = vec![ManifestSource {
            source_type: "local".to_string(),
            path: manifest_dir.to_string_lossy().to_string(),
            branch: None,
            auto_update: false,
        }];

        let result = config.find_tool_manifest("test-tool").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("test-tool.jsonc"));
    }

    #[test]
    fn test_find_tool_manifest_not_found() {
        let temp_dir = tempdir().unwrap();
        let manifest_dir = temp_dir.path().join("manifests");
        fs::create_dir_all(&manifest_dir).unwrap();

        let mut config = GlobalConfig::default();
        config.manifest_sources = vec![ManifestSource {
            source_type: "local".to_string(),
            path: manifest_dir.to_string_lossy().to_string(),
            branch: None,
            auto_update: false,
        }];

        let result = config.find_tool_manifest("nonexistent-tool").unwrap();
        assert!(result.is_none());
    }
}
