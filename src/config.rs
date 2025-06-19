use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub repo: Repository,
    pub dependencies: Vec<Dependency>,
    pub actions: Actions,
    /// Arguments that should trigger wait-and-show-output behavior (instead of spawning)
    #[serde(default)]
    pub info_args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub default_branch: Branch,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Branch {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Actions {
    pub installation: Vec<Action>,
    pub update: Vec<Action>,
    pub build: Vec<Action>,
    pub run: Vec<Action>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Action {
    #[serde(rename = "seq-id")]
    pub seq_id: u32,
    pub name: Option<String>,
    pub command: String,
    pub description: String,
    #[serde(default)]
    pub spawn: bool,
}

impl Config {
    pub fn load(config_dir: &Path, tool_name: &str) -> Result<Self> {
        let config_path = config_dir.join(format!("{}.jsonc", tool_name));
        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(config_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        // Remove comments for basic JSONC support
        let json_content = Self::strip_comments(&content);

        let config: Config = serde_json::from_str(&json_content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    }

    /// Basic JSONC comment stripping (removes // comments)
    fn strip_comments(content: &str) -> String {
        content
            .lines()
            .map(|line| {
                if let Some(pos) = line.find("//") {
                    // Check if // is inside quotes
                    let before_comment = &line[..pos];
                    let quote_count = before_comment.matches('"').count();
                    if quote_count % 2 == 0 {
                        // Even number of quotes, so // is not inside quotes
                        before_comment.to_string()
                    } else {
                        // Odd number of quotes, so // is inside quotes
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn interpolate(&self, text: &str) -> String {
        text.replace("[[repo.url]]", &self.repo.url)
            .replace("[[repo.name]]", &self.repo.name)
    }
}
