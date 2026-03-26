use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use std::env;
use tracing::{debug, error, info};

use crate::app::App;
use crate::config::{discover_modules, load_app_config};

mod action;
mod app;
mod cli;
mod components;
mod config;
mod error;
mod errors;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();

    // Set debug mode if requested
    if args.debug {
        env::set_var("RUST_LOG", "debug");
        debug!("Debug mode enabled");
    }

    // Handle CLI-only modes
    if args.list_modules {
        handle_list_modules().await?;
        return Ok(());
    }

    if let Some(module_name) = &args.module {
        handle_module_command(module_name, &args.command, &args.args).await?;
        return Ok(());
    }

    // Run the TUI application
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    let result = app.run().await;

    // Ensure terminal cleanup even if there's an error
    if let Err(e) = &result {
        error!("Application error: {}", e);
        crate::errors::restore_terminal();
    }

    result
}
async fn handle_list_modules() -> Result<()> {
    info!("Listing available modules...");

    // Load app configuration to get modules directory
    let app_config = load_app_config("config.jsonc")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e))?;
    let module_registry = discover_modules(&app_config.app_settings.modules_dir)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to discover modules: {}", e))?;

    println!("Available modules:");
    for module in module_registry.get_enabled_modules() {
        println!("  {} - {}", module.name, module.config.description);
        println!("    Version: {}", module.config.version);

        // List available commands
        if !module.commands().is_empty() {
            println!("    Commands:");
            for (cmd_name, cmd_def) in module.commands() {
                println!("      {} - {}", cmd_name, cmd_def.description);
            }
        }
        println!();
    }

    Ok(())
}

async fn handle_module_command(
    module_name: &str,
    command: &Option<String>,
    args: &[String],
) -> Result<()> {
    info!(
        "Executing module command: {} in module: {}",
        command.as_deref().unwrap_or("(no command)"),
        module_name
    );

    // Load app configuration and modules
    let app_config = load_app_config("config.jsonc")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e))?;
    let module_registry = discover_modules(&app_config.app_settings.modules_dir)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to discover modules: {}", e))?;

    // Find the requested module
    let module = module_registry
        .get_module(module_name)
        .ok_or_else(|| color_eyre::eyre::eyre!("Module '{}' not found", module_name))?;

    if !module.config.enabled {
        error!("Module '{}' is disabled", module_name);
        return Ok(());
    }

    // If no command specified, list available commands
    let Some(command_name) = command else {
        println!("Available commands for module '{}':", module_name);
        for (cmd_name, cmd_def) in module.commands() {
            println!("  {} - {}", cmd_name, cmd_def.description);
        }
        return Ok(());
    };

    // Find and execute the command
    let command_def = module.commands().get(command_name).ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "Command '{}' not found in module '{}'",
            command_name,
            module_name
        )
    })?;

    println!(
        "Executing command: {} - {}",
        command_name, command_def.description
    );

    // Check dependencies
    if let Some(deps) = &command_def.dependencies {
        println!("Required dependencies: {:?}", deps);
        // TODO: Implement dependency checking
    }

    // Execute the command
    // TODO: Implement actual command execution with shell
    println!("Command execution not yet implemented");
    println!("Would execute function: {}", command_def.function);
    if let Some(cmd_args) = &command_def.args {
        println!("With arguments: {:?}", cmd_args);
    }
    if !args.is_empty() {
        println!("User provided arguments: {:?}", args);
    }

    Ok(())
}
