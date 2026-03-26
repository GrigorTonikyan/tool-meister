use clap::{Parser, Subcommand};

pub const DEFAULT_ALIAS_NAME: &str = "up-all";

/// Universal Package Manager Updater
///
/// A tool to manage and run updates for multiple package managers at once.
/// It handles detection, configuration, and execution of updates for various
/// system and language-specific package managers.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None
)]
pub struct Cli {
    /// Increase verbosity (show debug logs)
    /// Use multiple times for more detailed output (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

// TODO: Add new commands for v0.2.0+:
// - config: Interactive configuration editing
// - history: View update history
// - schedule: Set up scheduled updates
// - parallel: Enable/disable parallel updates

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run updates for all enabled package managers
    Run {
        /// Skip confirmation prompts and run immediately
        #[arg(short, long)]
        yes: bool,
        // TODO: Add flag for parallel execution
        // TODO: Add option to specify which package managers to update
    },

    /// Validate configuration file format and contents
    Validate,

    /// Create a timestamped backup of the configuration file
    Backup,

    /// Setup shell alias for easier access
    SetupAlias {
        /// Custom name for the alias (default: up-all)
        #[arg(default_value = DEFAULT_ALIAS_NAME)]
        name: String,
    },

    /// Detect available but unconfigured package managers
    Detect,
}
