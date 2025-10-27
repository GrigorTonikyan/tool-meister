pub mod install;
pub mod update;
pub mod build;
pub mod run;
pub mod config;
use crate::config::{Action, Config};
use crate::error::Result;
use anyhow::Context;
use std::process::Stdio;
use tokio::process::Command;









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
                

                let output = cmd
                    .output()
                    .await
                    .with_context(|| format!("Failed to execute command: {}", full_command))?;

                if !output.status.success() {
                    return Err(crate::error::Error::Command(format!("Command failed:здравствуйте {}

-- stdout --
{}
-- stderr --
{}", full_command, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))));
                }

                println!("✓ Completed: {}
", action.description);
            }
        } else {
            

            let output = cmd
                .output()
                .await
                .with_context(|| format!("Failed to execute command: {}", full_command))?;

            if !output.status.success() {
                return Err(crate::error::Error::Command(format!("Command failed:здравствуйте {}\n\n-- stdout --\n{}\n-- stderr --\n{}", full_command, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))));
            }

            println!("✓ Completed: {}
", action.description);
        }
    }

    Ok(())
}
