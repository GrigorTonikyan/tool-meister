# GEMINI.md

## Project Overview

This project, "tool-meister," is a command-line application written in Rust for managing and running tools within a workspace. It allows users to install, update, build, and run tools defined in configuration files. The application uses a concept of "manifests" to define and discover tools. These manifests can be sourced from local directories, Git repositories, or URLs.

The core technologies used are:

- **Rust:** The programming language used for the application.
- **clap:** A library for parsing command-line arguments.
- **serde:** A framework for serializing and deserializing Rust data structures.
- **tokio:** An asynchronous runtime for Rust.
- **JSONC:** A format for configuration files that allows for comments.

## Building and Running

### Building the Project

To build the project, use the following command:

```sh
cargo build
```

### Running the Project

To run the application, use the following command:

```sh
cargo run -- [COMMAND] [OPTIONS]
```

For example, to see the help message, run:

```sh
cargo run -- --help
```

### Running Tests

To run the integration tests, use the following command:

```sh
cargo test
```

## Development Conventions

### Code Style

The project follows the standard Rust coding style, which is enforced by the `rustfmt` tool.

### Testing

The project has a suite of integration tests located in the `tests` directory. These tests use the `assert_cmd` and `predicates` crates to verify the command-line interface's behavior. The tests are designed to be run in an isolated environment using temporary directories.

### Configuration

Tool configurations are defined in JSONC files. These files specify the tool's repository, dependencies, and the actions (commands) for installation, updates, builds, and execution.

### Manifests

The application uses a system of manifest sources to discover tools. These sources can be local directories, Git repositories, or URLs. The `manifests` command is used to manage these sources.
