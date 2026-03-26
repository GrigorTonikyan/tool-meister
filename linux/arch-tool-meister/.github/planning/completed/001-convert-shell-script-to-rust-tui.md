# [Issue #001] – Convert Shell Script to Rust TUI Application

status: completed

## Feature Description

### Scope of Files Affected

- A new Rust project will be created based on the `blank-target` template, which
  will eventually replace the current shell script-based implementation.
- `arch-tools-meister-modular.sh`: This 1400+ line Bash script will be replaced
  by the new Rust binary.
- `config.jsonc`, `main_menu.jsonc`: These files will be parsed by the Rust
  application to build the UI and functionality.
- `modules/**/*.jsonc`: All JSON files within the `modules` directory will be
  read by the Rust application.
- Existing module structure: `vscode`, `aur_helpers`, `git_config`, `system`,
  `maintenance`, `module_creator`

### Context / Background

The current `arch-tool-meister` is a sophisticated modular Bash script system
for managing Arch Linux tools and configurations. It features:

- **Fully Modular Architecture**: Dynamic module discovery and loading from the
  `modules/` directory
- **Interactive TUI**: Arrow key navigation (↑/↓), Enter key selection, number
  key shortcuts (1-9), and '0' for back/exit
- **JSONC Configuration**: Support for JSON with comments across three file
  types per module:
  - `config.jsonc`: Module metadata, settings, and main menu entry configuration
  - `menu.jsonc`: Module-specific menu structure and options
  - `commands.jsonc`: Command definitions, dependencies, and Bash function
    implementations
- **Command Line Interface**: Direct command execution via
  `--module <n> <command>` syntax
- **Advanced Features**: Loading animations, version detection for installed
  tools, dependency checking, and debug mode
- **Dynamic Menu Generation**: Main menu automatically includes entries for
  enabled modules

The system currently supports 6 modules: VSCode management, AUR helpers, Git
configuration, system utilities, maintenance tasks, and module creation tools.

### Purpose & Goals

- Migrate all functionality from the existing 1400+ line Bash script to a modern
  Rust application
- Preserve the exact user experience including navigation patterns, menu
  structures, and command execution
- Maintain full compatibility with the existing JSONC configuration system
- Improve performance, reliability, and error handling
- Enhance maintainability while preserving the modular architecture
- Provide a foundation for future enhancements and new modules

### Expected Outcome / Deliverable

- A single, standalone executable binary that replicates and improves upon the
  original tool's functionality
- Full preservation of the existing modular architecture and configuration
  system
- Enhanced TUI with improved responsiveness and visual feedback
- Comprehensive CLI interface matching the original `--module`,
  `--list-modules`, and `--debug` functionality
- Documentation and migration guide for transitioning from the shell script to
  the Rust binary

### Requirements / Specifications

- **Language**: Rust with `ratatui` for the TUI implementation
- **Configuration Compatibility**: Must parse existing `.jsonc` files with
  comment support
- **Module System**: Dynamic discovery and loading of modules from the
  `modules/` directory
- **Navigation**: Exact replication of current navigation patterns (arrow keys,
  Enter, numbers, '0')
- **Command Execution**: Ability to execute shell commands defined in module
  `commands.jsonc` files
- **CLI Interface**: Support for `--module <n> <command>`, `--list-modules`, and
  `--debug` flags
- **Animation System**: Loading animations and visual feedback during operations
- **Error Handling**: Robust error handling with user-friendly messages
- **Foundation**: Built upon the `blank-target` template structure

## Work Breakdown

### stage-01: Project Setup & Foundation

- [x] stage-01/task-01/step-01: Initialize the Rust project by copying the
      `blank-target` directory to a new location (e.g., `atm-rust-tui`).
- [x] stage-01/task-01/step-02: Update `Cargo.toml` with necessary dependencies:
      `ratatui`, `serde`, `serde_json`, `crossterm`, `tokio`, `color-eyre`,
      `tracing`, `clap`, `regex`, and `anyhow`.
- [x] stage-01/task-02/step-01: Define Rust data structures that mirror the JSON
      configuration files structure and derive `serde::Deserialize`.
- [x] stage-01/task-02/step-02: Implement a JSONC parser to strip comments
      before parsing with `serde_json`.
- [x] stage-01/task-02/step-03: Implement configuration loader for main config,
      main menu, and module configurations.
- [x] stage-01/task-02/step-04: Implement module discovery to find and load all
      available modules from the `modules/` directory.
- [x] stage-01/task-02/step-05: Create a module registry system to track loaded
      modules and their configurations.

### stage-02: Core TUI Implementation

- [x] stage-02/task-01/step-01: Set up the terminal using `crossterm` and
      initialize `ratatui` in `tui.rs`.
- [x] stage-02/task-01/step-02: Create the main application state structure in
      `app.rs` to manage current view, selected indices, and module data.
- [x] stage-02/task-01/step-03: Design and implement a basic UI layout with
      header (app name/version), main content area, and footer (navigation
      help).
- [x] stage-02/task-01/step-04: Implement event handling system to process
      keyboard input (arrow keys, Enter, numbers 1-9, '0', 'q').
- [x] stage-02/task-02/step-01: Create a `MainMenu` component that displays the
      dynamic main menu with module entries.
- [x] stage-02/task-02/step-02: Implement list rendering with selection
      highlighting and proper visual styling.
- [x] stage-02/task-02/step-03: Add support for both arrow key navigation and
      direct number selection (1-9).
- [x] stage-02/task-02/step-04: Implement '0' key functionality for returning to
      previous menu or exiting from main menu.

### stage-03: Module System & Navigation

- [x] stage-03/task-01/step-01: Create a `ModuleMenu` component to display
      module-specific menus loaded from `menu.jsonc`.
- [x] stage-03/task-01/step-02: Implement menu stack management to handle
      navigation between main menu and module menus.
- [x] stage-03/task-01/step-03: Add support for different menu option types:
      `scriptFunction`, `moduleMenu`, `return`, and `exit`.
- [x] stage-03/task-01/step-04: Implement proper menu title display and context
      awareness (showing current module name).
- [x] stage-03/task-02/step-01: Create a command execution system that can run
      shell commands asynchronously using `tokio::process::Command`.
- [x] stage-03/task-02/step-02: Implement command resolution from module
      `commands.jsonc` files with proper argument handling.
- [x] stage-03/task-02/step-03: Add dependency checking before command execution
      with user-friendly warnings.
- [x] stage-03/task-02/step-04: Capture and display command output (stdout and
      stderr) in the TUI.

### stage-04: Advanced Features & CLI Integration

- [x] stage-04/task-01/step-01: Implement loading animations and progress
      indicators during command execution.
- [x] stage-04/task-01/step-02: Add special handling for VSCode module to
      display installed and available versions.
- [x] stage-04/task-01/step-03: Implement scrolling support for long menus and
      command output displays.
- [x] stage-04/task-01/step-04: Add visual feedback for selections, loading
      states, and command execution status.
- [x] stage-04/task-02/step-01: Implement CLI argument parsing with `clap` for
      `--module`, `--list-modules`, and `--debug` flags.
- [x] stage-04/task-02/step-02: Add direct command execution mode that bypasses
      the TUI for CLI usage.
- [x] stage-04/task-02/step-03: Implement debug mode with verbose logging and
      output.
- [x] stage-04/task-02/step-04: Add module listing functionality that displays
      all available modules and their commands.

### stage-05: Error Handling & User Experience

- [x] stage-05/task-01/step-01: Implement comprehensive error handling for file
      I/O operations, JSON parsing failures, and command execution errors.
- [x] stage-05/task-01/step-02: Create user-friendly error messages that guide
      users toward solutions.
- [x] stage-05/task-01/step-03: Add proper error recovery mechanisms and
      graceful degradation.
- [x] stage-05/task-02/step-01: Implement the `tracing` crate for structured
      logging with configurable log levels.
- [x] stage-05/task-02/step-02: Add status bar or help display showing available
      keybindings and current context.
- [x] stage-05/task-02/step-03: Implement "Press Enter to continue" prompts
      after command execution completion.
- [x] stage-05/task-02/step-04: Add color-coded output for different message
      types (info, warning, error, success).

### stage-06: Testing & Documentation

- [x] stage-06/task-01/step-01: Write unit tests for JSONC parsing and
      configuration loading logic.
- [x] stage-06/task-01/step-02: Write unit tests for module discovery and
      command resolution systems.
- [x] stage-06/task-01/step-03: Write integration tests covering the main
      application flow and CLI functionality.
- [x] stage-06/task-01/step-04: Add tests for error handling and edge cases.
- [x] stage-06/task-02/step-01: Update the main `README.md` file with
      comprehensive build and usage instructions.
- [x] stage-06/task-02/step-02: Create migration guide from shell script to Rust
      binary.
- [x] stage-06/task-02/step-03: Add comprehensive code-level documentation (doc
      comments) for all public functions, structs, and modules.
- [x] stage-06/task-02/step-04: Create a `CHANGELOG.md` file documenting the
      migration and version history.
- [x] stage-06/task-02/step-05: Document the module system architecture and how
      to create new modules.
