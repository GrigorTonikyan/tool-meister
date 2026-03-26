# Code Documentation Guide

This document provides comprehensive documentation for the Arch Tool Meister
Rust TUI application, covering all major modules, functions, and architectural
decisions.

## 📁 Project Structure Overview

```bash
atm-rust-tui/
├── src/
│   ├── main.rs           # Application entry point
│   ├── app.rs            # Main application state and logic
│   ├── tui.rs            # Terminal UI management
│   ├── config.rs         # Configuration loading and module management
│   ├── action.rs         # Application actions and events
│   ├── cli.rs            # Command-line interface parsing
│   ├── errors.rs         # Error handling and recovery
│   ├── logging.rs        # Structured logging setup
│   └── components/       # UI components
│       ├── fps.rs        # FPS counter component
│       ├── home.rs       # Home screen component
│       └── menu.rs       # Menu rendering and navigation
├── tests/
│   └── integration_tests.rs # End-to-end testing
├── modules/              # Module configuration directory
├── config.jsonc          # Application configuration
└── main_menu.jsonc       # Main menu structure
```

## 🚀 Core Modules

### main.rs

**Purpose**: Application entry point and CLI argument processing

**Key Components**:

- Initializes logging system with tracing subscriber
- Parses command-line arguments using clap
- Handles direct command execution mode vs interactive TUI mode
- Sets up terminal initialization and cleanup

**Error Handling**:

- Graceful terminal cleanup on panic or exit
- Proper error propagation to user
- Logging configuration validation

**Dependencies**:

- `clap` for CLI argument parsing
- `tracing` for structured logging
- `color_eyre` for enhanced error reporting

### app.rs

**Purpose**: Main application state management and business logic

**Key Structures**:

```rust
pub struct App {
    pub state: AppState,
    pub current_menu: Menu,
    pub module_registry: ModuleRegistry,
    pub status_message: Option<(String, MessageType)>,
    pub loading: bool,
    pub awaiting_confirmation: Option<String>,
    pub should_quit: bool,
}

pub enum AppState {
    MainMenu,
    ModuleMenu { module_name: String },
    Executing { command: String },
    Error { message: String },
}
```

**Core Methods**:

- `new() -> Result<Self>`: Initialize application with configuration loading
- `run(&mut self, terminal: &mut Terminal<B>) -> Result<()>`: Main application
  loop
- `handle_key_event(&mut self, key: KeyEvent) -> Result<()>`: Process user input
- `handle_action(&mut self, action: Action) -> Result<()>`: Process application
  actions
- `execute_command(&mut self, module: &str, command: &str) -> Result<()>`:
  Command execution

**State Management**:

- Manages current menu navigation state
- Tracks loading and confirmation states
- Handles status message display with color coding
- Maintains module registry for dynamic module discovery

**Event Handling**:

- Keyboard input processing with arrow keys, Enter, numbers, 'q'
- Action-based state transitions
- Async command execution with confirmation prompts
- Error recovery and user feedback

### tui.rs

**Purpose**: Terminal User Interface management and event loop

**Key Components**:

```rust
pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>>
pub fn restore() -> Result<()>
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()>
```

**Functionality**:

- Terminal initialization with raw mode and alternate screen
- Event loop with 250ms tick rate for smooth animations
- Crossterm backend for cross-platform terminal support
- Proper cleanup and restoration of terminal state

**Error Handling**:

- Terminal state restoration on errors
- Graceful handling of terminal resize events
- Recovery from rendering failures

### config.rs

**Purpose**: Configuration loading, JSONC parsing, and module management

**Key Structures**:

```rust
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleConfig>,
    menus: HashMap<String, Menu>,
    commands: HashMap<String, HashMap<String, CommandConfig>>,
}

pub struct ModuleConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub menu_title: String,
    pub main_menu_entry: String,
    pub enabled: bool,
    pub settings: Option<serde_json::Value>,
}
```

**Core Functions**:

- `load_config(path: &Path) -> Result<serde_json::Value>`: Load and parse JSONC
  files
- `parse_jsonc(content: &str) -> Result<serde_json::Value>`: Remove comments and
  parse JSON
- `load_main_menu(config_dir: &Path) -> Result<Menu>`: Load main menu
  configuration
- `ModuleRegistry::discover_modules(modules_dir: &Path) -> Result<Self>`:
  Dynamic module discovery

**Module Discovery Process**:

1. Scan modules directory for subdirectories
2. Load and validate `config.jsonc` for each module
3. Parse `menu.jsonc` for menu structure
4. Load `commands.jsonc` for command definitions
5. Build registry with all validated modules

**Configuration Validation**:

- JSONC comment removal with proper string handling
- Required field validation for all configuration files
- Error reporting with file path context
- Graceful fallback for missing optional settings

### action.rs

**Purpose**: Define application actions and events for decoupled communication

**Action Types**:

```rust
pub enum Action {
    Quit,
    NavigateUp,
    NavigateDown,
    Select,
    Return,
    ExecuteCommand { module: String, command: String },
    SetStatus { message: String, message_type: MessageType },
    ClearStatus,
    SetLoading(bool),
    AwaitingConfirmation { message: String },
    ConfirmationReceived,
}

pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}
```

**Design Pattern**:

- Action-based state management for predictable state transitions
- Decoupled component communication
- Type-safe event handling
- Clear separation of concerns between UI and business logic

### cli.rs

**Purpose**: Command-line interface definition and argument parsing

**CLI Structure**:

```rust
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub debug: bool,

    #[arg(long)]
    pub list_modules: bool,

    #[arg(long, value_name = "MODULE")]
    pub module: Option<String>,

    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,
}
```

**Features**:

- Version and help text generation
- Debug mode flag for verbose output
- Module listing functionality
- Direct command execution without TUI
- Argument validation and error reporting

### errors.rs

**Purpose**: Comprehensive error handling and recovery strategies

**Error Types**:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),
}
```

**Error Recovery**:

- Graceful fallback configurations
- User-friendly error messages with context
- Logging of detailed error information for debugging
- Recovery suggestions in error messages

### logging.rs

**Purpose**: Structured logging setup with tracing

**Configuration**:

- File-based logging to `~/.local/share/arch-tool-meister/arch-tool-meister.log`
- Configurable log levels (trace, debug, info, warn, error)
- Structured logging with context fields
- Debug mode console output

**Usage Examples**:

```rust
tracing::info!("Loading module: {}", module_name);
tracing::debug!("Configuration parsed: {:?}", config);
tracing::error!("Failed to execute command: {}", error);
```

## 🎨 UI Components

### components/menu.rs

**Purpose**: Menu rendering and navigation logic

**Key Functions**:

```rust
pub fn render_menu<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    menu: &Menu,
    selected: usize,
    status: Option<&(String, MessageType)>,
    loading: bool,
    awaiting_confirmation: Option<&String>,
)
```

**Features**:

- Scrollable menu support for long option lists
- Color-coded status messages (Info: Blue, Success: Green, Warning: Yellow,
  Error: Red)
- Loading animation with spinner
- Confirmation prompt overlay
- Keyboard shortcut display
- Dynamic menu sizing and layout

**Visual Elements**:

- Title bar with module information
- Numbered menu options (1-9, 0 for return/exit)
- Status bar with color-coded messages
- Help text with keybinding information
- Loading spinner animation

### components/home.rs

**Purpose**: Home screen component (currently minimal)

**Future Extensions**:

- Welcome message display
- System information summary
- Recent actions history
- Quick access shortcuts

### components/fps.rs

**Purpose**: FPS counter for performance monitoring

**Features**:

- Real-time frame rate calculation
- Debug mode display
- Performance monitoring during development

## ⚙️ Configuration System

### JSONC Support

The application supports JSONC (JSON with Comments) for all configuration files:

```jsonc
{
  // This is a comment
  "name": "example_module",
  "description": "Example module with comments",
  "settings": {
    "timeout": 30, // Timeout in seconds
    "retries": 3 // Number of retry attempts
  }
}
```

**Comment Removal Algorithm**:

1. Parse file content line by line
2. Remove `//` single-line comments
3. Remove `/* */` multi-line comments
4. Preserve comments within string literals
5. Maintain line numbers for error reporting

### Configuration Hierarchy

1. **Application Config** (`config.jsonc`): Global application settings
2. **Main Menu** (`main_menu.jsonc`): Main menu structure and options
3. **Module Config** (`modules/*/config.jsonc`): Module metadata and settings
4. **Module Menu** (`modules/*/menu.jsonc`): Module-specific menu structure
5. **Module Commands** (`modules/*/commands.jsonc`): Command definitions and
   implementations

### Fallback Behavior

- Missing configuration files trigger built-in defaults
- Invalid JSONC syntax reports detailed error with line numbers
- Partial module configurations are skipped with warnings
- Application continues with available modules on partial failures

## 🧪 Testing Architecture

### Unit Tests

Located within each module using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonc_parsing() {
        let content = r#"
        {
          // Comment
          "key": "value"
        }
        "#;
        let result = parse_jsonc(content).unwrap();
        assert_eq!(result["key"], "value");
    }
}
```

**Coverage Areas**:

- JSONC parsing with various comment formats
- Module discovery and validation
- Configuration loading with missing files
- Error handling and recovery scenarios
- Menu navigation logic
- Command resolution

### Integration Tests

Located in `tests/integration_tests.rs`:

**Test Categories**:

- CLI argument parsing and validation
- Application startup and shutdown
- Module loading and command execution
- Error handling with invalid configurations
- Terminal initialization and cleanup

**Test Utilities**:

- Temporary configuration generation
- Mock terminal backends for UI testing
- Command execution verification
- Log output validation

### Test Data

Comprehensive test configurations covering:

- Valid module configurations with all optional fields
- Invalid JSONC syntax for error handling testing
- Missing required fields for validation testing
- Edge cases like empty menus and commands
- Large configuration files for performance testing

## 🔄 Command Execution System

### Command Resolution

1. **Module Lookup**: Find module in registry by name
2. **Command Validation**: Verify command exists in module
3. **Dependency Check**: Validate required dependencies (future enhancement)
4. **Script Preparation**: Extract command code from configuration
5. **Execution**: Run command with proper environment and error handling

### Execution Flow

```rust
// Simplified execution flow
pub async fn execute_command(&self, module: &str, command: &str) -> Result<()> {
    let module_config = self.get_module(module)?;
    let command_config = self.get_command(module, command)?;

    // Prepare execution environment
    let script = &command_config.function_code;

    // Execute with timeout and error handling
    let output = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .output()
        .await?;

    // Handle result and user feedback
    self.handle_command_result(output).await
}
```

### Error Handling

- Command not found errors with suggestions
- Script execution failures with detailed output
- Timeout handling for long-running commands
- User cancellation support
- Proper cleanup of temporary resources

## 🎯 Performance Considerations

### Startup Performance

- **Module Discovery**: Concurrent loading of module configurations
- **Configuration Caching**: In-memory caching of parsed configurations
- **Lazy Loading**: Commands loaded only when needed
- **Optimized Parsing**: Efficient JSONC comment removal

### Runtime Performance

- **Event Loop**: 250ms tick rate for responsive UI without excessive CPU usage
- **Rendering Optimization**: Minimal redraw on static screens
- **Memory Management**: Efficient string handling and cloning
- **Async Execution**: Non-blocking command execution with progress feedback

### Memory Usage

- **Configuration Storage**: Optimized data structures for module registry
- **UI State**: Minimal state preservation for smooth navigation
- **Command History**: Limited history retention to prevent memory growth
- **Log Management**: Automatic log rotation and size limits

## 🔧 Development Guidelines

### Code Style

- **Rust Standards**: Follow official Rust style guidelines
- **Error Handling**: Use `Result<T, E>` for all fallible operations
- **Documentation**: Comprehensive doc comments for all public APIs
- **Testing**: Unit tests for all core functionality
- **Logging**: Structured logging with appropriate levels

### Architecture Principles

- **Separation of Concerns**: Clear boundaries between UI, business logic, and
  configuration
- **Modularity**: Well-defined interfaces between components
- **Error Recovery**: Graceful handling of all error conditions
- **User Experience**: Intuitive navigation and clear feedback
- **Performance**: Responsive UI with efficient resource usage

### Future Enhancements

- **Plugin System**: Dynamic loading of external modules
- **Configuration UI**: Web-based configuration management
- **Remote Modules**: Network-based module repositories
- **Command History**: Persistent command execution history
- **Scripting API**: Embedded scripting language for complex commands

---

This documentation provides a comprehensive overview of the Arch Tool Meister
codebase. For implementation details, refer to the inline documentation in each
source file generated with `cargo doc`.
