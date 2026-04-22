//! Compilation feature - Compiling blueprints to Rust code
//!
//! This module handles:
//! - Compiling graphs to Rust source code
//! - Converting between BlueprintGraph and GraphDescription formats
//! - Compilation status tracking and history
//! - Compiler panel rendering

pub mod compiler;
pub mod conversion;
pub mod panel;

// Re-export commonly used items
pub use compiler::{start_compilation, compile_to_rust, compile_to_class_directory};
pub use conversion::{convert_to_graph_description, convert_from_graph_description};
