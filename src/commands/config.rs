use crate::error::Result;
use crate::global_config::GlobalConfig;
use serde_json;

pub async fn config_command(show: bool, reset: bool, _global_config: &GlobalConfig) -> Result<()> {
    let config_path = GlobalConfig::get_config_path();

    if reset {
        let default_config = GlobalConfig::default();
        default_config.save()?;
        println!("✅ App configuration reset to defaults");
    } else if show {
        let global_config = GlobalConfig::load()?;
        let config_json = serde_json::to_string_pretty(&global_config).map_err(|e| crate::error::Error::JsonDe(e))?;
        println!("Current app configuration:");
        println!("{}", config_json);
        println!("location: {}", config_path.display())
    } else {
        println!("App configuration file: {}", config_path.display());
        if !config_path.exists() {
            let global_config = GlobalConfig::load()?;
            global_config.save()?;
            println!("✅ Created default app configuration");
        }
    }

    Ok(())
}
