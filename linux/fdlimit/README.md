# FDLimit - Linux File Descriptor Limits Manager

A Python utility for managing Linux file descriptor limits through a user-friendly TUI interface.

## Features

- View current system file descriptor limits
- Apply recommended limits for high-performance applications (65535/1000000)
- Set custom limits for user and system-wide file descriptors
- Manage both session and persistent limits
- Easy-to-use TUI (Text User Interface)
- Command-line interface for non-interactive usage
- Built-in safety checks and input validation

## Installation

### Using Poetry (recommended)

```bash
# Install Poetry if you don't have it yet
curl -sSL https://install.python-poetry.org | python3 -

# Install the package
poetry install

# For development with extra dependencies
poetry install --with dev
```

### Using pip (alternative)

```bash
# Install using pip
pip install .

# For development
pip install -e ".[dev]"
```

## Usage

### Interactive Mode

Run the file descriptor limits manager with:

```bash
# Using Poetry
poetry run fdlimit

# Using Poetry with root privileges
sudo poetry run fdlimit

# Or if installed globally
fdlimit

# With root privileges to modify system files
sudo fdlimit
```

### Non-Interactive Mode

The tool also supports non-interactive command-line usage:

```bash
# Show current limits (with Poetry)
poetry run fdlimit --show-current

# Apply recommended limits (65535/1000000)
sudo poetry run fdlimit --no-ui --apply-recommended

# Set custom limits (soft hard system_max)
sudo poetry run fdlimit --no-ui --set-limits 4096 8192 500000

# Show version
poetry run fdlimit --version

# Show help
poetry run fdlimit --help
```

## Safety Features

The tool includes several safety features:

- Validates all input values for proper format and reasonable ranges
- Creates backup files before modifying configuration files
- Restricts command execution to safe system commands only
- Checks for proper permissions before attempting to modify system files
- Provides clear error messages when operations fail

## Background

Linux file descriptor limits control how many files a process can open simultaneously. These limits are important for high-performance applications, databases, and services that handle many concurrent connections.

### Types of limits

- **Soft limit**: The current limit enforced by the kernel
- **Hard limit**: The maximum value that can be set for the soft limit
- **System-wide limit**: The total number of file descriptors the system can use

## Configuration files

The tool manages the following configuration files:

- `/etc/security/limits.conf` - User-level file descriptor limits
- `/etc/sysctl.conf` - System-wide file descriptor limits

## Development

### Running tests

```bash
# Run basic tests
poetry run test

# Run tests with coverage reporting
poetry run test-cov

# Run the full test suite with comprehensive checks and HTML coverage report
poetry run test-all

# Run code quality checks with Ruff and mypy
poetry run lint

# Format your code with Ruff
poetry run format
```

### Building documentation

```bash
# Build documentation with Poetry
cd docs
poetry run make html
```

### Building package

```bash
# Build the package
poetry build
```

### Cleaning project

```bash
# Clean all temporary files, build artifacts, and caches
poetry run clean
```

## License

MIT License
