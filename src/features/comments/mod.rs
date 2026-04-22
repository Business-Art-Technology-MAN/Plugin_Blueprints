//! Comments feature module
//!
//! This module handles all comment-related functionality in the blueprint editor.
//! Comments are visual boxes that can group and annotate nodes in the graph.
//!
//! ## Submodules
//! - `operations`: Comment manipulation logic (drag, resize, edit, add, delete)
//! - `rendering`: Visual rendering of comments including color picker integration
//!
//! ## Key Features
//! - Drag comments to move them and all contained nodes
//! - Resize comments with 8-directional handles
//! - Double-click to edit comment text
//! - Color picker for customizing comment appearance
//! - Automatic tracking of contained nodes

pub mod operations;
pub mod rendering;

// Re-export commonly used types
pub use operations::*;
pub use rendering::*;
