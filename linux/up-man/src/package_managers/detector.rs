use crate::config::model::PackageManagerConfig;
use anyhow::Result;
use log::{debug, info};
use std::collections::HashSet;
use which::which;

/// Detects package managers available on the system
pub struct PackageManagerDetector;

impl PackageManagerDetector {
    /// Detects package managers that are installed but not in the current configuration
    ///
    /// # Arguments
    /// * `current_configs` - A slice of existing package manager configurations
    ///
    /// # Returns
    /// * `Result<Vec<PackageManagerDefinition>>` - A list of detected but unconfigured package managers
    pub fn detect_unconfigured(
        &self,
        current_configs: &[PackageManagerConfig],
    ) -> Result<Vec<PackageManagerDefinition>> {
        info!("Checking for potentially unconfigured package managers...");

        let checks = self.get_known_package_managers();
        let mut detected = Vec::new();

        // Use a HashSet for efficient lookup of configured names
        let configured_names: HashSet<_> = current_configs
            .iter()
            .map(|c| c.name.to_uppercase())
            .collect();

        for definition in checks {
            // Check if command exists using 'which' crate
            if which(&definition.command_to_check).is_ok() {
                debug!("Found command: {}", definition.command_to_check);
                // Check if not already configured by name (case insensitive)
                let upper_name = definition.name.to_uppercase();
                if !configured_names.contains(&upper_name) {
                    debug!("Detected package manager: {}", definition.name);
                    detected.push(definition);
                }
            }
        }

        Ok(detected)
    }

    /// Returns a list of known package managers with their detection commands and configuration snippets
    fn get_known_package_managers(&self) -> Vec<PackageManagerDefinition> {
        vec![
            // System package managers
            PackageManagerDefinition {
                name: "APT",
                command_to_check: "apt",
                default_toml_snippet: r#"[[package_manager]]
name = "APT"
enabled = true
command = "apt update && apt full-upgrade -y && apt autoremove -y && apt autoclean"
needs-sudo = true"#,
            },
            PackageManagerDefinition {
                name: "SNAP",
                command_to_check: "snap",
                default_toml_snippet: r#"[[package_manager]]
name = "SNAP"
enabled = true
command = "snap refresh"
needs-sudo = true"#,
            },
            PackageManagerDefinition {
                name: "FLATPAK",
                command_to_check: "flatpak",
                default_toml_snippet: r#"[[package_manager]]
name = "FLATPAK"
enabled = true
command = "flatpak update -y"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "DNF",
                command_to_check: "dnf",
                default_toml_snippet: r#"[[package_manager]]
name = "DNF"
enabled = true
command = "dnf upgrade -y"
needs-sudo = true"#,
            },
            PackageManagerDefinition {
                name: "YUM",
                command_to_check: "yum",
                default_toml_snippet: r#"[[package_manager]]
name = "YUM"
enabled = true
command = "yum update -y"
needs-sudo = true"#,
            },
            PackageManagerDefinition {
                name: "PACMAN",
                command_to_check: "pacman",
                default_toml_snippet: r#"[[package_manager]]
name = "PACMAN"
enabled = true
command = "pacman -Syu --noconfirm"
needs-sudo = true"#,
            },
            PackageManagerDefinition {
                name: "ZYPPER",
                command_to_check: "zypper",
                default_toml_snippet: r#"[[package_manager]]
name = "ZYPPER"
enabled = true
command = "zypper update -y"
needs-sudo = true"#,
            },
            // Language and tool specific package managers
            PackageManagerDefinition {
                name: "BREW",
                command_to_check: "brew",
                default_toml_snippet: r#"[[package_manager]]
name = "BREW"
enabled = true
command = "brew update && brew upgrade"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "RUSTUP",
                command_to_check: "rustup",
                default_toml_snippet: r#"[[package_manager]]
name = "RUSTUP"
enabled = true
command = "rustup update"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "CARGO",
                command_to_check: "cargo",
                default_toml_snippet: r#"[[package_manager]]
name = "CARGO"
enabled = true
command = "cargo install-update -a"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "NPM",
                command_to_check: "npm",
                default_toml_snippet: r#"[[package_manager]]
name = "NPM"
enabled = true
command = "npm update -g"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "PIP",
                command_to_check: "pip",
                default_toml_snippet: r#"[[package_manager]]
name = "PIP"
enabled = true
command = "pip list --outdated --format=json | jq -r '.[] | .name' | xargs -n1 pip install -U"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "PIP3",
                command_to_check: "pip3",
                default_toml_snippet: r#"[[package_manager]]
name = "PIP3"
enabled = true
command = "pip3 list --outdated --format=json | jq -r '.[] | .name' | xargs -n1 pip3 install -U"
needs-sudo = false"#,
            },
            PackageManagerDefinition {
                name: "GEM",
                command_to_check: "gem",
                default_toml_snippet: r#"[[package_manager]]
name = "GEM"
enabled = true
command = "gem update"
needs-sudo = false"#,
            },
        ]
    }

    /// Test helper method that allows injecting package manager definitions
    #[cfg(test)]
    pub fn detect_unconfigured_with_defs(
        &self,
        current_configs: &[PackageManagerConfig],
        definitions: Vec<PackageManagerDefinition>,
    ) -> Result<Vec<PackageManagerDefinition>> {
        info!("Checking for potentially unconfigured package managers...");

        let mut detected = Vec::new();

        // Use a HashSet for efficient lookup of configured names
        let configured_names: HashSet<_> = current_configs
            .iter()
            .map(|c| c.name.to_uppercase())
            .collect();

        for definition in definitions {
            // Check if command exists using 'which' crate
            if which(&definition.command_to_check).is_ok() {
                debug!("Found command: {}", definition.command_to_check);
                // Check if not already configured by name (case insensitive)
                let upper_name = definition.name.to_uppercase();
                if !configured_names.contains(&upper_name) {
                    debug!("Detected package manager: {}", definition.name);
                    detected.push(definition);
                }
            }
        }

        Ok(detected)
    }
}

/// Represents a package manager that can be detected and configured
#[derive(Debug, Clone)]
pub struct PackageManagerDefinition {
    /// The display name of the package manager
    pub name: &'static str,
    /// The command to check for existence
    pub command_to_check: &'static str,
    /// Default TOML configuration snippet for this package manager
    pub default_toml_snippet: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_mock_executable(dir: &TempDir, name: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, "#!/bin/sh\necho 'Mock executable'").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn test_detect_unconfigured_pm() {
        // Create a temporary directory for mock executables
        let temp_dir = TempDir::new().unwrap();
        let _mock_path = create_mock_executable(&temp_dir, "mock-pm");

        // Add the temp directory to PATH
        let original_path = env::var("PATH").unwrap_or_default();
        env::set_var(
            "PATH",
            format!("{}:{}", temp_dir.path().display(), original_path),
        );

        // Create a detector and some configs
        let detector = PackageManagerDetector;
        let configs = vec![PackageManagerConfig {
            name: "EXISTING_PM".to_string(),
            enabled: true,
            command: "command".to_string(),
            needs_sudo: false,
        }];

        // Mock our package manager list to include the mock executable
        let test_pm_def = PackageManagerDefinition {
            name: "MOCK_PM",
            command_to_check: "mock-pm",
            default_toml_snippet: "# mock snippet",
        };

        // Call detect_unconfigured with our mocked implementation
        let detected = detector
            .detect_unconfigured_with_defs(&configs, vec![test_pm_def])
            .unwrap();

        // Restore PATH
        env::set_var("PATH", original_path);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].name, "MOCK_PM");
    }

    #[test]
    fn test_no_detection_when_already_configured() {
        // Create a temporary directory for mock executables
        let temp_dir = TempDir::new().unwrap();
        let _mock_path = create_mock_executable(&temp_dir, "mock-pm");

        // Add the temp directory to PATH
        let original_path = env::var("PATH").unwrap_or_default();
        env::set_var(
            "PATH",
            format!("{}:{}", temp_dir.path().display(), original_path),
        );

        // Create a detector and some configs
        let detector = PackageManagerDetector;
        let configs = vec![PackageManagerConfig {
            name: "MOCK_PM".to_string(), // Same as our mock PM (case insensitive)
            enabled: true,
            command: "command".to_string(),
            needs_sudo: false,
        }];

        // Mock our package manager list to include the mock executable
        let test_pm_def = PackageManagerDefinition {
            name: "mock_pm", // Lowercase, to test case-insensitive matching
            command_to_check: "mock-pm",
            default_toml_snippet: "# mock snippet",
        };

        // Call detect_unconfigured with our mocked implementation
        let detected = detector
            .detect_unconfigured_with_defs(&configs, vec![test_pm_def])
            .unwrap();

        // Restore PATH
        env::set_var("PATH", original_path);

        assert_eq!(detected.len(), 0); // Should not detect anything as it's already configured
    }

    // TODO: Test new detection features for v0.2.0:
    // - Test version detection
    // - Test detection of similar package managers
}
