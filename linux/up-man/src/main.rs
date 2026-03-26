mod cli;
mod config;
mod output;
mod package_managers;
mod shell;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use std::process;

use cli::{Cli, Commands};
use config::manager::ConfigManager;
use output::OutputManager;
use package_managers::{detector::PackageManagerDetector, runner::PackageManagerRunner};
use shell::alias::AliasManager;

// TODO: Consider implementing a proper error enum for more specific error handling
// TODO: Add a logging module that tracks update history for future reporting

/// Main entry point for the up-man application
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Setup logging based on verbosity
    let verbose = cli.verbose > 0;
    let output = OutputManager::new(verbose);
    output.setup_logging()?;

    // Initialize config manager
    let config_manager = ConfigManager::new()?;

    // Process commands
    match &cli.command {
        Some(Commands::Run { yes }) => run_updates(&config_manager, &output, *yes),
        Some(Commands::Validate) => validate_config(&config_manager),
        Some(Commands::Backup) => backup_config(&config_manager),
        Some(Commands::SetupAlias { name }) => setup_alias(name),
        Some(Commands::Detect) => detect_package_managers(&config_manager),
        None => run_updates(&config_manager, &output, false), // Default to running updates
                                                              // TODO: Add new commands:
                                                              // - 'config': For interactive configuration editing
                                                              // - 'history': For viewing update history
    }
}

// TODO: Enhance with parallel execution support for independent package managers
fn run_updates(
    config_manager: &ConfigManager,
    output: &OutputManager,
    skip_confirm: bool,
) -> Result<()> {
    info!("Starting system update process");
    output.separator("heavy");

    // Ensure config exists
    config_manager.create_default_if_missing()?;

    // Load configuration
    let config = match config_manager.load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {:#}", e);
            process::exit(1);
        }
    };

    if config.package_managers.is_empty() {
        warn!("No package managers configured");
        return Ok(());
    }

    // Optionally ask for confirmation
    if !skip_confirm {
        let enabled_count = config
            .package_managers
            .iter()
            .filter(|pm| pm.enabled)
            .count();
        info!(
            "About to update {} enabled package manager(s)",
            enabled_count
        );

        if enabled_count > 0 {
            info!("Press Enter to continue or Ctrl+C to cancel...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }
    }

    let start_time = std::time::Instant::now();

    // Run updates
    let runner = PackageManagerRunner;
    // Check if parallel updates are enabled in config
    let use_parallel = config.settings.as_ref()
        .and_then(|s| s.parallel_updates)
        .unwrap_or(false);
    let results = runner.run_all_updates(&config.package_managers, output, use_parallel)?;

    // Display summary
    output.separator("heavy");
    let total_duration = start_time.elapsed().as_secs_f64();
    info!("📊 Update Summary (completed in {:.1}s):", total_duration);

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    let skipped_count = config.package_managers.len() - (success_count + failure_count);

    output.success(&format!("✅ Successful: {}", success_count));
    if failure_count > 0 {
        error!("❌ Failed: {}", failure_count);
    }
    if skipped_count > 0 {
        info!("⏭️ Skipped: {}", skipped_count);
    }

    // Show more details about failed updates if any
    if failure_count > 0 {
        output.separator("light");
        info!("Failed updates:");
        for result in results.iter().filter(|r| !r.success) {
            if let Some(err) = &result.error_message {
                error!("{}: {}", result.name, err);
            } else if let Some(code) = result.exit_status {
                error!("{}: exited with code {}", result.name, code);
            } else {
                error!("{}: unknown error", result.name);
            }
        }
        process::exit(1);
    }

    Ok(())
}

fn validate_config(config_manager: &ConfigManager) -> Result<()> {
    // First ensure config exists
    if config_manager.create_default_if_missing()? {
        info!(
            "Created default configuration file at {}",
            config_manager.get_config_path().display()
        );
    }

    // Validate configuration
    match config_manager.validate() {
        Ok(true) => {
            info!("Configuration validation successful!");
            Ok(())
        }
        Ok(false) => {
            error!("Configuration validation failed");
            process::exit(1);
        }
        Err(e) => {
            error!("Error during validation: {:#}", e);
            process::exit(1);
        }
    }
}

fn backup_config(config_manager: &ConfigManager) -> Result<()> {
    match config_manager.backup() {
        Ok(path) => {
            info!("Backup created at: {}", path.display());
            Ok(())
        }
        Err(e) => {
            error!("Failed to create backup: {:#}", e);
            process::exit(1);
        }
    }
}

fn setup_alias(name: &str) -> Result<()> {
    // Get path to the current executable
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    match AliasManager::setup_alias(name, &current_exe) {
        Ok(()) => {
            info!("Alias '{}' set up successfully", name);
            Ok(())
        }
        Err(e) => {
            error!("Failed to set up alias: {:#}", e);
            process::exit(1);
        }
    }
}

fn detect_package_managers(config_manager: &ConfigManager) -> Result<()> {
    info!("Detecting available package managers...");

    // First ensure config exists
    config_manager.create_default_if_missing()?;

    // Load configuration if exists
    let config = match config_manager.load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {:#}", e);
            process::exit(1);
        }
    };

    // Detect unconfigured package managers
    let detector = PackageManagerDetector;
    let detected = detector.detect_unconfigured(&config.package_managers)?;

    if detected.is_empty() {
        info!("No new package managers detected");
        return Ok(());
    }

    // Display detected package managers
    info!(
        "Detected {} package manager(s) that are not in your configuration:",
        detected.len()
    );

    for (i, pm) in detected.iter().enumerate() {
        info!("{}. {}", i + 1, pm.name);
    }

    info!(
        "\nTo add these to your configuration at {}, you can:",
        config_manager.get_config_path().display()
    );
    info!("1. Manually edit the configuration file");

    for pm in &detected {
        info!("\n--- {} Configuration Snippet ---", pm.name);
        info!("{}", pm.default_toml_snippet);
    }

    Ok(())
}
