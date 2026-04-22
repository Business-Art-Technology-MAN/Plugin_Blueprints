//! Node rendering - visual representation of blueprint nodes
//!
//! This module will handle rendering of individual nodes including:
//! - Node boxes with headers and bodies
//! - Input/output pins
//! - Node icons and titles
//! - Selection highlighting
//! - Reroute nodes

use gpui::*;
use crate::editor::panel::BlueprintEditorPanel;
use crate::core::types::BlueprintNode;

/// Render all nodes in the graph
/// This will be called from rendering/graph.rs
pub fn render_all(
    _panel: &mut BlueprintEditorPanel,
    _cx: &mut Context<BlueprintEditorPanel>,
) -> impl IntoElement {
    // TODO: Implement node rendering
    // This should:
    // 1. Iterate through visible nodes (with virtualization)
    // 2. Render each node with render_node()
    // 3. Apply selection highlighting

    div().absolute().inset_0()
}

/// Render a single blueprint node
pub fn render_node(
    _node: &BlueprintNode,
    _panel: &BlueprintEditorPanel,
    _cx: &mut Context<BlueprintEditorPanel>,
) -> impl IntoElement {
    // TODO: Implement single node rendering
    // This will be migrated from src_old/node_graph.rs

    div()
}
