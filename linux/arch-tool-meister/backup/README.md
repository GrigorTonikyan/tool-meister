# Arch Tool Meister

A modular Bash script for managing Arch Linux tools and configurations.

## Features

- **Fully Modular Architecture**: Easy to extend with new modules
- **Interactive TUI**: Arrow key navigation, keyboard shortcuts
- **Version Detection**: Automatic detection of installed and available versions
- **Loading Animations**: Visual feedback during operations
- **Dependency Management**: Automatic detection of required dependencies
- **JSONC Configuration**: Clean and documented configuration with comments support

## Architecture

Arch Tool Meister uses a modular approach:

- **Main Script (`arch-tools-meister.sh`)**: Core framework that discovers and loads modules
- **Modules Directory**: Contains individual modules, each with its own functionality
- **Configuration Files**: JSON with comments (JSONC) files for configuration

### Module Structure

Each module follows this structure:

```
modules/
  module-name/
    config.jsonc    # Module configuration
    menu.jsonc      # Module menu structure
    commands.jsonc  # Module commands and functions
```

### Module Configuration

The `config.jsonc` file contains:

- Basic module information (name, description, version)
- Module-specific settings
- Main menu entry configuration

Example:
```jsonc
{
  "name": "example_module",
  "description": "Example module description",
  "version": "1.0.0",
  "menuTitle": "Example Menu",
  "mainMenuEntry": "Example Module",
  "enabled": true,
  "settings": {
    // Module-specific settings go here
    "setting1": "value1",
    "setting2": "value2"
  }
}
```

### Module Menu

The `menu.jsonc` file defines the menu structure for the module:

- Menu title
- Menu options
- Actions to perform when options are selected

Example:
```jsonc
{
  "title": "Example Module Menu",
  "options": [
    {
      "text": "Option 1",
      "type": "scriptFunction",
      "functionName": "example_function1"
    },
    {
      "text": "Option 2",
      "type": "scriptFunction",
      "functionName": "example_function2"
    },
    {
      "text": "Return to Main Menu",
      "type": "return"
    }
  ]
}
```

### Module Commands

The `commands.jsonc` file defines:

- Command metadata (description, dependencies)
- Function implementations in Bash script format
- Command arguments and options

Example:
```jsonc
{
  "commands": {
    "example_function1": {
      "description": "Example function 1",
      "dependencies": ["curl", "jq"],
      "function": "_example_function1",
      "args": []
    },
    "example_function2": {
      "description": "Example function 2",
      "function": "_example_function2",
      "args": ["arg1", "arg2"]
    }
  },
  "functions": {
    "_example_function1": {
      "code": "echo \"Running example function 1\"\nreturn 0"
    },
    "_example_function2": {
      "code": "local arg1=\"$1\"\nlocal arg2=\"$2\"\necho \"Running example function 2 with args: $arg1, $arg2\"\nreturn 0"
    }
  }
}
```

## Included Modules

- **VSCode**: Install, manage, and update Visual Studio Code (stable and insiders)
- **AUR Helpers**: Install popular AUR helpers (yay, paru)
- **Git Config**: Configure Git settings
- **System**: Basic system commands and information
- **Maintenance**: Clean up artifacts and manage backups
- **Module Creator**: Create new module templates easily

## Command Line Usage

```bash
./arch-tools-meister.sh [--debug] [--module <module-name> <command-name> [args...]] [--list-modules]
```

### Examples

- List all modules:

  ```bash
  ./arch-tools-meister.sh --list-modules
  ```

- Execute a module command directly:

  ```bash
  ./arch-tools-meister.sh --module vscode deploy_vscode_stable
  ```

- Run in debug mode:

  ```bash
  ./arch-tools-meister.sh --debug
  ```

## Interactive Menu Navigation

- **Arrow keys (↑/↓)**: Navigate through menu options
- **Enter**: Select the highlighted option
- **Numbers (1-9)**: Directly select an option by number
- **0**: Return to previous menu or exit from the main menu

## Adding a New Module

To create a new module:

1. Create a directory in the `modules/` folder with your module name
2. Add the three required files: `config.jsonc`, `menu.jsonc`, and `commands.jsonc`
3. The module will be automatically discovered and loaded on the next run

## Extending Existing Modules

To extend an existing module, simply modify its configuration files:

- Add new settings in `config.jsonc`
- Add new menu options in `menu.jsonc`
- Add new commands and functions in `commands.jsonc`

## Dependencies

- **Required**: jq, bash (4.0+), bc
- **Optional**: fzf (for enhanced selection), gum (for enhanced animations)

## License

This project is open source and available under the MIT license.

## Migration Summary

The Arch Tool Meister script has been completely refactored to use a modular architecture:

1. **Decoupled Configuration**: All menu options and commands moved from hardcoded script to JSONC files
2. **Module-Based Architecture**: Functionality organized into separate modules with standard interfaces
3. **Dynamic Module Discovery**: Automatically discovers and loads modules from the modules directory
4. **Enhanced Features**: Retained and improved arrow key navigation, animations, and version display
5. **New Features**: Added maintenance and module creation capabilities
6. **Improved Extensibility**: New functionality can be added without modifying the core script

This modular approach makes the script much more maintainable and easier to extend with new functionality.
