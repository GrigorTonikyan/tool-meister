use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{
    action::{Action, StatusType, VersionInfo},
    components::{fps::FpsCounter, home::Home, menu::Menu, Component},
    config::{
        discover_modules, load_app_config, load_main_menu_config, AnimationConfig, AppConfig,
        AppSettings, MainMenuConfig, MenuOption, MenuOptionType, ModuleRegistry,
    },
    tui::{Event, Tui},
};

pub struct App {
    app_config: AppConfig,
    main_menu_config: MainMenuConfig,
    module_registry: ModuleRegistry,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    menu_component: Menu,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    current_menu_stack: Vec<MenuState>,
    selected_index: usize,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    loading_state: LoadingState,
    version_info: HashMap<String, VersionInfo>,
    scroll_state: ScrollState,
    status_state: StatusState,
    awaiting_confirmation: bool,
    confirmation_message: String,
    confirmation_result_type: crate::action::StatusType,
}

#[derive(Debug, Clone, Default)]
pub struct LoadingState {
    pub is_loading: bool,
    pub message: String,
    pub elapsed_time: Option<std::time::Duration>,
    /// Track operation start time for elapsed time calculation
    pub started_at: Option<std::time::Instant>,
    pub operation_type: String,
    /// Animation frame counter for spinners
    pub animation_frame: usize,
    /// Progress percentage for operations that support it
    pub progress: Option<f32>,
    /// Start time for tracking elapsed time
    pub start_time: Option<std::time::Instant>,
}

#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    pub offset: usize,
    pub content_length: usize,
    pub viewport_height: usize,
}

impl ScrollState {
    pub fn can_scroll_up(&self) -> bool {
        self.offset > 0
    }

    pub fn can_scroll_down(&self) -> bool {
        self.offset + self.viewport_height < self.content_length
    }

    pub fn scroll_up(&mut self) {
        if self.can_scroll_up() {
            self.offset = self.offset.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.can_scroll_down() {
            self.offset += 1;
        }
    }

    pub fn update_content(&mut self, content_length: usize, viewport_height: usize) {
        self.content_length = content_length;
        self.viewport_height = viewport_height;

        // Adjust offset if it's now out of bounds
        if self.content_length <= self.viewport_height {
            self.offset = 0;
        } else {
            let max_offset = self.content_length.saturating_sub(self.viewport_height);
            if self.offset > max_offset {
                self.offset = max_offset;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusState {
    pub message: String,
    pub status_type: crate::action::StatusType,
    pub visible: bool,
    pub timestamp: Option<Instant>,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            message: String::new(),
            status_type: crate::action::StatusType::Info,
            visible: false,
            timestamp: None,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
    ModuleMenu,
    CommandExecution,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    Main,
    Module(String), // Module name
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        // Load configuration files with graceful fallback
        let app_config = load_app_config("config.jsonc")
            .or_else(|_| {
                eprintln!("⚠️  Failed to load config.jsonc, attempting fallback...");
                load_app_config("atm-rust-tui/config.jsonc")
            })
            .or_else(|_| {
                eprintln!("⚠️  Creating minimal default configuration...");
                Ok(Self::create_fallback_app_config())
            })
            .map_err(|e: color_eyre::Report| {
                color_eyre::eyre::eyre!("Critical: Cannot create any configuration: {}", e)
            })?;

        let main_menu_config = load_main_menu_config("main_menu.jsonc")
            .or_else(|_| {
                eprintln!("⚠️  Failed to load main_menu.jsonc, attempting fallback...");
                load_main_menu_config("atm-rust-tui/main_menu.jsonc")
            })
            .or_else(|_| {
                eprintln!("⚠️  Creating minimal default main menu...");
                Ok(Self::create_fallback_main_menu_config())
            })
            .map_err(|e: color_eyre::Report| {
                color_eyre::eyre::eyre!("Critical: Cannot create any menu configuration: {}", e)
            })?;

        let module_registry = discover_modules(&app_config.app_settings.modules_dir)
            .or_else(|_| {
                eprintln!(
                    "⚠️  Failed to discover modules in '{}', trying alternate paths...",
                    app_config.app_settings.modules_dir
                );
                discover_modules("atm-rust-tui/modules")
            })
            .or_else(|_| {
                eprintln!("⚠️  Loading without modules (module-free mode)...");
                Ok(ModuleRegistry::new())
            })
            .map_err(|e: color_eyre::Report| {
                color_eyre::eyre::eyre!("Critical: Cannot initialize module system: {}", e)
            })?;

        if module_registry.modules.is_empty() {
            eprintln!("ℹ️  No modules loaded. Some functionality may be limited.");
        } else {
            eprintln!(
                "✅ Loaded {} modules successfully",
                module_registry.modules.len()
            );
        }

        Ok(Self {
            app_config,
            main_menu_config,
            module_registry,
            tick_rate,
            frame_rate,
            components: vec![Box::new(Home::new()), Box::new(FpsCounter::default())],
            menu_component: Menu::new(),
            should_quit: false,
            should_suspend: false,
            mode: Mode::Home,
            current_menu_stack: vec![MenuState::Main],
            selected_index: 0,
            action_tx,
            action_rx,
            loading_state: LoadingState::default(),
            version_info: HashMap::new(),
            scroll_state: ScrollState::default(),
            status_state: StatusState::default(),
            awaiting_confirmation: false,
            confirmation_message: String::new(),
            confirmation_result_type: StatusType::Info,
        })
    }

    /// Create a minimal fallback configuration when the main config is unavailable
    fn create_fallback_app_config() -> AppConfig {
        use std::collections::HashMap;

        AppConfig {
            app_settings: AppSettings {
                app_name: "Arch Tool Meister".to_string(),
                version: "1.0.0".to_string(),
                modules_dir: "modules".to_string(),
                download_dir: Some("/tmp/atm-downloads".to_string()),
                install_prefix: Some("/usr/local".to_string()),
                animation: Some(AnimationConfig {
                    steps: 4,
                    delay_ms: 200,
                }),
            },
            vscode_config: None,
            menu_paths: None,
            aur_helpers: Some(HashMap::new()),
        }
    }

    /// Create a minimal fallback main menu when the config is unavailable
    fn create_fallback_main_menu_config() -> MainMenuConfig {
        MainMenuConfig {
            title: "Arch Tool Meister - Safe Mode".to_string(),
            dynamic_menu: Some(false),
            options: vec![
                MenuOption {
                    text: "System Information".to_string(),
                    option_type: MenuOptionType::ScriptFunction,
                    function_name: Some("display_system_info".to_string()),
                    module_name: None,
                },
                MenuOption {
                    text: "Exit".to_string(),
                    option_type: MenuOptionType::Exit,
                    function_name: None,
                    module_name: None,
                },
            ],
        }
    }

    /// Attempt to recover from critical errors
    pub fn attempt_recovery(&mut self, error_type: &str) -> Result<bool> {
        match error_type {
            "config" => {
                eprintln!("🔄 Attempting configuration recovery...");

                // Try to reload configurations
                if let Ok(new_config) = load_app_config("config.jsonc") {
                    self.app_config = new_config;
                    eprintln!("✅ Configuration recovered successfully");
                    return Ok(true);
                }

                // Fall back to minimal config
                self.app_config = Self::create_fallback_app_config();
                eprintln!("⚠️  Using minimal fallback configuration");
                Ok(true)
            }
            "modules" => {
                eprintln!("🔄 Attempting module recovery...");

                // Try to reload modules
                if let Ok(new_registry) =
                    discover_modules(&self.app_config.app_settings.modules_dir)
                {
                    if !new_registry.modules.is_empty() {
                        self.module_registry = new_registry;
                        eprintln!("✅ Modules recovered successfully");
                        return Ok(true);
                    }
                }

                // Continue with empty registry
                self.module_registry = ModuleRegistry::new();
                eprintln!("⚠️  Running in module-free mode");
                Ok(true)
            }
            "tui" => {
                eprintln!("🔄 Attempting TUI recovery...");
                // For TUI errors, we can't really recover in the same session
                // Log the error and suggest restart
                eprintln!("❌ TUI recovery requires application restart");
                eprintln!("💡 Try:");
                eprintln!("   1. Check terminal compatibility");
                eprintln!("   2. Resize terminal window");
                eprintln!("   3. Restart the application");
                Ok(false)
            }
            _ => {
                eprintln!("❓ Unknown error type '{}' - cannot recover", error_type);
                Ok(false)
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);

        // Ensure cleanup on any exit
        let cleanup = || {
            // Call our cleanup function to ensure terminal is restored
            crate::errors::restore_terminal();
        };

        let result = self.run_tui(&mut tui).await;

        // Ensure TUI cleanup
        if let Err(e) = tui.exit() {
            error!("Failed to exit TUI cleanly: {}", e);
            cleanup();
        }

        result
    }
    async fn run_tui(&mut self, tui: &mut Tui) -> Result<()> {
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        self.menu_component
            .register_action_handler(self.action_tx.clone())?;

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(tui).await?;
            self.handle_actions(tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        // If awaiting confirmation, only handle Enter key
        if self.awaiting_confirmation {
            match key.code {
                KeyCode::Enter => {
                    action_tx.send(Action::ConfirmationReceived)?;
                }
                KeyCode::Char('q') => {
                    action_tx.send(Action::Quit)?;
                }
                _ => {
                    // Ignore other keys when awaiting confirmation
                }
            }
            return Ok(());
        }

        // Handle basic navigation keys
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                let max_index = self.get_current_menu_options().len().saturating_sub(1);
                if self.selected_index < max_index {
                    self.selected_index += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll_state.scroll_up();
            }
            KeyCode::PageDown => {
                self.scroll_state.scroll_down();
            }
            KeyCode::Enter => {
                self.handle_menu_selection()?;
            }
            KeyCode::Char('q') => {
                action_tx.send(Action::Quit)?;
            }
            KeyCode::Char('0') => {
                self.handle_back_navigation()?;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let index = (digit as usize).saturating_sub(1);
                    if index < self.get_current_menu_options().len() {
                        self.selected_index = index;
                        self.handle_menu_selection()?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn get_current_menu_options(&self) -> Vec<String> {
        match self.current_menu_stack.last() {
            Some(MenuState::Main) => {
                let mut options = Vec::new();

                // Add main menu options
                for option in &self.main_menu_config.options {
                    options.push(option.text.clone());
                }

                // Add enabled modules if dynamic menu is enabled
                if self.main_menu_config.dynamic_menu.unwrap_or(false) {
                    for module in self.module_registry.get_enabled_modules() {
                        if let Some(main_menu_entry) = &module.config.menu.main_menu_entry {
                            options.push(main_menu_entry.clone());
                        } else {
                            options.push(module.config.menu.title.clone());
                        }
                    }
                }

                options
            }
            Some(MenuState::Module(module_name)) => {
                if let Some(module) = self.module_registry.get_module(module_name) {
                    module
                        .menu()
                        .options
                        .iter()
                        .map(|o| o.text.clone())
                        .collect()
                } else {
                    Vec::new()
                }
            }
            None => Vec::new(),
        }
    }

    fn handle_menu_selection(&mut self) -> Result<()> {
        let action_tx = self.action_tx.clone();

        // Clone the current menu state to avoid borrowing issues
        let current_state = self.current_menu_stack.last().cloned();

        match current_state {
            Some(MenuState::Main) => {
                let main_options_count = self.main_menu_config.options.len();

                if self.selected_index < main_options_count {
                    // Handle main menu option
                    let option = self.main_menu_config.options[self.selected_index].clone();
                    match option.option_type {
                        crate::config::MenuOptionType::ScriptFunction => {
                            // Execute built-in function
                            if let Some(function_name) = &option.function_name {
                                self.execute_builtin_function(function_name)?;
                            }
                        }
                        crate::config::MenuOptionType::Exit => {
                            action_tx.send(Action::Quit)?;
                        }
                        _ => {}
                    }
                } else {
                    // Handle module selection
                    let module_index = self.selected_index - main_options_count;
                    let enabled_modules: Vec<_> = self.module_registry.get_enabled_modules();

                    if let Some(module) = enabled_modules.get(module_index) {
                        self.current_menu_stack
                            .push(MenuState::Module(module.name.clone()));
                        self.selected_index = 0;
                        self.mode = Mode::ModuleMenu;

                        // Trigger version detection for VSCode module
                        if module.name == "vscode" {
                            self.detect_vscode_versions()?;
                        }
                    }
                }
            }
            Some(MenuState::Module(module_name)) => {
                if let Some(module) = self.module_registry.get_module(&module_name) {
                    let option = module.menu().options[self.selected_index].clone();
                    match option.option_type {
                        crate::config::MenuOptionType::ScriptFunction => {
                            if let Some(function_name) = &option.function_name {
                                self.execute_module_function(&module_name, function_name)?;
                            }
                        }
                        crate::config::MenuOptionType::Return => {
                            self.handle_back_navigation()?;
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        }
        Ok(())
    }

    fn handle_back_navigation(&mut self) -> Result<()> {
        if self.current_menu_stack.len() > 1 {
            self.current_menu_stack.pop();
            self.selected_index = 0;

            match self.current_menu_stack.last() {
                Some(MenuState::Main) => {
                    self.mode = Mode::Home;
                }
                Some(MenuState::Module(_)) => {
                    self.mode = Mode::ModuleMenu;
                }
                None => {
                    self.mode = Mode::Home;
                }
            }
        } else {
            // We're at the main menu, quit
            self.action_tx.send(Action::Quit)?;
        }
        Ok(())
    }

    fn execute_builtin_function(&mut self, function_name: &str) -> Result<()> {
        match function_name {
            "run_uname_a" => {
                self.execute_command("uname", &["-a"])?;
            }
            "run_pwd" => {
                self.execute_command("pwd", &[])?;
            }
            "run_ls_lh" => {
                self.execute_command("ls", &["-lh"])?;
            }
            _ => {
                info!("Unknown builtin function: {}", function_name);
            }
        }
        Ok(())
    }

    fn execute_module_function(&mut self, module_name: &str, function_name: &str) -> Result<()> {
        // First, get the command and function code without holding references
        let (command_clone, function_code) = {
            if let Some(module) = self.module_registry.get_module(module_name) {
                if let Some(command) = module.commands().get(function_name) {
                    let function_code = if let Some(functions) = module.functions() {
                        functions.get(&command.function).map(|f| f.code.clone())
                    } else {
                        None
                    };
                    (Some(command.clone()), function_code)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        };

        if let Some(command) = command_clone {
            info!(
                "Executing module function: {} in module: {}",
                function_name, module_name
            );

            // Check dependencies first
            if let Some(deps) = &command.dependencies {
                for dep in deps {
                    info!("Checking dependency: {}", dep);
                    if !self.check_dependency(dep)? {
                        error!("Missing dependency: {}", dep);
                        // Send error action instead of returning immediately
                        let _ = self
                            .action_tx
                            .send(Action::Error(format!("Missing dependency: {}", dep)));
                        return Ok(());
                    }
                }
            }

            // Execute the function code if available
            if let Some(code) = function_code {
                self.execute_shell_script(&code)?;
            } else {
                info!("No function implementation found for: {}", function_name);
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> Result<()> {
        use tokio::process::Command;

        let action_tx = self.action_tx.clone();
        let command = command.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        // Start loading animation
        let loading_message = format!("Executing: {} {}", command, args.join(" "));
        action_tx.send(Action::CommandStarted {
            command: command.clone(),
            description: loading_message.clone(),
        })?;
        action_tx.send(Action::ShowLoading {
            message: loading_message,
        })?;

        // Switch to command execution mode
        self.mode = Mode::CommandExecution;

        tokio::spawn(async move {
            let output = Command::new(&command).args(&args).output().await;

            // Hide loading animation
            let _ = action_tx.send(Action::HideLoading);

            match output {
                Ok(result) => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);

                    if !stdout.is_empty() {
                        info!("Command output:\n{}", stdout);
                    }
                    if !stderr.is_empty() {
                        info!("Command stderr:\n{}", stderr);
                    }

                    let success = result.status.success();
                    if success {
                        let _ = action_tx.send(Action::ShowStatus {
                            message: format!("✅ Command '{}' completed successfully", command),
                            status_type: crate::action::StatusType::Success,
                        });
                    } else {
                        let exit_code = result.status.code();
                        let error_msg = if !stderr.is_empty() {
                            stderr.to_string()
                        } else {
                            "Command failed with no error output".to_string()
                        };

                        let atm_error = crate::errors::AtmError::command_execution_failed(
                            &format!("{} {}", command, args.join(" ")),
                            exit_code,
                            &error_msg,
                        );

                        let _ = action_tx.send(Action::ShowStatus {
                            message: format!("❌ {}", atm_error),
                            status_type: crate::action::StatusType::Error,
                        });
                    }

                    let _ = action_tx.send(Action::CommandCompleted {
                        success,
                        output: stdout.to_string(),
                        error: stderr.to_string(),
                    });
                }
                Err(e) => {
                    let atm_error = crate::errors::AtmError::command_execution_failed(
                        &format!("{} {}", command, args.join(" ")),
                        None,
                        &e.to_string(),
                    );

                    let _ = action_tx.send(Action::ShowStatus {
                        message: format!("❌ {}", atm_error),
                        status_type: crate::action::StatusType::Error,
                    });

                    let _ = action_tx.send(Action::Error(atm_error.to_string()));
                }
            }
        });

        Ok(())
    }

    fn execute_shell_script(&mut self, script: &str) -> Result<()> {
        use tokio::process::Command;

        let action_tx = self.action_tx.clone();
        let script = script.to_string();

        // Start loading animation
        let loading_message = "Executing shell script...".to_string();
        action_tx.send(Action::CommandStarted {
            command: "bash".to_string(),
            description: loading_message.clone(),
        })?;
        action_tx.send(Action::ShowLoading {
            message: loading_message,
        })?;

        // Switch to command execution mode
        self.mode = Mode::CommandExecution;

        tokio::spawn(async move {
            let output = Command::new("bash").arg("-c").arg(&script).output().await;

            // Hide loading animation
            let _ = action_tx.send(Action::HideLoading);

            match output {
                Ok(result) => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);

                    if !stdout.is_empty() {
                        info!("Script output:\n{}", stdout);
                    }
                    if !stderr.is_empty() {
                        info!("Script stderr:\n{}", stderr);
                    }

                    let success = result.status.success();
                    if success {
                        let _ = action_tx.send(Action::ShowStatus {
                            message: "✅ Shell script executed successfully".to_string(),
                            status_type: crate::action::StatusType::Success,
                        });
                    } else {
                        let exit_code = result.status.code();
                        let error_msg = if !stderr.is_empty() {
                            stderr.to_string()
                        } else {
                            "Script failed with no error output".to_string()
                        };

                        let atm_error = crate::errors::AtmError::command_execution_failed(
                            "bash script",
                            exit_code,
                            &error_msg,
                        );

                        let _ = action_tx.send(Action::ShowStatus {
                            message: format!("❌ {}", atm_error),
                            status_type: crate::action::StatusType::Error,
                        });
                    }

                    let _ = action_tx.send(Action::CommandCompleted {
                        success,
                        output: stdout.to_string(),
                        error: stderr.to_string(),
                    });
                }
                Err(e) => {
                    let atm_error = crate::errors::AtmError::command_execution_failed(
                        "bash script",
                        None,
                        &e.to_string(),
                    );

                    let _ = action_tx.send(Action::ShowStatus {
                        message: format!("❌ {}", atm_error),
                        status_type: crate::action::StatusType::Error,
                    });

                    let _ = action_tx.send(Action::Error(atm_error.to_string()));
                }
            }
        });

        Ok(())
    }

    fn handle_command_completed(
        &mut self,
        success: bool,
        output: &str,
        error_output: &str,
    ) -> Result<()> {
        // Switch back to the appropriate menu mode
        match self.current_menu_stack.last() {
            Some(MenuState::Main) => {
                self.mode = Mode::Home;
            }
            Some(MenuState::Module(_)) => {
                self.mode = Mode::ModuleMenu;
            }
            None => {
                self.mode = Mode::Home;
            }
        }

        // Show status message based on command result with confirmation prompt
        if success {
            info!("Command completed successfully");
            if !output.is_empty() {
                info!(
                    "Output:
{}",
                    output
                );
            }

            // Show success message with confirmation prompt
            self.action_tx.send(Action::AwaitingConfirmation {
                message: "✅ Command completed successfully - Press Enter to continue".to_string(),
                result_type: StatusType::Success,
            })?;
        } else {
            error!("Command failed");
            if !error_output.is_empty() {
                error!(
                    "Error output:
{}",
                    error_output
                );
            }

            // Show error message with confirmation prompt
            self.action_tx.send(Action::AwaitingConfirmation {
                message: "❌ Command failed - Press Enter to continue".to_string(),
                result_type: StatusType::Error,
            })?;
        }

        // TODO: Show command output in the UI (maybe a popup or dedicated view)
        // For now, just log it

        Ok(())
    }

    fn check_dependency(&self, dependency: &str) -> Result<bool> {
        use std::process::Command;

        // Check if the dependency (command/package) is available
        let result = Command::new("which").arg(dependency).output();

        match result {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                // If 'which' command fails, try alternative checks
                match dependency {
                    "git" => self.check_package_installed("git"),
                    "curl" => self.check_package_installed("curl"),
                    "wget" => self.check_package_installed("wget"),
                    "yay" => self.check_package_installed("yay"),
                    "paru" => self.check_package_installed("paru"),
                    _ => {
                        // For unknown dependencies, assume they're available
                        // This is safer than blocking execution
                        info!("Unknown dependency '{}', assuming available", dependency);
                        Ok(true)
                    }
                }
            }
        }
    }

    fn check_package_installed(&self, package: &str) -> Result<bool> {
        use std::process::Command;

        // Use pacman to check if package is installed (Arch Linux specific)
        let result = Command::new("pacman").args(&["-Q", package]).output();

        match result {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                info!("Could not check package '{}' with pacman", package);
                Ok(false)
            }
        }
    }
    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }

            // Clone the action for component updates before consuming it in the match
            let action_for_components = action.clone();

            match action {
                Action::Tick => {
                    // Update loading animation frame
                    if self.loading_state.is_loading {
                        self.loading_state.animation_frame =
                            (self.loading_state.animation_frame + 1) % 8;
                    }

                    // Auto-clear status messages after 5 seconds
                    if self.status_state.visible {
                        if let Some(timestamp) = self.status_state.timestamp {
                            if timestamp.elapsed().as_secs() >= 5 {
                                self.status_state.visible = false;
                                self.status_state.timestamp = None;
                            }
                        }
                    }
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::CommandStarted {
                    command: _,
                    description: _,
                } => {
                    // Command started notification - just log it for now
                    debug!("Command started");
                }
                Action::ShowLoading { message } => {
                    self.loading_state.is_loading = true;
                    self.loading_state.message = message;
                    self.loading_state.start_time = Some(Instant::now());
                    self.loading_state.animation_frame = 0;
                }
                Action::HideLoading => {
                    self.loading_state.is_loading = false;
                    self.loading_state.start_time = None;
                    self.loading_state.progress = None;
                }
                Action::UpdateProgress {
                    percentage,
                    message,
                } => {
                    if self.loading_state.is_loading {
                        self.loading_state.progress = Some(percentage as f32);
                        self.loading_state.message = message;
                    }
                }
                Action::VersionDetected {
                    module,
                    version_info,
                } => {
                    self.version_info.insert(module, version_info);
                }
                Action::ShowStatus {
                    message,
                    status_type,
                } => {
                    self.status_state.message = message;
                    self.status_state.status_type = status_type;
                    self.status_state.visible = true;
                    self.status_state.timestamp = Some(Instant::now());
                }
                Action::ClearStatus => {
                    self.status_state.visible = false;
                    self.status_state.timestamp = None;
                }
                Action::CommandCompleted {
                    success,
                    output,
                    error,
                } => {
                    self.handle_command_completed(success, &output, &error)?;
                }
                Action::Error(msg) => {
                    error!("Application error: {}", msg);

                    // Parse error message to provide helpful guidance
                    let user_friendly_msg = if msg.contains("Configuration Error") {
                        format!("📄 {}", msg)
                    } else if msg.contains("Module Error") {
                        format!("🧩 {}", msg)
                    } else if msg.contains("Command Failed") {
                        format!("⚡ {}", msg)
                    } else if msg.contains("File Error") {
                        format!("📁 {}", msg)
                    } else if msg.contains("JSON Parsing Error") {
                        format!("📝 {}", msg)
                    } else if msg.contains("Terminal UI Error") {
                        format!("🖥️  {}", msg)
                    } else {
                        format!("❓ Unexpected Error: {}", msg)
                    };

                    // Show error status with clear categorization
                    self.status_state.message = user_friendly_msg;
                    self.status_state.status_type = crate::action::StatusType::Error;
                    self.status_state.visible = true;
                    self.status_state.timestamp = Some(Instant::now());

                    // For critical errors, provide additional guidance
                    if msg.contains("Configuration Error") || msg.contains("Module Error") {
                        eprintln!("\n🔧 Quick Fix Guide:");
                        eprintln!("   1. Check file paths and permissions");
                        eprintln!("   2. Validate JSON syntax with an online checker");
                        eprintln!("   3. Ensure all required configuration files exist");
                        eprintln!(
                            "   4. Run: './arch-tool-meister --list-modules' to verify setup"
                        );
                    }
                }
                Action::AwaitingConfirmation {
                    message,
                    result_type,
                } => {
                    self.awaiting_confirmation = true;
                    self.confirmation_message = message;
                    self.confirmation_result_type = result_type;
                }
                Action::ConfirmationReceived => {
                    self.awaiting_confirmation = false;

                    // Show the final status after confirmation
                    self.status_state.message = self.confirmation_message.clone();
                    self.status_state.status_type = self.confirmation_result_type.clone();
                    self.status_state.visible = true;
                    self.status_state.timestamp = Some(Instant::now());

                    // Clear confirmation state
                    self.confirmation_message.clear();
                    self.confirmation_result_type = StatusType::Info;
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action_for_components.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            // Use the menu component to render the current menu
            if let Err(err) = self.menu_component.render_menu(frame, frame.area(), self) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to render menu: {:?}", err)));
            }
        })?;
        Ok(())
    }

    // Public getters for components to access application state
    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }

    pub fn get_main_menu_config(&self) -> &MainMenuConfig {
        &self.main_menu_config
    }

    pub fn get_module_registry(&self) -> &ModuleRegistry {
        &self.module_registry
    }

    pub fn get_current_menu_state(&self) -> Option<&MenuState> {
        self.current_menu_stack.last()
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    pub fn get_loading_state(&self) -> &LoadingState {
        &self.loading_state
    }

    pub fn get_loading_animation(&self) -> &str {
        const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        SPINNER_FRAMES[self.loading_state.animation_frame % SPINNER_FRAMES.len()]
    }

    pub fn get_version_info(&self, module: &str) -> Option<&VersionInfo> {
        self.version_info.get(module)
    }

    pub fn get_scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }

    pub fn get_status_state(&self) -> &StatusState {
        &self.status_state
    }

    pub fn is_awaiting_confirmation(&self) -> bool {
        self.awaiting_confirmation
    }

    pub fn get_confirmation_message(&self) -> &str {
        &self.confirmation_message
    }

    pub fn get_confirmation_result_type(&self) -> &StatusType {
        &self.confirmation_result_type
    }

    pub fn update_scroll_for_menu(&mut self, viewport_height: usize) {
        let options = self.get_current_menu_options();
        self.scroll_state
            .update_content(options.len(), viewport_height);
    }

    fn detect_vscode_versions(&mut self) -> Result<()> {
        use tokio::process::Command;

        let action_tx = self.action_tx.clone();

        tokio::spawn(async move {
            let mut version_info = VersionInfo {
                stable_version: None,
                insiders_version: None,
                stable_installed: false,
                insiders_installed: false,
            };

            // Check VS Code Stable
            if let Ok(output) = Command::new("code").arg("--version").output().await {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    let version = version_str.lines().next().unwrap_or("Unknown").to_string();
                    version_info.stable_version = Some(version);
                    version_info.stable_installed = true;
                }
            }

            // Check VS Code Insiders
            if let Ok(output) = Command::new("code-insiders")
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    let version = version_str.lines().next().unwrap_or("Unknown").to_string();
                    version_info.insiders_version = Some(version);
                    version_info.insiders_installed = true;
                }
            }

            let _ = action_tx.send(Action::VersionDetected {
                module: "vscode".to_string(),
                version_info,
            });
        });

        Ok(())
    }
}
