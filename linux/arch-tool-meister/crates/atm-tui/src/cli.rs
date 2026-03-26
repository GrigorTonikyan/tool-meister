use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Execute a module command directly
    #[arg(long, value_name = "MODULE")]
    pub module: Option<String>,

    /// Command to execute (used with --module)
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,

    /// Command arguments (used with --module and command)
    #[arg(value_name = "ARGS")]
    pub args: Vec<String>,

    /// List all available modules
    #[arg(long)]
    pub list_modules: bool,

    /// Enable debug mode
    #[arg(long)]
    pub debug: bool,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

A modular Rust TUI application for managing Arch Linux tools and configurations."
    )
}
