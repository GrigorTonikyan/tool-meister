# up-man: Universal Package Manager Updater

A command-line utility to manage and run updates for multiple package managers simultaneously.

## Features

- **Unified Updates**: Run updates for multiple package managers with a single command
- **Parallel Execution**: Optionally run package manager updates concurrently for faster performance
- **Real-time Progress**: Visual progress indicators for each package manager update
- **Auto-detection**: Automatically detect available package managers on your system
- **Configurability**: Customize which package managers to update and how
- **Shell Integration**: Set up aliases for easy access
- **Multiple Package Formats**: Available as Debian, RPM, AppImage, or standalone binary

## Installation

### Package Distributions

#### Debian/Ubuntu

```bash
sudo dpkg -i up-man_*.deb
```

#### Fedora/RHEL/CentOS

```bash
sudo rpm -i up-man-*.rpm
```

#### AppImage

```bash
chmod +x up-man-*.AppImage
./up-man-*.AppImage
```

#### Static Binary

```bash
tar -xzf up-man-*.tar.gz
sudo cp up-man /usr/local/bin/
```

### From Source

1. Clone the repository
2. Build with Cargo:

```bash
cargo build --release
```

3. The binary will be available at `target/release/up-man`

### Building Distribution Packages

To build all supported package formats (deb, rpm, AppImage, tarball):

```bash
./scripts/build-release.sh
```

This will create packages in the `packages/` directory.

### Testing Packages on Multiple Distributions

To test built packages on multiple Linux distributions using Docker:

```bash
./scripts/test-packages.sh
```

This will verify package installation and basic functionality on Ubuntu, Debian, Fedora, AlmaLinux, and Alpine.

## Usage

### Basic Usage

Run updates for all enabled package managers:

```bash
up-man
```

### Available Commands

- **Run updates**: `up-man run` or simply `up-man`
- **Validate config**: `up-man validate`
- **Backup config**: `up-man backup`
- **Detect package managers**: `up-man detect`
- **Setup shell alias**: `up-man setup-alias [NAME]`

### Options

- **Skip confirmation**: `up-man run --yes` or `up-man run -y`
- **Increase verbosity**: `up-man -v` or `up-man -vv` for more detailed output
- **Toggle parallel updates**: `up-man run --parallel` to override the config setting

## Configuration

Configuration is stored in `~/.config/up-all.toml`. A default configuration is created on first run.

### Global Settings

You can configure global settings in the `settings` section:

```toml
[settings]
# Enable parallel execution of package manager updates (default: false)
parallel-updates = true

# Default timeout in seconds for package manager updates (default: 300)
update-timeout-seconds = 600

# Default shell to use for commands (default: system default)
default-shell = "/bin/bash"

# Whether to keep a history of update logs (default: false)
log-history = true
```

### Package Manager Configuration

Each package manager is configured in a `[[package_manager]]` section:

```toml
[[package_manager]]
name = "APT"
enabled = true
command = "apt update && apt upgrade -y"
needs-sudo = true

[[package_manager]]
name = "Flatpak"
enabled = true
command = "flatpak update -y"
needs-sudo = false
```

Example configuration:

```toml
[[package_manager]]
name = "APT"
enabled = true
command = "apt update && apt full-upgrade -y && apt autoremove -y && apt autoclean"
needs-sudo = true

[[package_manager]]
name = "SNAP"
enabled = true
command = "snap refresh"
needs-sudo = true
```

### Configuration Options

- **name**: Display name for the package manager
- **enabled**: Whether to include this package manager when running updates
- **command**: The command to execute
- **needs-sudo**: Whether the command requires sudo privileges

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

See the [TODO.md](./TODO.md) file for planned features and improvements.
