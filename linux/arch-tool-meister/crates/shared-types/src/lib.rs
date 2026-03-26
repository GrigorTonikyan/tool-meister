//! # Shared Types
//!
//! Common data structures and types used across all Arch Tool Meister crates.
//! This crate provides shared error types, configuration structures, and module-related types
//! to ensure consistency and interoperability between the core library, TUI, and web API.
//!
//! ## Modules
//!
//! - [`error`] - Shared error types and error handling utilities
//! - [`config`] - Configuration data structures and validation
//! - [`module`] - Module-related types and metadata structures

pub mod config;
pub mod error;
pub mod module;

// Re-export commonly used types for convenience
pub use config::{AppConfig, MenuConfig};
pub use error::SharedError;
pub use module::{
    MenuOption, MenuOptionType, ModuleCommand, ModuleDependency, ModuleFunction, ModuleInfo,
    ModuleInfoBuilder, ModuleMenu, ModuleStatus,
};
