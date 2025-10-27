use crate::config::Config;
use crate::error::Result;
use crate::global_config::GlobalConfig;
use anyhow::Context;

pub async fn install_command(
    config: &Config,
    global_config: &GlobalConfig,
) -> Result<()> {
    println!("Installing {}...", config.repo.name);

    // Check if repo directory already exists in the tools directory
    let tools_dir = global_config.get_tools_directory();
    let repo_dir = tools_dir.join(&config.repo.name);

    if !repo_dir.exists() {
        std::fs::create_dir_all(&repo_dir).with_context(|| {
            format!("Failed to create repo directory: {}", repo_dir.display())
        })?;
    }

    if repo_dir.exists() {
        println!(
            "Repository {} already exists. Proceeding with installation actions.",
            config.repo.name
        );
    }

    super::execute_actions(
        config,
        &config.actions.installation,
        Some(tools_dir),
        None,
        false,
        false,
    )
    .await
}
