// filepath: /home/greg/dev/up-man/src/package_managers/runner.rs
use crate::config::model::PackageManagerConfig;
use crate::output::OutputManager;
use anyhow::{bail, Context, Result};
use log::debug;
use rayon::prelude::*;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use which::which;

/// Represents the result of a package manager update operation
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Name of the package manager
    pub name: String,
    /// Whether the update was successful
    pub success: bool,
    /// Duration of the update operation in milliseconds
    pub duration_ms: u128,
    /// Exit status code if available
    pub exit_status: Option<i32>,
    /// Error message if the update failed
    pub error_message: Option<String>,
}

/// Runs package manager update commands
pub struct PackageManagerRunner;

impl PackageManagerRunner {
    /// Runs an update for a single package manager
    ///
    /// # Arguments
    /// * `config` - The package manager configuration to run
    /// * `output` - The output manager for displaying progress
    ///
    /// # Returns
    /// * `Result<UpdateResult>` - The result of the update operation
    pub fn run_update(&self, config: &PackageManagerConfig, output: &OutputManager) -> Result<UpdateResult> {
        debug!("Starting update for {}", config.name);
        let start_time = Instant::now();

        if config.command.trim().is_empty() {
            return Ok(UpdateResult {
                name: config.name.clone(),
                success: false,
                duration_ms: 0,
                exit_status: None,
                error_message: Some("Empty command".to_string()),
            });
        }

        // Determine shell to use
        let shell = "bash";
        if which(shell).is_err() {
            bail!(
                "Shell '{}' not found, cannot execute command for {}",
                shell,
                config.name
            );
        }

        let mut cmd;
        if config.needs_sudo {
            // Check if sudo is available
            if which("sudo").is_err() {
                bail!("'sudo' command not found, but required for {}", config.name);
            }
            
            // Properly handle sudo prompt in TUI mode
            output.show_sudo_prompt(&config.name);
            
            cmd = Command::new("sudo");
            // Make sure sudo uses the TTY for password prompt
            cmd.arg("-S")
               .arg(shell)
               .arg("-c")
               .arg(&config.command)
               .stdin(Stdio::inherit())
               .stdout(Stdio::inherit())
               .stderr(Stdio::inherit());
        } else {
            cmd = Command::new(shell);
            cmd.arg("-c")
               .arg(&config.command)
               .stdin(Stdio::inherit())
               .stdout(Stdio::inherit())
               .stderr(Stdio::inherit());
        }

        // Execute command
        let status_result = cmd.status().with_context(|| {
            format!(
                "Failed to execute update command for {}: '{}'",
                config.name, config.command
            )
        });

        // Resume progress display after command completes
        output.resume_progress();

        let duration = start_time.elapsed();

        match status_result {
            Ok(status) => Ok(UpdateResult {
                name: config.name.clone(),
                success: status.success(),
                duration_ms: duration.as_millis(),
                exit_status: status.code(),
                error_message: None,
            }),
            Err(e) => Ok(UpdateResult {
                name: config.name.clone(),
                success: false,
                duration_ms: duration.as_millis(),
                exit_status: None,
                error_message: Some(e.to_string()),
            }),
        }
    }

    /// Runs updates for all enabled package managers in the provided configuration
    /// Updates can be executed sequentially or in parallel based on the parallel flag
    ///
    /// # Arguments
    /// * `configs` - A slice of package manager configurations
    /// * `output` - The output manager for displaying progress
    /// * `parallel` - Whether to run updates in parallel
    ///
    /// # Returns
    /// * `Result<Vec<UpdateResult>>` - Results of all update operations
    pub fn run_all_updates(
        &self,
        configs: &[PackageManagerConfig],
        output: &OutputManager,
        parallel: bool,
    ) -> Result<Vec<UpdateResult>> {
        use indicatif::{ProgressBar, ProgressStyle};
        use std::time::Duration;

        // Filter enabled package managers
        let enabled_configs: Vec<&PackageManagerConfig> =
            configs.iter().filter(|c| c.enabled).collect();

        let enabled_count = enabled_configs.len();
        
        // Initialize the multi progress display
        let multi_progress = output.init_multi_progress();

        // Create overall progress bar
        let overall_pb = multi_progress.add(ProgressBar::new(enabled_count as u64));
        overall_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} package managers updated ({eta})")
                .expect("Failed to set progress bar style")
                .progress_chars("#>-")
        );

        // Create shared results vector and progress counter
        let results = Arc::new(Mutex::new(Vec::with_capacity(enabled_count)));
        let completed = Arc::new(Mutex::new(0));

        // Define a closure that handles both sequential and parallel updates
        let process_update = |config: &PackageManagerConfig| {
            // Create individual progress bar for this package manager
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {prefix:.bold.dim} {wide_msg}")
                    .expect("Failed to set spinner style"),
            );
            let prefix = format!("[{}]", config.name);
            pb.set_prefix(prefix);
            pb.set_message("Updating...");
            pb.enable_steady_tick(Duration::from_millis(100));

            match self.run_update(config, output) {
                Ok(result) => {
                    // Report status based on result
                    if result.success {
                        let duration_secs = result.duration_ms as f64 / 1000.0;
                        pb.finish_with_message(format!(
                            "Updated successfully in {:.2}s",
                            duration_secs
                        ));
                        output.update_status(&result.name, "success");
                        debug!("{} updated in {:.2}s", result.name, duration_secs);
                    } else if let Some(err) = &result.error_message {
                        pb.finish_with_message(format!("Failed: {}", err));
                        output.update_status(&result.name, "failure");
                        debug!("{} update failed: {}", result.name, err);
                    } else if let Some(code) = result.exit_status {
                        pb.finish_with_message(format!("Failed with exit code {}", code));
                        output.update_status(&result.name, "failure");
                        debug!("{} update failed with exit code {}", result.name, code);
                    } else {
                        pb.finish_with_message("Failed with unknown error");
                        output.update_status(&result.name, "failure");
                        debug!("{} update failed with unknown error", result.name);
                    }
                    
                    // Store result in shared vector
                    let mut results_lock = results.lock().unwrap();
                    results_lock.push(result);
                    drop(results_lock);

                    // Update progress
                    let mut completed_lock = completed.lock().unwrap();
                    *completed_lock += 1;
                    drop(completed_lock);
                    overall_pb.inc(1);
                }
                Err(e) => {
                    pb.finish_with_message(format!("Error: {}", e));
                    output.update_status(&config.name, "failure");
                    debug!("Failed to run update for {}: {:#}", config.name, e);

                    // Store error result in shared vector
                    let mut results_lock = results.lock().unwrap();
                    results_lock.push(UpdateResult {
                        name: config.name.clone(),
                        success: false,
                        duration_ms: 0,
                        exit_status: None,
                        error_message: Some(e.to_string()),
                    });
                    drop(results_lock);

                    // Update progress
                    let mut completed_lock = completed.lock().unwrap();
                    *completed_lock += 1;
                    drop(completed_lock);
                    overall_pb.inc(1);
                }
            }
        };

        // Execute updates based on parallel flag
        if parallel {
            // Process updates in parallel using Rayon
            enabled_configs.par_iter().for_each(|config| {
                process_update(config);
            });
        } else {
            // Process updates sequentially
            enabled_configs.iter().for_each(|config| {
                process_update(config);
            });
        }

        // Finish the overall progress bar
        let completed_count = *completed.lock().unwrap();
        overall_pb.finish_with_message(format!(
            "Completed {} package manager updates",
            completed_count
        ));

        // Return the collected results
        let final_results = results.lock().unwrap().clone();
        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::OutputManager;
    use mockall::predicate::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    fn create_mock_executable(
        dir_path: &std::path::Path,
        name: &str,
        exit_code: i32,
    ) -> std::path::PathBuf {
        let path = dir_path.join(name);
        let script = format!("#!/bin/sh\nexit {}", exit_code);
        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn test_run_update_success() {
        let temp_dir = tempdir().unwrap();
        let success_script = create_mock_executable(temp_dir.path(), "success_script.sh", 0);

        let config = PackageManagerConfig {
            name: "TEST_PM".to_string(),
            enabled: true,
            command: success_script.to_string_lossy().to_string(),
            needs_sudo: false,
        };

        let runner = PackageManagerRunner;
        let output = OutputManager::new(false);
        let result = runner.run_update(&config, &output).unwrap();

        assert!(result.success);
        assert_eq!(result.name, "TEST_PM");
        assert_eq!(result.exit_status, Some(0));
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_run_update_failure() {
        let temp_dir = tempdir().unwrap();
        let failure_script = create_mock_executable(temp_dir.path(), "failure_script.sh", 1);

        let config = PackageManagerConfig {
            name: "TEST_PM".to_string(),
            enabled: true,
            command: failure_script.to_string_lossy().to_string(),
            needs_sudo: false,
        };

        let runner = PackageManagerRunner;
        let output = OutputManager::new(false);
        let result = runner.run_update(&config, &output).unwrap();

        assert!(!result.success);
        assert_eq!(result.name, "TEST_PM");
        assert_eq!(result.exit_status, Some(1));
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_run_update_empty_command() {
        let config = PackageManagerConfig {
            name: "EMPTY_PM".to_string(),
            enabled: true,
            command: "".to_string(),
            needs_sudo: false,
        };

        let runner = PackageManagerRunner;
        let output = OutputManager::new(false);
        let result = runner.run_update(&config, &output).unwrap();

        assert!(!result.success);
        assert_eq!(result.name, "EMPTY_PM");
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.unwrap(), "Empty command");
    }

    #[test]
    fn test_run_all_updates() {
        let temp_dir = tempdir().unwrap();
        let success_script = create_mock_executable(temp_dir.path(), "success_script.sh", 0);
        let failure_script = create_mock_executable(temp_dir.path(), "failure_script.sh", 1);

        let configs = vec![
            // Successful update
            PackageManagerConfig {
                name: "SUCCESS_PM".to_string(),
                enabled: true,
                command: success_script.to_string_lossy().to_string(),
                needs_sudo: false,
            },
            // Failed update
            PackageManagerConfig {
                name: "FAILURE_PM".to_string(),
                enabled: true,
                command: failure_script.to_string_lossy().to_string(),
                needs_sudo: false,
            },
            // Disabled PM (should be skipped)
            PackageManagerConfig {
                name: "DISABLED_PM".to_string(),
                enabled: false,
                command: "echo disabled".to_string(),
                needs_sudo: false,
            },
        ];

        let runner = PackageManagerRunner;
        let output_manager = OutputManager::new(false);
        let results = runner
            .run_all_updates(&configs, &output_manager, false)
            .unwrap();

        // Should have 2 results (disabled one is skipped)
        assert_eq!(results.len(), 2);

        // First should be success
        assert!(results[0].success);
        assert_eq!(results[0].name, "SUCCESS_PM");

        // Second should be failure
        assert!(!results[1].success);
        assert_eq!(results[1].name, "FAILURE_PM");
    }
}
