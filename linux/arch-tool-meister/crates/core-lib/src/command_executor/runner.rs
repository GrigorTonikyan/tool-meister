//! Command Runner Module
//!
//! This module implements the actual command execution with async support,
//! timeout handling, output streaming, and privilege escalation management.

use crate::error::{CoreError, CoreResult};
use shared_types::module::ModuleFunction;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

use super::{ExecutionContext, ExecutionOptions, ExecutionResult};

/// Command runner for executing system commands
#[derive(Debug)]
pub struct CommandRunner {
    /// Current working directory override
    working_dir: Option<String>,
    /// Default environment variables
    default_env: HashMap<String, String>,
}

impl CommandRunner {
    /// Create a new command runner
    pub fn new() -> Self {
        Self {
            working_dir: None,
            default_env: HashMap::new(),
        }
    }

    /// Create a new command runner with working directory
    pub fn with_working_dir(working_dir: String) -> Self {
        Self {
            working_dir: Some(working_dir),
            default_env: HashMap::new(),
        }
    }

    /// Execute a module function
    #[instrument(skip(self, function, context, options))]
    pub async fn run_function(
        &self,
        function: &ModuleFunction,
        context: &ExecutionContext,
        options: &ExecutionOptions,
    ) -> CoreResult<ExecutionResult> {
        debug!("Running function '{}'", context.function_name);

        // Parse the function code to extract command and arguments
        let (command, args) = self.parse_function_code(&function.code, &context.args)?;

        self.run_command(&command, &args, context, options).await
    }

    /// Execute a direct command
    #[instrument(skip(self, command, args, context, options))]
    pub async fn run_command(
        &self,
        command: &str,
        args: &[String],
        context: &ExecutionContext,
        options: &ExecutionOptions,
    ) -> CoreResult<ExecutionResult> {
        let start_time = Instant::now();

        debug!("Executing command: {} {:?}", command, args);

        // Build the command
        let mut cmd = if options.allow_sudo && self.needs_sudo(command) {
            self.build_sudo_command(command, args)?
        } else {
            self.build_regular_command(command, args)?
        };

        // Set working directory
        if let Some(ref wd) = context.working_dir.as_ref().or(self.working_dir.as_ref()) {
            cmd.current_dir(wd);
        }

        // Set environment variables
        let mut env_vars = self.default_env.clone();
        env_vars.extend(context.env_vars.clone());
        env_vars.extend(options.env_vars.clone());

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Configure stdio based on streaming options
        if options.stream_output {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to spawn command '{}': {}",
                command, e
            )))
        })?;

        // Handle execution with timeout
        let execution_future = async {
            if options.stream_output {
                self.handle_streaming_execution(&mut child).await
            } else {
                self.handle_regular_execution(&mut child).await
            }
        };

        let result = match timeout(context.timeout, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred, kill the process
                warn!(
                    "Command '{}' timed out after {:?}",
                    command, context.timeout
                );
                let _ = child.kill().await;
                let _ = child.wait().await;

                Ok(ExecutionResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Command timed out after {:?}", context.timeout),
                    duration: start_time.elapsed(),
                    timed_out: true,
                })
            }
        };

        result
    }

    /// Set default working directory
    pub fn set_working_dir(&mut self, working_dir: Option<String>) {
        self.working_dir = working_dir;
    }

    /// Set default environment variables
    pub fn set_default_env(&mut self, env: HashMap<String, String>) {
        self.default_env = env;
    }

    /// Add default environment variable
    pub fn add_default_env(&mut self, key: String, value: String) {
        self.default_env.insert(key, value);
    }

    // Private helper methods

    /// Parse function code to extract command and arguments
    fn parse_function_code(
        &self,
        code: &str,
        args: &[String],
    ) -> CoreResult<(String, Vec<String>)> {
        // Simple parsing - in a real implementation, this could be more sophisticated
        // For now, we'll assume the code is a simple shell command that can be templated

        let mut parsed_code = code.to_string();

        // Replace argument placeholders like {0}, {1}, etc.
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            parsed_code = parsed_code.replace(&placeholder, arg);
        }

        // Replace $@ with all arguments
        if parsed_code.contains("$@") {
            parsed_code = parsed_code.replace("$@", &args.join(" "));
        }

        // Replace $1, $2, etc. with positional arguments
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            parsed_code = parsed_code.replace(&placeholder, arg);
        }

        // Split the code into command and arguments
        let parts: Vec<&str> = parsed_code.split_whitespace().collect();
        if parts.is_empty() {
            return Err(CoreError::from(shared_types::SharedError::validation(
                "Empty command code".to_string(),
            )));
        }

        let command = parts[0].to_string();
        let command_args = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok((command, command_args))
    }

    /// Check if command needs sudo privileges
    fn needs_sudo(&self, command: &str) -> bool {
        // Commands that typically require sudo
        let sudo_commands = [
            "pacman",
            "systemctl",
            "mount",
            "umount",
            "modprobe",
            "iptables",
            "ip",
            "netctl",
            "systemd-networkd",
            "dhcpcd",
            "wpa_supplicant",
            "hostapd",
        ];

        sudo_commands.iter().any(|&cmd| command.starts_with(cmd))
    }

    /// Build sudo command
    fn build_sudo_command(&self, command: &str, args: &[String]) -> CoreResult<TokioCommand> {
        let mut cmd = TokioCommand::new("sudo");
        cmd.arg(command);
        cmd.args(args);
        Ok(cmd)
    }

    /// Build regular command
    fn build_regular_command(&self, command: &str, args: &[String]) -> CoreResult<TokioCommand> {
        let mut cmd = TokioCommand::new(command);
        cmd.args(args);
        Ok(cmd)
    }

    /// Handle regular (non-streaming) execution
    async fn handle_regular_execution(
        &self,
        child: &mut tokio::process::Child,
    ) -> CoreResult<ExecutionResult> {
        let start_time = Instant::now();

        // Take stdout and stderr if they were piped
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Wait for the process to complete
        let status = child.wait().await.map_err(|e| {
            CoreError::from(shared_types::SharedError::io(format!(
                "Failed to wait for command completion: {}",
                e
            )))
        })?;

        // Read remaining output if we captured streams
        let mut stdout_str = String::new();
        if let Some(mut stream) = stdout {
            use tokio::io::AsyncReadExt;
            let mut buffer = Vec::new();
            let _ = stream.read_to_end(&mut buffer).await;
            stdout_str = String::from_utf8_lossy(&buffer).to_string();
        }

        let mut stderr_str = String::new();
        if let Some(mut stream) = stderr {
            use tokio::io::AsyncReadExt;
            let mut buffer = Vec::new();
            let _ = stream.read_to_end(&mut buffer).await;
            stderr_str = String::from_utf8_lossy(&buffer).to_string();
        }

        let duration = start_time.elapsed();
        let exit_code = status.code().unwrap_or(-1);

        debug!(
            "Command completed with exit code {} in {:?}",
            exit_code, duration
        );

        Ok(ExecutionResult {
            exit_code,
            stdout: stdout_str,
            stderr: stderr_str,
            duration,
            timed_out: false,
        })
    }

    /// Handle streaming execution with real-time output
    async fn handle_streaming_execution(
        &self,
        child: &mut tokio::process::Child,
    ) -> CoreResult<ExecutionResult> {
        let start_time = Instant::now();

        let stdout = child.stdout.take().ok_or_else(|| {
            CoreError::from(shared_types::SharedError::internal(
                "Failed to capture stdout".to_string(),
            ))
        })?;

        let stderr = child.stderr.take().ok_or_else(|| {
            CoreError::from(shared_types::SharedError::internal(
                "Failed to capture stderr".to_string(),
            ))
        })?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        // Read output streams concurrently
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            info!("STDOUT: {}", line);
                            stdout_lines.push(line);
                        }
                        Ok(None) => {
                            // stdout closed
                        }
                        Err(e) => {
                            error!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            warn!("STDERR: {}", line);
                            stderr_lines.push(line);
                        }
                        Ok(None) => {
                            // stderr closed
                        }
                        Err(e) => {
                            error!("Error reading stderr: {}", e);
                            break;
                        }
                    }
                }
                status = child.wait() => {
                    match status {
                        Ok(status) => {
                            let duration = start_time.elapsed();
                            let exit_code = status.code().unwrap_or(-1);

                            let stdout = stdout_lines.join("\n");
                            let stderr = stderr_lines.join("\n");

                            debug!(
                                "Streaming command completed with exit code {} in {:?}",
                                exit_code, duration
                            );

                            return Ok(ExecutionResult {
                                exit_code,
                                stdout,
                                stderr,
                                duration,
                                timed_out: false,
                            });
                        }
                        Err(e) => {
                            return Err(CoreError::from(shared_types::SharedError::io(
                                format!("Failed to wait for command: {}", e),
                            )));
                        }
                    }
                }
            }
        }

        // This shouldn't be reached, but just in case
        let duration = start_time.elapsed();
        Ok(ExecutionResult {
            exit_code: -1,
            stdout: stdout_lines.join("\n"),
            stderr: stderr_lines.join("\n"),
            duration,
            timed_out: false,
        })
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_simple_command_execution() {
        let runner = CommandRunner::new();
        let context = ExecutionContext {
            module_name: "test".to_string(),
            function_name: "test".to_string(),
            args: vec![],
            dry_run: false,
            timeout: Duration::from_secs(10),
            working_dir: None,
            env_vars: HashMap::new(),
        };
        let options = ExecutionOptions::default();

        let result = runner
            .run_command("echo", &["hello".to_string()], &context, &options)
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert_eq!(exec_result.exit_code, 0);
        assert!(exec_result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_command_timeout() {
        let runner = CommandRunner::new();
        let context = ExecutionContext {
            module_name: "test".to_string(),
            function_name: "test".to_string(),
            args: vec![],
            dry_run: false,
            timeout: Duration::from_millis(100), // Very short timeout
            working_dir: None,
            env_vars: HashMap::new(),
        };
        let options = ExecutionOptions::default();

        let result = runner
            .run_command("sleep", &["1".to_string()], &context, &options)
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.timed_out);
    }

    #[test]
    fn test_function_code_parsing() {
        let runner = CommandRunner::new();

        // Test simple substitution
        let (cmd, args) = runner
            .parse_function_code("echo {0}", &["hello".to_string()])
            .unwrap();
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello"]);

        // Test $@ substitution
        let (cmd, args) = runner
            .parse_function_code("echo $@", &["hello".to_string(), "world".to_string()])
            .unwrap();
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_sudo_detection() {
        let runner = CommandRunner::new();

        assert!(runner.needs_sudo("pacman"));
        assert!(runner.needs_sudo("systemctl"));
        assert!(!runner.needs_sudo("echo"));
        assert!(!runner.needs_sudo("ls"));
    }
}
