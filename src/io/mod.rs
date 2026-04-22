//! File I/O and persistence layer
//!
//! Handles saving and loading blueprint files, including:
//! - Blueprint graph serialization/deserialization
//! - Legacy format support
//! - Format conversion utilities
//! - Autosave functionality

pub mod formats;
pub mod legacy;
pub mod save_load;

// Re-export main types and functions
pub use formats::{BlueprintAsset, BlueprintEditorState, serialize_blueprint_with_header, deserialize_blueprint};
pub use legacy::{try_parse_legacy_format, is_legacy_format};
