use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct AliasManager;

impl AliasManager {
    /// Sets up an alias in the user's shell configuration
    pub fn setup_alias(name: &str, executable_path: &Path) -> Result<()> {
        info!(
            "Setting up alias '{}' to point to {}",
            name,
            executable_path.display()
        );

        // Get executable absolute path if not already absolute
        let executable_path = if executable_path.is_absolute() {
            executable_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(executable_path)
        };

        // Ensure the file exists
        if !executable_path.exists() {
            return Err(anyhow::anyhow!(
                "Executable not found: {}",
                executable_path.display()
            ));
        }

        // Try to detect shell and set up alias
        let shell_config_path = Self::detect_shell_config()?;

        if let Some(path) = shell_config_path {
            Self::add_alias_to_config(&path, name, &executable_path)
        } else {
            warn!("Could not detect shell configuration file.");
            info!("To manually set up alias, add the following line to your shell config:");
            info!("alias {}='{}'", name, executable_path.display());
            Ok(())
        }
    }

    /// Detects user's shell and returns the path to its configuration file
    fn detect_shell_config() -> Result<Option<PathBuf>> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;

        // Try to detect shell from SHELL environment variable
        if let Ok(shell) = std::env::var("SHELL") {
            debug!("SHELL environment variable: {}", shell);

            if shell.contains("bash") {
                let bash_rc = home_dir.join(".bashrc");
                if bash_rc.exists() {
                    return Ok(Some(bash_rc));
                }
            } else if shell.contains("zsh") {
                let zshrc = home_dir.join(".zshrc");
                if zshrc.exists() {
                    return Ok(Some(zshrc));
                }
            } else if shell.contains("fish") {
                let fish_config = home_dir.join(".config/fish/config.fish");
                if fish_config.exists() {
                    return Ok(Some(fish_config));
                }
            }
        }

        // Fallback to checking common config files
        let configs = [
            home_dir.join(".bashrc"),
            home_dir.join(".zshrc"),
            home_dir.join(".config/fish/config.fish"),
            home_dir.join(".profile"),
        ];

        for config in configs {
            if config.exists() {
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    /// Adds the alias to the specified shell configuration file
    fn add_alias_to_config(
        config_path: &Path,
        alias_name: &str,
        executable_path: &Path,
    ) -> Result<()> {
        // Check if alias already exists
        let alias_line = format!("alias {}='{}'", alias_name, executable_path.display());
        let file = fs::File::open(config_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim() == alias_line.trim() {
                info!(
                    "Alias '{}' already exists in {}",
                    alias_name,
                    config_path.display()
                );
                return Ok(());
            }
        }

        // Append alias to the end of the file
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(config_path)?;

        writeln!(file, "\n# Added by up-man")?;
        writeln!(file, "{}", alias_line)?;

        info!("Alias '{}' added to {}", alias_name, config_path.display());
        info!(
            "To use it in your current shell, run 'source {}'",
            config_path.display()
        );

        Ok(())
    }
}
