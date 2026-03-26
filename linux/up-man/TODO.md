# Roadmap & TODOs for up-man

This document outlines planned features and improvements for the up-man project.

## Upcoming Features

### High Priority

- [x] **Testing Framework**: Implement unit and integration tests
  - [x] Config model tests
  - [x] Config manager tests
  - [x] Package manager detection tests
  - [x] Package manager runner tests
  - [ ] CLI command tests (partially implemented)
  - [ ] Mock test environment setup

- [ ] **Parallel Updates**: Implement concurrent execution of package manager updates
  - Add a configuration option to enable/disable parallel updates
  - Allow specifying dependencies between package managers
  - Add a timeout mechanism for hanging processes

- [ ] **Interactive Configuration Editor**: Add a command to edit configuration interactively
  - `up-man config edit` command to launch TUI editor
  - Add/remove/enable/disable package managers without manually editing TOML

### Medium Priority

- [ ] **Enhanced Package Manager Detection**:
  - Add version checking for detected package managers
  - Add version-specific command suggestions
  - Better handling of similar package managers (e.g., apt vs apt-get)
  - Suggestions for missing dependencies (e.g., suggesting cargo-update for Cargo)

- [x] **Terminal User Interface Improvements**:
  - [x] Real-time progress indicators for updates
  - [ ] Interactive selection of package managers to update
  - [ ] Detailed view of update logs
  - [ ] Split pane view showing active updates and completed updates

- [x] **Build and Packaging System**:
  - [x] Automated release builds for multiple targets
  - [x] Debian/Ubuntu package creation
  - [x] RPM package creation
  - [x] Static binary builds using musl
  - [x] Changelog management
  - [ ] AppImage creation

- [ ] **Logging Improvements**:
  - Log rotation for update history
  - Export update results to file (JSON, CSV)
  - Optional detailed logging of command outputs
  - Quiet mode for scripting (only errors)

### Low Priority

- [ ] **Desktop Notifications**:
  - Send desktop notification when updates complete
  - Summary of success/failure in notification
  - Click notification to open detailed report

- [ ] **Scheduler Integration**:
  - Helpers to set up cron/systemd timers
  - Automatic update scheduling
  - Low-priority execution mode for background updates

- [ ] **Update Management**:
  - Track update history
  - Statistical reports (update frequency, success rates)
  - Rollback capability for supported package managers
  - Support for pre/post update hooks

## Technical Improvements

- [x] **Code Documentation**:
  - [x] Module documentation
  - [x] Function documentation
  - [x] Configuration documentation

- [ ] **Code Quality**:
  - Address remaining warnings and dead code
  - Add more comprehensive error types
  - Improve error handling with custom error types
  - Add telemetry for crash reporting (opt-in)

- [ ] **Performance Optimization**:
  - Reduce memory usage
  - Optimize startup time
  - Lazy loading of package manager definitions

- [ ] **Packaging**:
  - Create distribution packages (.deb, .rpm)
  - Publish to crates.io
  - Docker container for CI/CD environments

## Release Planning

### v0.1.1

- Comprehensive test coverage
- Documentation improvements
- Bug fixes

### v0.2.0

- Parallel updates implementation
- Enhanced error handling
- Testing framework

### v0.3.0

- Interactive configuration editor
- TUI improvements
- Better package manager detection

### v1.0.0

- All high priority features
- Comprehensive test coverage
- Distribution packages
