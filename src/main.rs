use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

mod commands;
mod config;
mod error;
mod global_config;

use config::Config;
use global_config::GlobalConfig;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = format!("{} - manage and run tools in a workspace", env!("CARGO_PKG_NAME")))]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[arg(short, long, global = true)]
    config_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a tool
    Install {
        /// Tool name (corresponds to config file name without extension)
        tool: String,
    },
    /// Update a tool
    Update {
        /// Tool name (corresponds to config file name without extension)
        tool: String,
    },
    /// Build a tool
    Build {
        /// Tool name (corresponds to config file name without extension)
        tool: String,
    },
    /// Run a tool
    #[command(trailing_var_arg = true)]
    Run {
        /// Tool name (corresponds to config file name without extension)
        tool: String,
        /// Force spawn mode (detach process) even with arguments
        #[arg(long, short = 's')]
        spawn: bool,
        /// Wait for completion even when spawn=true in config
        #[arg(long, short = 'w')]
        wait: bool,
        /// Additional arguments to pass to the tool
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Generate global configuration file
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Reset to default configuration
        #[arg(long)]
        reset: bool,
    },
    /// Manage manifest sources where the app looks for new tool manifests
    #[command(subcommand)]
    Manifests(ManifestCommands),
}

#[derive(Subcommand)]
enum ManifestCommands {
    /// Add a manifest source to the configuration
    AddSource {
        /// Type of source: local, git, or url
        #[arg(short = 't', long)]
        source_type: String,
        /// Path or URL to the source
        path: String,
        /// Branch for git sources (optional)
        #[arg(short, long)]
        branch: Option<String>,
        /// Disable automatic updates
        #[arg(long)]
        no_auto_update: bool,
    },
    /// List all configured manifest sources
    List,
    /// Get information about available tools from each source
    Info {
        /// Show detailed information for specific source
        #[arg(short, long)]
        source: Option<String>,
    },
}

#[tokio::main]
async fn main() -> crate::error::Result<()> {
    let cli = Cli::parse();
    let global_config = GlobalConfig::load()?;

    // Determine manifest directory: CLI arg > global config > default
    let config_dir = cli
        .config_dir
        .unwrap_or_else(|| global_config.default_manifest_dir.clone());

    match cli.command {
        Commands::Install { tool } => {
            let config = load_tool_config(&global_config, &config_dir, &tool)?;
            commands::install::install_command(&config, &global_config).await?;
            println!(
                "✅ Installation of {} completed successfully!",
                config.repo.name
            );
        }
        Commands::Update { tool } => {
            let config = load_tool_config(&global_config, &config_dir, &tool)?;
            commands::update::update_command(&config, &global_config).await?;
            println!("✅ Update of {} completed successfully!", config.repo.name);
        }
        Commands::Build { tool } => {
            let config = load_tool_config(&global_config, &config_dir, &tool)?;
            commands::build::build_command(&config, &global_config).await?;
            println!("✅ Build of {} completed successfully!", config.repo.name);
        }
        Commands::Run {
            tool,
            spawn,
            wait,
            args,
        } => {
            let config = load_tool_config(&global_config, &config_dir, &tool)?;
            commands::run::run_command(&config, &args, spawn, wait, &global_config).await?;
            println!("✅ {} execution completed!", config.repo.name);
        }
        Commands::Config { show, reset } => {
            commands::config::config_command(show, reset, &global_config).await?;
        }
        Commands::Manifests(manifest_cmd) => match manifest_cmd {
            ManifestCommands::AddSource {
                source_type,
                path,
                branch,
                no_auto_update,
            } => {
                add_manifest_source(source_type, path, branch, !no_auto_update)?;
            }
            ManifestCommands::List => {
                list_manifest_sources(&global_config)?;
            }
            ManifestCommands::Info { source } => {
                show_manifest_info(&global_config, &source).await?;
            }
        },
    }

    Ok(())
}

fn load_tool_config(
    global_config: &GlobalConfig,
    fallback_dir: &std::path::Path,
    tool_name: &str,
) -> crate::error::Result<Config> {
    // First try to find manifest through global config sources
    if let Some(manifest_path) = global_config.find_tool_manifest(tool_name)? {
        return Config::load_from_path(&manifest_path);
    }

    // Fall back to local directory
    Config::load(fallback_dir, tool_name)
}

fn add_manifest_source(
    source_type: String,
    path: String,
    branch: Option<String>,
    auto_update: bool,
) -> crate::error::Result<()> {
    // Load current config (prefer project-local if available)
    let mut config = GlobalConfig::load()?;

    // Add the new source and get the validated path
    let validated_path =
        config.add_manifest_source(source_type.clone(), path, branch.clone(), auto_update)?;

    // Save the updated config
    config.save()?;

    // Print confirmation with the validated path
    let branch_info = match branch {
        Some(ref b) => format!(" (branch: {})", b),
        None => String::new(),
    };

    let auto_update_info = if auto_update {
        " with auto-update"
    } else {
        " without auto-update"
    };

    println!(
        "✅ Added manifest source: {} {}{}{}",
        source_type, validated_path, branch_info, auto_update_info
    );

    Ok(())
}

fn list_manifest_sources(global_config: &GlobalConfig) -> crate::error::Result<()> {
    println!("Configured manifest sources:");

    if global_config.manifest_sources.is_empty() {
        println!("  No manifest sources configured.");
        return Ok(());
    }

    for (index, source) in global_config.manifest_sources.iter().enumerate() {
        let auto_update_status = if source.auto_update {
            "auto-update"
        } else {
            "manual"
        };
        let branch_info = match &source.branch {
            Some(branch) => format!(" (branch: {})", branch),
            None => String::new(),
        };

        println!(
            "  {}: {} {} [{}]{}",
            index + 1,
            source.source_type,
            source.path,
            auto_update_status,
            branch_info
        );
    }

    Ok(())
}

async fn show_manifest_info(
    global_config: &GlobalConfig,
    source_filter: &Option<String>,
) -> crate::error::Result<()> {
    println!("Manifest source information:");

    for (index, source) in global_config.manifest_sources.iter().enumerate() {
        // If source filter is provided, skip sources that don't match
        if let Some(filter) = source_filter {
            if !source.path.contains(filter) && !source.source_type.contains(filter) {
                continue;
            }
        }

        println!(
            "\n📁 Source {}: {} {}",
            index + 1,
            source.source_type,
            source.path
        );

        match source.source_type.as_str() {
            "local" => {
                let manifest_dir = std::path::PathBuf::from(&source.path);
                if manifest_dir.exists() {
                    let entries = std::fs::read_dir(&manifest_dir)?;
                    let mut manifest_count = 0;

                    println!("  Available manifests:");
                    for entry in entries {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "jsonc") {
                            if let Some(name) = path.file_stem() {
                                println!("    - {}", name.to_string_lossy());
                                manifest_count += 1;
                            }
                        }
                    }

                    if manifest_count == 0 {
                        println!("    No manifest files found");
                    }
                } else {
                    println!("  ⚠️  Directory not found: {}", source.path);
                }
            }
            "git" => {
                println!("  Git repository source");
                if let Some(branch) = &source.branch {
                    println!("  Branch: {}", branch);
                }
                println!(
                    "  Auto-update: {}",
                    if source.auto_update {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                println!("  Note: Use 'update' command to fetch latest manifests");
            }
            "url" => {
                println!("  URL source");
                println!(
                    "  Auto-update: {}",
                    if source.auto_update {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                println!("  Note: Remote manifest content will be cached locally");
            }
            _ => {
                println!("  ⚠️  Unknown source type: {}", source.source_type);
            }
        }
    }

    if source_filter.is_some()
        && global_config.manifest_sources.iter().all(|s| {
            let filter = source_filter.as_ref().unwrap();
            !s.path.contains(filter) && !s.source_type.contains(filter)
        })
    {
        println!(
            "No sources found matching filter: {}",
            source_filter.as_ref().unwrap()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_config::ManifestSource;
    use std::{env, fs};
    use tempfile::tempdir;

    /// Helper function to run tests with isolated config
    fn with_test_config<T>(test_fn: T)
    where
        T: FnOnce(),
    {
        let temp_dir = tempdir().unwrap();
        let original_xdg = env::var("XDG_CONFIG_HOME").ok();

        // Set XDG_CONFIG_HOME to temporary directory
        unsafe { env::set_var("XDG_CONFIG_HOME", temp_dir.path()) };

        // Run the test
        test_fn();

        // Restore original environment
        match original_xdg {
            Some(val) => unsafe { env::set_var("XDG_CONFIG_HOME", val) },
            None => unsafe { env::remove_var("XDG_CONFIG_HOME") },
        }
    }

    #[test]
    fn test_list_manifest_sources_empty() {
        let mut config = GlobalConfig::default();
        config.manifest_sources.clear();

        let result = list_manifest_sources(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_manifest_sources_with_sources() {
        let mut config = GlobalConfig::default();
        config.manifest_sources.push(ManifestSource {
            source_type: "local".to_string(),
            path: "/test/path".to_string(),
            branch: None,
            auto_update: false,
        });
        config.manifest_sources.push(ManifestSource {
            source_type: "git".to_string(),
            path: "https://github.com/example/repo.git".to_string(),
            branch: Some("main".to_string()),
            auto_update: true,
        });

        let result = list_manifest_sources(&config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_manifest_info_local_source() {
        let temp_dir = tempdir().unwrap();
        let manifest_dir = temp_dir.path().join("manifests");
        fs::create_dir_all(&manifest_dir).unwrap();

        // Create test manifest
        let test_manifest = r#"{"repo": {"name": "test"}, "actions": {}}"#;
        fs::write(manifest_dir.join("test.jsonc"), test_manifest).unwrap();

        let mut config = GlobalConfig::default();
        config.manifest_sources.clear();
        config.manifest_sources.push(ManifestSource {
            source_type: "local".to_string(),
            path: manifest_dir.to_string_lossy().to_string(),
            branch: None,
            auto_update: false,
        });

        let result = show_manifest_info(&config, &None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_manifest_info_nonexistent_local_source() {
        let mut config = GlobalConfig::default();
        config.manifest_sources.clear();
        config.manifest_sources.push(ManifestSource {
            source_type: "local".to_string(),
            path: "/nonexistent/path".to_string(),
            branch: None,
            auto_update: false,
        });

        let result = show_manifest_info(&config, &None).await;
        assert!(result.is_ok()); // Should not fail, just show warning
    }

    #[tokio::test]
    async fn test_show_manifest_info_with_filter() {
        let mut config = GlobalConfig::default();
        config.manifest_sources.clear();
        config.manifest_sources.push(ManifestSource {
            source_type: "local".to_string(),
            path: "/test/local".to_string(),
            branch: None,
            auto_update: false,
        });
        config.manifest_sources.push(ManifestSource {
            source_type: "git".to_string(),
            path: "https://github.com/example/repo.git".to_string(),
            branch: None,
            auto_update: true,
        });

        let filter = Some("github".to_string());
        let result = show_manifest_info(&config, &filter).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_manifest_source_function() {
        with_test_config(|| {
            let temp_dir = tempdir().unwrap();
            let manifest_dir = temp_dir.path().join("manifests");
            fs::create_dir_all(&manifest_dir).unwrap();

            let result = add_manifest_source(
                "local".to_string(),
                manifest_dir.to_string_lossy().to_string(),
                None,
                true,
            );

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_add_manifest_source_invalid_path() {
        with_test_config(|| {
            let result = add_manifest_source(
                "local".to_string(),
                "/nonexistent/path".to_string(),
                None,
                true,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Path does not exist")
            );
        });
    }

    #[test]
    fn test_add_manifest_source_git_valid() {
        with_test_config(|| {
            let result = add_manifest_source(
                "git".to_string(),
                "https://github.com/example/repo.git".to_string(),
                Some("main".to_string()),
                true,
            );

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_add_manifest_source_git_invalid() {
        with_test_config(|| {
            let result =
                add_manifest_source("git".to_string(), "invalid-url".to_string(), None, true);

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("must be a valid git URL")
            );
        });
    }

    #[test]
    fn test_add_manifest_source_url_valid() {
        with_test_config(|| {
            let result = add_manifest_source(
                "url".to_string(),
                "https://example.com/manifests".to_string(),
                None,
                false,
            );

            if let Err(ref e) = result {
                eprintln!("Error: {}", e);
            }
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_add_manifest_source_url_invalid() {
        with_test_config(|| {
            let result = add_manifest_source(
                "url".to_string(),
                "ftp://example.com/manifests".to_string(),
                None,
                true,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("must be a valid HTTP/HTTPS URL")
            );
        });
    }
}
