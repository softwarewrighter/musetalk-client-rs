//! MuseTalk CLI library.
//!
//! This crate provides a command-line interface for generating lip-synced
//! avatar videos using the MuseTalk inference server.

pub mod assembler;
pub mod cli;
pub mod client;
pub mod error;
pub mod loader;
pub mod validation;

pub use cli::Args;
pub use error::{CliError, Result};
pub use validation::{ReferenceType, validate_inputs};
