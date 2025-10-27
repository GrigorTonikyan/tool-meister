use crate::config::Config;
use crate::error::Result;
use crate::global_config::GlobalConfig;

pub async fn update_command(config: &Config, global_config: &GlobalConfig) -> Result<()> {
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

    super::execute_actions(
        config,
        &config.actions.update,
        Some(&repo_dir),
        None,
        false,
        false,
    )
    .await
}
