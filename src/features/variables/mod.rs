//! Variables feature module
//!
//! This module contains everything related to class variables:
//! - Type definitions (ClassVariable, VariableDrag, TypeItem)
//! - Variable lifecycle operations (create, delete, get/set nodes)
//! - Variables panel UI
//! - Variable list rendering

pub mod types;
pub mod operations;
pub mod panel;
pub mod rendering;

// Re-export commonly used types
pub use types::{ClassVariable, VariableDrag, TypeItem};
pub use panel::VariablesPanel;
pub use rendering::VariablesRenderer;
