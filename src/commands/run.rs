use crate::config::Config;
use crate::error::Result;
use crate::global_config::GlobalConfig;

pub async fn run_command(
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

    super::execute_actions(
        config,
        &config.actions.run,
        Some(&repo_dir),
        Some(args),
        force_spawn,
        force_wait,
    )
    .await
}
