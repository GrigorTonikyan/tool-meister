// filepath: /home/greg/dev/up-man/src/output.rs
use anyhow::Result;
use colored::*;
use indicatif::MultiProgress;
use log::{error, info, LevelFilter};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Manages output formatting and logging for the application
#[derive(Clone)]
pub struct OutputManager {
    verbose: bool,
    multi_progress: Arc<Mutex<Option<MultiProgress>>>,
    tui_mode: bool,
    status_lines: Arc<Mutex<Vec<String>>>,
}

impl OutputManager {
    /// Creates a new OutputManager with the specified verbosity level
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            multi_progress: Arc::new(Mutex::new(None)),
            tui_mode: true,
            status_lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Creates a new thread-safe OutputManager with the specified verbosity level
    pub fn new_shared(verbose: bool) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            multi_progress: Arc::new(Mutex::new(None)),
            tui_mode: true,
            status_lines: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Sets up logging with appropriate formatting and verbosity level
    ///
    /// # Returns
    /// * `Result<()>` - Success or failure of logging setup
    pub fn setup_logging(&self) -> Result<()> {
        let mut builder = env_logger::Builder::new();

        builder
            .format(|buf, record| {
                let style = match record.level() {
                    log::Level::Error => "❌ ".red().to_string(),
                    log::Level::Warn => "⚠️ ".yellow().to_string(),
                    log::Level::Info => "ℹ️ ".blue().to_string(),
                    log::Level::Debug => "🔍 ".dimmed().to_string(),
                    log::Level::Trace => "🔬 ".dimmed().to_string(),
                };

                writeln!(buf, "{}{}", style, record.args())
            })
            .filter(
                None,
                if self.verbose {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Info
                },
            );

        builder.init();
        Ok(())
    }

    /// Initialize a MultiProgress instance for TUI mode
    pub fn init_multi_progress(&self) -> MultiProgress {
        let multi = MultiProgress::new();
        let mut lock = self.multi_progress.lock().unwrap();
        *lock = Some(multi.clone());
        multi
    }

    /// Get the current MultiProgress instance or create a new one
    pub fn get_multi_progress(&self) -> MultiProgress {
        let mut lock = self.multi_progress.lock().unwrap();
        if let Some(ref multi) = *lock {
            multi.clone()
        } else {
            let multi = MultiProgress::new();
            *lock = Some(multi.clone());
            multi
        }
    }

    /// Suspend the progress bars temporarily (useful for password prompts)
    pub fn suspend_progress(&self) {
        if let Some(multi) = self.multi_progress.lock().unwrap().as_ref() {
            multi.suspend(|| {
                // Ensure cursor is visible and at end of current line
                print!("\r\x1B[?25h");
                io::stdout().flush().unwrap();
            });
        }
    }

    /// Resume the progress bars after suspension
    pub fn resume_progress(&self) {
        if self.multi_progress.lock().unwrap().is_some() {
            // No explicit resume function in indicatif, progress bars auto-resume on next update
            // Just ensure cursor is hidden for progress bars
            print!("\x1B[?25l");
            io::stdout().flush().unwrap();
        }
    }

    /// Display a password prompt that's clearly visible
    pub fn show_sudo_prompt(&self, package_name: &str) {
        self.suspend_progress();

        // Display the sudo prompt in a highly visible format
        eprintln!(
            "\n{} {}: {}",
            "🔐".yellow().bold(),
            "SUDO PASSWORD REQUIRED".yellow().bold(),
            package_name.cyan().bold()
        );

        // The actual password prompt will be shown by sudo itself
    }

    /// Logs a success message with green highlighting
    ///
    /// # Arguments
    /// * `msg` - The success message to display
    pub fn success(&self, msg: &str) {
        info!("{}", msg.green());
    }

    /// Updates status of an operation with appropriate formatting
    ///
    /// # Arguments
    /// * `name` - The name of the component being updated
    /// * `status` - The status to display ("started", "success", "failure", or custom)
    pub fn update_status(&self, name: &str, status: &str) {
        match status {
            "started" => {
                if !self.tui_mode {
                    info!("⏳ Updating {}...", name.cyan());
                }
            }
            "success" => {
                if !self.tui_mode {
                    info!("✅ {} update completed successfully.", name.green());
                }
            }
            "failure" => {
                if !self.tui_mode {
                    error!("❌ {} update failed.", name.red());
                }
            }
            _ => {
                if !self.tui_mode {
                    info!("{}: {}", name, status);
                }
            }
        }

        // Store status in status lines for summary
        if status == "success" || status == "failure" {
            let mut status_lines = self.status_lines.lock().unwrap();
            let status_msg = match status {
                "success" => format!("✅ {} update completed successfully", name),
                "failure" => format!("❌ {} update failed", name),
                _ => format!("{}: {}", name, status),
            };
            status_lines.push(status_msg);
        }
    }

    /// Displays a separator line for visual grouping
    ///
    /// # Arguments
    /// * `style` - The style of separator to display ("heavy", "light", or empty)
    pub fn separator(&self, style: &str) {
        match style {
            "heavy" => info!("{}", "========================================".blue()),
            "light" => info!("{}", "----------------------------------------".dimmed()),
            _ => info!(""),
        }
    }
}
