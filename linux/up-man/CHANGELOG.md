# Changelog

All notable changes to the up-man project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Unreleased)

- Comprehensive test suite for core components
  - Unit tests for config model and manager
  - Unit tests for package manager detection
  - Unit tests for package manager runner
  - Integration tests for CLI commands
- Additional test helper methods for testability
- Updated TODO list with new feature ideas and progress tracking

### Changed

- Enhanced documentation with better module and function comments
- Improved error handling in package manager detection
- Case-insensitive package manager name matching

## [0.1.0] - 2025-04-04

### Added (0.1.0)

- Initial implementation of the up-man universal package manager updater
- Command-line interface with clap for argument parsing
- Configuration management with TOML format
- Package manager detection system
- Package manager update execution with status reporting
- Backup and validation functionality for configurations
- Shell alias setup functionality
- Comprehensive logging with colored output

[Unreleased]: https://github.com/GrigorTonikyan/up-man/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/GrigorTonikyan/up-man/releases/tag/v0.1.0
