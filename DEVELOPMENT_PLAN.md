# Development Plan: tool-meister

---

## **I. Code Refinements and Immediate Improvements**

### **1. Enhanced Error Handling and User Feedback**

* [x] **1.1. Capture and Display `stdout`/`stderr` on Command Failure**
  * [ ] Modify the `execute_actions` function in `src/commands.rs`.
  * [ ] When a command's exit status is not successful, capture the `stdout` and `stderr` of the failed command.
  * [ ] Update the `anyhow::bail!` message to include the captured `stdout` and `stderr`.
  * [ ] Add a new integration test in `tests/integration_tests.rs` that asserts that the error message contains the command's output on failure.

* [x] **1.2. Introduce More Specific Error Types**
  * [ ] Create a new `error.rs` module within the `src` directory.
  * [ ] Define a custom `Error` enum with variants for different error scenarios (e.g., `Config`, `Command`, `Io`).
  * [ ] Implement the `std::error::Error` and `std::fmt::Display` traits for the custom error type.
  * [ ] Implement `From` traits to convert underlying errors (e.g., `std::io::Error`, `toml::de::Error`) into the custom error types.
  * [ ] Refactor functions that currently return `anyhow::Result` to return `Result<T, crate::error::Error>` where appropriate (e.g., in `src/config.rs` and `src/global_config.rs`).
  * [ ] Update the call sites to handle the new, more specific error types.

#### **2. Improved Code Structure and Maintainability**

* [x] **2.1. Refactor `main.rs` by Modularizing Subcommand Logic**
  * [ ] In the `src/commands/` directory, create new files for each subcommand (e.g., `install.rs`, `update.rs`, `build.rs`, `run.rs`, `config.rs`, `manifests.rs`).
  * [ ] Move the implementation logic for each subcommand from `src/main.rs` into its corresponding new module.
  * [ ] In `src/main.rs`, replace the moved logic with calls to the new functions in the command modules.
  * [ ] Update `src/commands.rs` to act as a `mod.rs` file, declaring the new modules.

* [ ] **2.2. Consolidate Configuration Loading Logic**
  * [ ] Move the `load_tool_config` function from `src/main.rs` into `src/config.rs`.
  * [ ] Implement it as a new method on the `Config` struct, for example, `Config::load_with_fallback(global_config: &GlobalConfig, tool_name: &str) -> Result<Self>`.
  * [ ] Update `src/main.rs` to use the new `Config::load_with_fallback` method.

#### **3. Robust Configuration and Manifests**

* [ ] **3.1. Implement Manifest Caching for Remote Sources**
  * [ ] In `src/global_config.rs`, create a function to get the path to the manifest cache directory (e.g., in the user's config directory).
  * [ ] Modify the `find_tool_manifest` function to implement caching for `git` and `url` sources.
  * [ ] For `git` sources, clone the repository into the cache directory if it doesn't already exist.
  * [ ] For `url` sources, download the manifest to the cache directory.
  * [ ] Add logic to update the cache based on the `auto_update` flag (e.g., by running `git pull` or re-downloading the file).
  * [ ] Add integration tests to verify the caching and update logic.

* [ ] **3.2. Introduce JSON Schema Validation for Manifests**
  * [ ] Add a JSON schema validation library (e.g., `jsonschema`) to the `Cargo.toml` dependencies.
  * [ ] Create a `manifest.schema.json` file that defines the schema for the tool manifest files.
  * [ ] In `src/config.rs`, within the `Config::load_from_path` function, read the schema file.
  * [ ] After stripping comments from the manifest file, validate the JSON content against the loaded schema.
  * [ ] If validation fails, return a detailed error message.
  * [ ] Add a test case with an invalid manifest to ensure the validation is working correctly.

---

### **II. Future Development and New Features**

#### **1. Parallel Execution**

* [ ] **1.1. Design for Parallel Actions**
  * [ ] In `src/config.rs`, add a field to the `Action` struct to indicate whether it can be run in parallel (e.g., `parallel: bool`).
  * [ ] Update the `execute_actions` function in `src/commands.rs` to group consecutive actions that are marked for parallel execution.

* [ ] **1.2. Implement Parallel Execution**
  * [ ] Use `tokio::join!` or `futures::future::join_all` to execute the parallelizable actions concurrently.
  * [ ] Ensure that the `stdout` and `stderr` from the parallel commands are handled correctly to prevent interleaved output.
  * [ ] Add a test case with a manifest that defines parallel actions to verify that they are executed concurrently.

#### **2. Tool Versioning**

* [ ] **2.1. Update Configuration for Versioning**
  * [ ] In `src/config.rs`, modify the `Repository` struct to support versioning (e.g., by adding a `versions` field, which could be a list of git tags or URLs).
  * [ ] Add a `--version` option to the `install` and `run` commands in `src/main.rs`.

* [ ] **2.2. Implement Version-Aware Installation and Execution**
  * [ ] Modify the `install` command in `src/commands.rs` to handle the `--version` option.
  * [ ] Update the tool installation path to include the version (e.g., `tools/<tool_name>/<version>`).
  * [ ] Implement a mechanism to manage and switch between different installed versions of a tool (e.g., using symlinks).

#### **3. Plugin System**

* [ ] **3.1. Design the Plugin Architecture**
  * [ ] Define a clear plugin API (e.g., a Rust trait) that specifies how plugins can extend the application.
  * [ ] Determine the plugin discovery mechanism (e.g., a dedicated `plugins` directory).
  * [ ] Design how plugins can register new subcommands with `clap`.

* [ ] **3.2. Implement Plugin Loading and Integration**
  * [ ] Add a dynamic loading library (e.g., `libloading`) to `Cargo.toml`.
  * [ ] Implement the logic for discovering and loading plugins.
  * [ ] Integrate the loaded plugins into the main `clap` command structure.

#### **4. Interactive Mode**

* [ ] **4.1. Design the Interactive User Experience**
  * [ ] Map out the interactive flows for the `install` and `config` commands.
  * [ ] Choose a library for creating interactive command-line prompts (e.g., `dialoguer`).

* [ ] **4.2. Implement Interactive Commands**
  * [ ] Add an `--interactive` flag to the `install` and `config` commands.
  * [ ] When the `--interactive` flag is present, use the chosen library to guide the user with prompts.
  * [ ] Use the user's input from the prompts to execute the relevant actions.
