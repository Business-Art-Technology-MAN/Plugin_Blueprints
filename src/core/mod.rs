//! Core module for the Blueprint Editor
//!
//! This module contains the fundamental types, data structures, and functionality
//! that power the blueprint graph system. It is organized into several submodules:
//!
//! - `types`: Core data structures (nodes, pins, connections, comments)
//! - `graph`: The main blueprint graph container and state management
//! - `definitions`: Node definition system for loading and managing node metadata
//! - `events`: Actions and event types for editor interactions
//! - `serialization`: Serde helpers for GPUI types and blueprint persistence

pub mod types;
pub mod graph;
pub mod definitions;
pub mod events;
pub mod serialization;

// Re-export commonly used types for convenience
pub use types::{
    BlueprintNode, Pin, Connection, BlueprintComment,
    NodeType, PinType, CompilationState, CompilationStatus,
    VirtualizationStats,
};

pub use graph::BlueprintGraph;

pub use definitions::{
    NodeDefinitions, NodeCategory, NodeDefinition, PinDefinition,
};

pub use events::{
    DuplicateNode, DeleteNode, CopyNode, PasteNode,
    DisconnectPin, OpenAddNodeMenu, OpenEngineLibraryRequest,
};
