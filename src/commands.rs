use crate::config::{Action, Config};
use crate::global_config::GlobalConfig;
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

pub async fn install(config: &Config, global_config: &GlobalConfig) -> Result<()> {
    println!("Installing {}...", config.repo.name);

    // Check if repo directory already exists in the tools directory
    let tools_dir = global_config.get_tools_directory();
    let repo_dir = tools_dir.join(&config.repo.name);

    if repo_dir.exists() {
        println!(
            "Repository {} already exists. Use 'update' command to update it.",
            config.repo.name
        );
        return Ok(());
    }

    // Create tools directory if it doesn't exist
    if !tools_dir.exists() {
        std::fs::create_dir_all(tools_dir).with_context(|| {
            format!("Failed to create tools directory: {}", tools_dir.display())
        })?;
    }

    execute_actions(
        config,
        &config.actions.installation,
        Some(tools_dir),
        None,
        false,
        false,
    )
    .await
}

pub async fn update(config: &Config, global_config: &GlobalConfig) -> Result<()> {
    println!("Updating {}...", config.repo.name);

    let tools_dir = global_config.get_tools_directory();
    let repo_dir = tools_dir.join(&config.repo.name);

    if !repo_dir.exists() {
        println!(
            "Repository {} does not exist. Use 'install' command first.",
            config.repo.name
        );
        return Ok(());
    }

    execute_actions(
        config,
        &config.actions.update,
        Some(&repo_dir),
        None,
        false,
        false,
    )
    .await
}

pub async fn build(config: &Config, global_config: &GlobalConfig) -> Result<()> {
    println!("Building {}...", config.repo.name);

    let tools_dir = global_config.get_tools_directory();
    let repo_dir = tools_dir.join(&config.repo.name);

    if !repo_dir.exists() {
        println!(
            "Repository {} does not exist. Use 'install' command first.",
            config.repo.name
        );
        return Ok(());
    }

    execute_actions(
        config,
        &config.actions.build,
        Some(&repo_dir),
        None,
        false,
        false,
    )
    .await
}

pub async fn run(
    config: &Config,
    args: &[String],
    force_spawn: bool,
    force_wait: bool,
    global_config: &GlobalConfig,
) -> Result<()> {
    println!("Running {}...", config.repo.name);

    let tools_dir = global_config.get_tools_directory();
    let repo_dir = tools_dir.join(&config.repo.name);

    if !repo_dir.exists() {
        println!(
            "Repository {} does not exist. Use 'install' command first.",
            config.repo.name
        );
        return Ok(());
    }

    execute_actions(
        config,
        &config.actions.run,
        Some(&repo_dir),
        Some(args),
        force_spawn,
        force_wait,
    )
    .await
}

async fn execute_actions(
    config: &Config,
    actions: &[Action],
    working_dir: Option<&std::path::Path>,
    extra_args: Option<&[String]>,
    force_spawn: bool,
    force_wait: bool,
) -> Result<()> {
    for action in actions {
        println!("Step {}: {}", action.seq_id, action.description);

        let interpolated_command = config.interpolate(&action.command);

        // Add extra arguments if provided
        let full_command = if let Some(args) = extra_args {
            if args.is_empty() {
                interpolated_command
            } else {
                format!("{} {}", interpolated_command, args.join(" "))
            }
        } else {
            interpolated_command
        };

        println!("Executing: {}", full_command);

        let mut cmd = if full_command.starts_with("./") {
            // Handle relative executable paths
            let mut command = Command::new("sh");
            command.arg("-c").arg(&full_command);
            command
        } else if full_command.contains(' ') {
            // Handle commands with arguments
            let mut command = Command::new("sh");
            command.arg("-c").arg(&full_command);
            command
        } else {
            // Handle simple commands
            Command::new(&full_command)
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        if action.spawn {
            // Determine spawn behavior based on flags and arguments
            let should_spawn = if force_wait {
                false // --wait flag overrides everything
            } else if force_spawn {
                true // --spawn flag forces spawning
            } else {
                // Smart default: spawn only if no args, or if args look like they won't produce output
                let has_args = extra_args.is_some_and(|args| !args.is_empty());
                if !has_args {
                    true // No args, likely GUI launch
                } else {
                    // Check if args suggest this is informational (will produce output and exit)
                    let config_info_args: Vec<&str> =
                        config.info_args.iter().map(|s| s.as_str()).collect();
                    let default_info_args = ["--help", "-h", "--version", "-V", "--list", "--show"];

                    // Use config info_args if provided, otherwise use defaults
                    let info_args = if config_info_args.is_empty() {
                        &default_info_args[..]
                    } else {
                        &config_info_args[..]
                    };

                    let has_info_arg = extra_args
                        .unwrap_or(&[])
                        .iter()
                        .any(|arg| info_args.contains(&arg.as_str()));
                    !has_info_arg // Spawn unless it's an info command
                }
            };

            if should_spawn {
                // Spawn mode: detach process
                cmd.stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .stdin(Stdio::null());

                let child = cmd
                    .spawn()
                    .with_context(|| format!("Failed to spawn command: {}", full_command))?;

                println!(
                    "✓ Spawned: {} (PID: {})\n",
                    action.description,
                    child.id().unwrap_or(0)
                );
            } else {
                // Wait mode: show output and wait for completion
                cmd.stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit());

                let status = cmd
                    .status()
                    .await
                    .with_context(|| format!("Failed to execute command: {}", full_command))?;

                if !status.success() {
                    anyhow::bail!("Command failed: {}", full_command);
                }

                println!("✓ Completed: {}\n", action.description);
            }
        } else {
            // For regular processes, inherit stdio and wait for completion
            cmd.stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit());

            let status = cmd
                .status()
                .await
                .with_context(|| format!("Failed to execute command: {}", full_command))?;

            if !status.success() {
                anyhow::bail!("Command failed: {}", full_command);
            }

            println!("✓ Completed: {}\n", action.description);
        }
    }

    Ok(())
}
