//! Main editor state container and lifecycle management.
//!
//! This module contains the BlueprintEditorPanel - the central state container
//! for the blueprint editor, along with workspace, tabs, and toolbar.

pub mod panel;
pub mod workspace;
pub mod tabs;
pub mod toolbar;

pub use panel::BlueprintEditorPanel;
