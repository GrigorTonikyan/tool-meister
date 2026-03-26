# Arch Tool Meister

[![CI](https://github.com/GrigorTonikyan/arch-tool-meister/workflows/CI/badge.svg)](https://github.com/GrigorTonikyan/arch-tool-meister/actions)

A modular Rust TUI application for managing Arch Linux tools and configurations.

## Features

- **Modular Architecture**: Dynamically loads modules from the `modules/`
  directory
- **Terminal User Interface**: Rich TUI experience with ratatui
- **Configuration-Driven**: JSON configuration files with comments support
- **Module System**: Easily extensible with custom modules
- **Cross-Platform**: Works on any Unix-like system (primary focus on Arch
  Linux)

## Modules

The application comes with several built-in modules:

- **VSCode**: Install and manage VS Code stable/insiders
- **AUR Helpers**: Install Yay and Paru AUR helpers
- **Git Config**: Configure Git user settings
- **System**: System information and maintenance
- **Maintenance**: System cleanup and optimization tools

## Installation

### From Source

```bash
git clone https://github.com/GrigorTonikyan/arch-tool-meister.git
cd arch-tool-meister
cargo build --release
sudo cp target/release/arch-tool-meister /usr/local/bin/
```

## Usage

### Interactive TUI Mode

```bash
arch-tool-meister
```

### CLI Mode

```bash
# List available modules
arch-tool-meister --list-modules

# Execute module command
arch-tool-meister --module vscode install_stable

# Debug mode
arch-tool-meister --debug
```

## Configuration

The application uses JSONC (JSON with comments) configuration files:

- `config.jsonc` - Main application configuration
- `main_menu.jsonc` - Main menu structure  
- `modules/*/module.jsonc` - Unified module configurations

Each module has a single `module.jsonc` file that contains all module configuration including settings, menu structure, commands, and functions.

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Adding Modules

See the [Module System Documentation](.github/docs/MODULE_SYSTEM.md) for
detailed instructions on creating new modules.

## Documentation

- [Module System](.github/docs/MODULE_SYSTEM.md)
- [Code Documentation](.github/docs/CODE_DOCUMENTATION.md)
- [Migration Guide](.github/docs/MIGRATION_GUIDE.md)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.
