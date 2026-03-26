# Module System Architecture

This document provides comprehensive documentation for the Arch Tool Meister
module system, including how modules work, how to create new modules, and
architectural design principles.

## 🎯 Overview

The Arch Tool Meister module system is designed to be:

- **Modular**: Each module is self-contained with its own configuration
- **Dynamic**: Modules are discovered and loaded automatically at runtime
- **Extensible**: New modules can be added without modifying core code
- **Configurable**: Rich JSONC configuration with comment support
- **Compatible**: Works identically in both Bash and Rust implementations

## 🏗️ Architecture Design

### Module Discovery Process

1. **Directory Scanning**: The application scans the `modules/` directory for
   subdirectories
2. **Configuration Validation**: Each module directory must contain three
   required files
3. **Registration**: Valid modules are registered in the module registry
4. **Integration**: Modules are integrated into the main menu and command system

### Module Registry

The module registry is the central hub for module management:

```rust
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleConfig>,      // Module metadata
    menus: HashMap<String, Menu>,                // Module menu structures
    commands: HashMap<String, HashMap<String, CommandConfig>>, // Module commands
}
```

**Key Features**:

- Concurrent module loading for performance
- Validation of all configuration files
- Error handling with detailed reporting
- Command resolution and execution

## 📁 Module Structure

Each module follows a standardized directory structure:

```text
modules/
  module-name/
    config.jsonc    # Module metadata and settings
    menu.jsonc      # Module menu structure
    commands.jsonc  # Command definitions and implementations
```

### Required Files

All three files are **required** for a module to be valid:

1. **`config.jsonc`**: Module metadata, settings, and main menu integration
2. **`menu.jsonc`**: Module-specific menu structure and navigation
3. **`commands.jsonc`**: Command definitions, dependencies, and shell
   implementations

## 📝 Configuration File Specifications

### config.jsonc - Module Metadata

**Purpose**: Defines module identity, settings, and main menu integration.

**Required Fields**:

```jsonc
{
  "name": "module_name", // Unique module identifier
  "description": "Module description", // Human-readable description
  "version": "1.0.0", // Module version (semantic versioning)
  "menuTitle": "Module Menu", // Title displayed in module menu
  "mainMenuEntry": "Module Tools", // Entry text in main menu
  "enabled": true // Whether module is active
}
```

**Optional Fields**:

```jsonc
{
  "settings": {
    // Module-specific settings
    "timeout": 30, // Command timeout in seconds
    "retries": 3, // Number of retry attempts
    "logLevel": "info", // Logging level for this module
    "dependencies": ["curl", "jq"], // Required system dependencies
    "environment": {
      // Environment variables
      "CUSTOM_VAR": "value"
    }
  },
  "metadata": {
    // Additional metadata
    "author": "Module Author",
    "license": "MIT",
    "repository": "https://github.com/user/repo",
    "documentation": "https://docs.example.com"
  }
}
```

**Validation Rules**:

- `name` must be a valid directory name (no spaces, special characters)
- `version` should follow semantic versioning (major.minor.patch)
- `enabled` determines if module appears in main menu
- `settings` can contain any valid JSON values for module-specific configuration

### menu.jsonc - Menu Structure

**Purpose**: Defines the module's menu structure and navigation options.

**Basic Structure**:

```jsonc
{
  "title": "Module Menu Title", // Menu header text
  "options": [
    // Array of menu options
    {
      "text": "Option Display Text", // Text shown to user
      "type": "scriptFunction", // Option type (see types below)
      "function": "command_name" // Command to execute
    }
  ]
}
```

**Option Types**:

1. **`scriptFunction`**: Execute a command from commands.jsonc

   ```jsonc
   {
     "text": "Install Package",
     "type": "scriptFunction",
     "function": "install_package" // Must exist in commands.jsonc
   }
   ```

2. **`return`**: Return to previous menu

   ```jsonc
   {
     "text": "Return to Main Menu",
     "type": "return"
   }
   ```

3. **`exit`**: Exit the application

   ```jsonc
   {
     "text": "Exit Application",
     "type": "exit"
   }
   ```

4. **`moduleMenu`**: Navigate to another module (future enhancement)

   ```jsonc
   {
     "text": "Go to System Tools",
     "type": "moduleMenu",
     "module": "system"
   }
   ```

**Advanced Options**:

```jsonc
{
  "text": "Advanced Command",
  "type": "scriptFunction",
  "function": "advanced_command",
  "description": "Detailed description of what this option does",
  "requiresConfirmation": true, // Show confirmation prompt
  "dangerous": true, // Mark as potentially dangerous
  "dependencies": ["git", "curl"] // Required for this specific option
}
```

### commands.jsonc - Command Definitions

**Purpose**: Defines executable commands with their implementations and
metadata.

**Structure Overview**:

```jsonc
{
  "commands": {
    // Command metadata
    "command_name": {
      "description": "What this command does",
      "function": "function_name", // Function to execute
      "dependencies": ["curl"], // Required system tools
      "args": ["--verbose"] // Default arguments
    }
  },
  "functions": {
    // Function implementations
    "function_name": {
      "code": "#!/bin/bash\necho 'Hello World'\nreturn 0"
    }
  }
}
```

**Command Definition Fields**:

```jsonc
{
  "description": "Human-readable description of the command",
  "function": "function_implementation_name", // Links to functions section
  "dependencies": ["git", "curl", "jq"], // Required system tools
  "args": ["--verbose", "--no-cache"], // Default command arguments
  "timeout": 60, // Command timeout (seconds)
  "requiresRoot": false, // Whether sudo is needed
  "interactive": false, // Whether command is interactive
  "dangerous": false, // Whether command modifies system
  "environment": {
    // Environment variables
    "GIT_EDITOR": "vim",
    "CUSTOM_PATH": "/opt/custom/bin"
  }
}
```

**Function Implementation**:

```jsonc
{
  "functions": {
    "install_package": {
      "code": "#!/bin/bash\nset -e\necho \"Installing package: $1\"\nsudo pacman -S --needed \"$1\"\necho \"Package installed successfully\"\nreturn 0"
    },
    "complex_function": {
      "code": "#!/bin/bash\n\n# Function with error handling\ninstall_tool() {\n    local tool=\"$1\"\n    \n    if command -v \"$tool\" >/dev/null 2>&1; then\n        echo \"$tool is already installed\"\n        return 0\n    fi\n    \n    echo \"Installing $tool...\"\n    if sudo pacman -S --needed \"$tool\"; then\n        echo \"Successfully installed $tool\"\n        return 0\n    else\n        echo \"Failed to install $tool\" >&2\n        return 1\n    fi\n}\n\n# Main execution\ninstall_tool \"$1\""
    }
  }
}
```

**Script Guidelines**:

- Always include shebang (`#!/bin/bash`)
- Use `set -e` for error handling
- Return proper exit codes (0 for success, non-zero for failure)
- Include user feedback with echo statements
- Handle errors gracefully with meaningful messages

## 🛠️ Creating a New Module

### Step 1: Module Planning

Before creating a module, define:

- **Purpose**: What problem does this module solve?
- **Commands**: What operations will it provide?
- **Dependencies**: What tools or packages are required?
- **Target Users**: Who will use this module?

### Step 2: Directory Setup

Create the module directory structure:

```bash
# Navigate to modules directory
cd modules/

# Create new module directory
mkdir my_module

# Create required configuration files
cd my_module
touch config.jsonc menu.jsonc commands.jsonc
```

### Step 3: Configure Module Metadata

Edit `config.jsonc`:

```jsonc
{
  // Basic module information
  "name": "my_module",
  "description": "My custom module for specific tasks",
  "version": "1.0.0",
  "menuTitle": "My Module Tools",
  "mainMenuEntry": "My Custom Tools",
  "enabled": true,

  // Optional settings
  "settings": {
    "timeout": 30,
    "logLevel": "info",
    "dependencies": ["curl", "jq"]
  },

  // Module metadata
  "metadata": {
    "author": "Your Name",
    "license": "MIT",
    "created": "2024-01-01"
  }
}
```

### Step 4: Design Menu Structure

Edit `menu.jsonc`:

```jsonc
{
  "title": "My Module Tools",
  "options": [
    {
      "text": "Basic Operation",
      "type": "scriptFunction",
      "function": "basic_operation",
      "description": "Performs a basic operation"
    },
    {
      "text": "Advanced Operation",
      "type": "scriptFunction",
      "function": "advanced_operation",
      "description": "Performs an advanced operation",
      "requiresConfirmation": true
    },
    {
      "text": "Show Information",
      "type": "scriptFunction",
      "function": "show_info"
    },
    {
      "text": "Return to Main Menu",
      "type": "return"
    }
  ]
}
```

### Step 5: Implement Commands

Edit `commands.jsonc`:

```jsonc
{
  "commands": {
    "basic_operation": {
      "description": "Performs a basic operation with minimal requirements",
      "function": "basic_operation_impl",
      "dependencies": [],
      "timeout": 10
    },
    "advanced_operation": {
      "description": "Performs an advanced operation with system modifications",
      "function": "advanced_operation_impl",
      "dependencies": ["curl", "jq"],
      "requiresRoot": true,
      "dangerous": true,
      "timeout": 60
    },
    "show_info": {
      "description": "Display module and system information",
      "function": "show_info_impl",
      "dependencies": []
    }
  },

  "functions": {
    "basic_operation_impl": {
      "code": "#!/bin/bash\nset -e\n\necho \"Performing basic operation...\"\necho \"Operation completed successfully!\"\nreturn 0"
    },

    "advanced_operation_impl": {
      "code": "#!/bin/bash\nset -e\n\n# Advanced operation with error handling\nperform_advanced_operation() {\n    echo \"Starting advanced operation...\"\n    \n    # Check dependencies\n    if ! command -v curl >/dev/null 2>&1; then\n        echo \"Error: curl is required but not installed\" >&2\n        return 1\n    fi\n    \n    if ! command -v jq >/dev/null 2>&1; then\n        echo \"Error: jq is required but not installed\" >&2\n        return 1\n    fi\n    \n    # Perform operation\n    echo \"Downloading data...\"\n    if curl -s \"https://api.github.com/zen\" | jq -r '.'; then\n        echo \"Advanced operation completed successfully!\"\n        return 0\n    else\n        echo \"Advanced operation failed\" >&2\n        return 1\n    fi\n}\n\n# Execute main function\nperform_advanced_operation"
    },

    "show_info_impl": {
      "code": "#!/bin/bash\nset -e\n\necho \"=== My Module Information ===\"\necho \"Version: 1.0.0\"\necho \"Author: Your Name\"\necho \"Description: Custom module for specific tasks\"\necho \"\"\necho \"=== System Information ===\"\necho \"Hostname: $(hostname)\"\necho \"Kernel: $(uname -r)\"\necho \"Architecture: $(uname -m)\"\necho \"Uptime: $(uptime -p)\"\nreturn 0"
    }
  }
}
```

### Step 6: Test the Module

Test your module using the CLI:

```bash
# List all modules (should include your new module)
arch-tool-meister --list-modules

# Test individual commands
arch-tool-meister --module my_module basic_operation
arch-tool-meister --module my_module show_info

# Test interactive mode
arch-tool-meister  # Navigate to your module in the TUI
```

### Step 7: Debug and Iterate

Use debug mode to troubleshoot issues:

```bash
# Debug mode for detailed output
arch-tool-meister --debug --module my_module basic_operation

# Check logs for errors
tail -f ~/.local/share/arch-tool-meister/arch-tool-meister.log

# Validate JSONC syntax
arch-tool-meister --debug --list-modules
```

## 📋 Best Practices

### Configuration Design

1. **Consistent Naming**: Use snake_case for module names and function names
2. **Clear Descriptions**: Write descriptive text for all commands and options
3. **Semantic Versioning**: Use proper version numbers (major.minor.patch)
4. **Dependency Declaration**: Always list required system dependencies

### Menu Design

1. **Logical Organization**: Group related commands together
2. **Clear Text**: Use descriptive menu option text
3. **Return Option**: Always include a "Return to Main Menu" option
4. **Confirmation**: Use `requiresConfirmation` for dangerous operations

### Command Implementation

1. **Error Handling**: Use `set -e` and proper return codes
2. **User Feedback**: Provide clear progress and completion messages
3. **Dependency Checking**: Validate required tools before execution
4. **Safe Defaults**: Use conservative defaults for dangerous operations

### Code Quality

1. **Shell Best Practices**: Follow shell scripting best practices
2. **Documentation**: Include comments in complex shell functions
3. **Testing**: Test all commands individually and in sequence
4. **Validation**: Ensure JSONC syntax is valid

## 🔧 Advanced Features

### Dynamic Command Arguments

Commands can accept arguments from user input:

```jsonc
{
  "commands": {
    "install_package": {
      "description": "Install a specific package",
      "function": "install_package_impl",
      "args": ["${PACKAGE_NAME}"] // Placeholder for user input
    }
  },
  "functions": {
    "install_package_impl": {
      "code": "#!/bin/bash\nset -e\npackage=\"$1\"\nif [ -z \"$package\" ]; then\n    read -p \"Enter package name: \" package\nfi\nsudo pacman -S --needed \"$package\""
    }
  }
}
```

### Environment Variables

Set custom environment variables for commands:

```jsonc
{
  "commands": {
    "custom_git_command": {
      "description": "Git command with custom configuration",
      "function": "custom_git_impl",
      "environment": {
        "GIT_EDITOR": "vim",
        "GIT_PAGER": "less -R"
      }
    }
  }
}
```

### Conditional Menu Options

Show menu options based on system state:

```jsonc
{
  "options": [
    {
      "text": "Update VSCode",
      "type": "scriptFunction",
      "function": "update_vscode",
      "condition": "vscode_installed" // Only show if VSCode is installed
    }
  ]
}
```

### Module Dependencies

Reference commands from other modules:

```jsonc
{
  "commands": {
    "complex_operation": {
      "description": "Operation that uses system module",
      "function": "complex_operation_impl",
      "moduleDependencies": ["system"], // Requires system module
      "prerequisites": ["system.get_info"] // Must run system info first
    }
  }
}
```

## 🧪 Testing Modules

### Unit Testing

Test individual commands:

```bash
#!/bin/bash
# test_my_module.sh

# Test basic operation
echo "Testing basic operation..."
if arch-tool-meister --module my_module basic_operation; then
    echo "✅ Basic operation passed"
else
    echo "❌ Basic operation failed"
    exit 1
fi

# Test info display
echo "Testing info display..."
if arch-tool-meister --module my_module show_info; then
    echo "✅ Info display passed"
else
    echo "❌ Info display failed"
    exit 1
fi

echo "All tests passed!"
```

### Integration Testing

Test module integration with the main application:

```bash
# Test module discovery
arch-tool-meister --list-modules | grep "my_module" || {
    echo "❌ Module not discovered"
    exit 1
}

# Test menu navigation in TUI mode
# (Manual testing required for interactive features)
```

### Error Testing

Test error conditions:

```bash
# Test with missing dependencies
arch-tool-meister --module my_module advanced_operation  # Should handle missing deps gracefully

# Test with invalid JSONC
# Temporarily corrupt a JSONC file and verify error handling
```

## 🔍 Troubleshooting

### Common Issues

#### Module Not Appearing

**Symptoms**: Module doesn't appear in `--list-modules` output

**Diagnosis**:

```bash
# Check directory structure
ls -la modules/my_module/
# Should show: config.jsonc, menu.jsonc, commands.jsonc

# Debug module loading
arch-tool-meister --debug --list-modules
```

**Solutions**:

1. Ensure all three files exist and are readable
2. Validate JSONC syntax
3. Check module name consistency across files

#### Command Execution Failures

**Symptoms**: Commands fail to execute or return errors

**Diagnosis**:

```bash
# Test command directly
arch-tool-meister --debug --module my_module my_command

# Check logs
tail -f ~/.local/share/arch-tool-meister/arch-tool-meister.log
```

**Solutions**:

1. Verify shell script syntax
2. Check dependency availability
3. Test script in isolation
4. Review error handling

#### JSONC Syntax Errors

**Symptoms**: Module fails to load with parsing errors

**Diagnosis**:

- Check for trailing commas
- Verify quote matching
- Ensure proper comment syntax

**Solutions**:

1. Use a JSONC validator
2. Remove comments temporarily to test JSON validity
3. Check escape sequences in strings

### Debug Commands

```bash
# Module discovery debugging
RUST_LOG=debug arch-tool-meister --list-modules

# Command execution debugging
RUST_LOG=trace arch-tool-meister --module my_module my_command

# Configuration parsing debugging
arch-tool-meister --debug --list-modules 2>&1 | grep my_module
```

## 📚 Examples

### Complete Example Module

See the `modules/` directory for working examples:

- **`vscode/`**: Complex module with version detection and multiple install
  options
- **`git_config/`**: Simple configuration module with user input
- **`system/`**: Information display module with system commands
- **`maintenance/`**: System maintenance with potentially dangerous operations

### Minimal Example

For a quick start, here's a minimal working module:

```bash
# Create minimal module
mkdir -p modules/hello_world

# config.jsonc
cat > modules/hello_world/config.jsonc << 'EOF'
{
  "name": "hello_world",
  "description": "Simple hello world module",
  "version": "1.0.0",
  "menuTitle": "Hello World",
  "mainMenuEntry": "Hello World",
  "enabled": true
}
EOF

# menu.jsonc
cat > modules/hello_world/menu.jsonc << 'EOF'
{
  "title": "Hello World Menu",
  "options": [
    {
      "text": "Say Hello",
      "type": "scriptFunction",
      "function": "say_hello"
    },
    {
      "text": "Return to Main Menu",
      "type": "return"
    }
  ]
}
EOF

# commands.jsonc
cat > modules/hello_world/commands.jsonc << 'EOF'
{
  "commands": {
    "say_hello": {
      "description": "Print hello world message",
      "function": "say_hello_impl"
    }
  },
  "functions": {
    "say_hello_impl": {
      "code": "#!/bin/bash\necho 'Hello, World from Arch Tool Meister!'\nreturn 0"
    }
  }
}
EOF

# Test the module
arch-tool-meister --list-modules
arch-tool-meister --module hello_world say_hello
```

---

This documentation provides everything needed to understand and extend the Arch
Tool Meister module system. For additional examples and advanced features,
explore the existing modules in the `modules/` directory.
