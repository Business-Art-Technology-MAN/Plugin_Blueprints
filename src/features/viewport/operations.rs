//! Viewport operations - pan, zoom, and camera controls
//!
//! This module contains all viewport manipulation operations:
//! - Panning: Click-drag to move the viewport
//! - Zooming: Mouse wheel to zoom in/out, centered on cursor position

use gpui::*;
use crate::core::BlueprintGraph;
use crate::editor::panel::BlueprintEditorPanel;
use super::coordinates::screen_to_graph_pos;

// ============================================================================
// Pan Operations
// ============================================================================

/// Start panning the viewport
///
/// Called when the user initiates a pan gesture (e.g., middle mouse button down).
/// Captures the current state to calculate deltas during pan updates.
///
/// # Arguments
/// * `panel` - The editor panel containing viewport state
/// * `start_pos` - The screen position where panning started
/// * `cx` - GPUI context for triggering updates
pub fn start_panning(panel: &mut BlueprintEditorPanel, start_pos: Point<f32>, cx: &mut Context<BlueprintEditorPanel>) {
    panel.is_panning = true;
    panel.pan_start = start_pos;
    panel.pan_start_offset = panel.graph.pan_offset;
    cx.notify();
}

/// Update pan position during a pan gesture
///
/// Called on mouse move events while panning is active.
/// Calculates the delta from the pan start position and updates the graph offset.
///
/// # Arguments
/// * `panel` - The editor panel containing viewport state
/// * `current_pos` - The current screen position of the cursor
/// * `cx` - GPUI context for triggering updates
pub fn update_pan(panel: &mut BlueprintEditorPanel, current_pos: Point<f32>, cx: &mut Context<BlueprintEditorPanel>) {
    if panel.is_panning {
        let delta = Point::new(
            current_pos.x - panel.pan_start.x,
            current_pos.y - panel.pan_start.y,
        );
        panel.graph.pan_offset = Point::new(
            panel.pan_start_offset.x + delta.x / panel.graph.zoom_level,
            panel.pan_start_offset.y + delta.y / panel.graph.zoom_level,
        );
        cx.notify();
    }
}

/// End panning gesture
///
/// Called when the user releases the pan button.
/// Clears the panning state.
///
/// # Arguments
/// * `panel` - The editor panel containing viewport state
/// * `cx` - GPUI context for triggering updates
pub fn end_panning(panel: &mut BlueprintEditorPanel, cx: &mut Context<BlueprintEditorPanel>) {
    panel.is_panning = false;
    cx.notify();
}

// ============================================================================
// Zoom Operations
// ============================================================================

/// Handle zoom with mouse wheel
///
/// Zooms around the cursor position to keep the point under cursor fixed.
/// This creates an intuitive zoom experience where the user zooms "into" the
/// point they're looking at.
///
/// # Algorithm
/// 1. Get the graph position under the cursor before zoom
/// 2. Calculate the new zoom level (clamped to 0.1-3.0)
/// 3. Calculate new pan offset to keep the focus point under the cursor
/// 4. Apply correction to compensate for coordinate system differences
///
/// # Arguments
/// * `panel` - The editor panel containing viewport state
/// * `delta_y` - Mouse wheel delta (positive = zoom in, negative = zoom out)
/// * `screen_pos` - Screen position of the cursor (for zoom focus point)
/// * `cx` - GPUI context for triggering updates
pub fn handle_zoom(
    panel: &mut BlueprintEditorPanel,
    delta_y: f32,
    screen_pos: Point<Pixels>,
    cx: &mut Context<BlueprintEditorPanel>,
) {
    let screen: Point<f32> = Point::new(screen_pos.x.into(), screen_pos.y.into());

    // Get graph position under cursor before zoom
    let focus_graph_pos = screen_to_graph_pos(
        Point::new(px(screen.x), px(screen.y)),
        &panel.graph,
    );

    // Calculate new zoom level (inverted scroll direction)
    let zoom_factor = if delta_y > 0.0 { 1.1 } else { 0.9 };
    let new_zoom = (panel.graph.zoom_level * zoom_factor).clamp(0.1, 3.0);

    #[cfg(debug_assertions)]
    tracing::debug!(
        "[ZOOM] screen=({:.2},{:.2}), focus_graph=({:.2},{:.2}), old_zoom={:.3}, old_pan=({:.2},{:.2}), delta_y={:.2}",
        screen.x, screen.y,
        focus_graph_pos.x, focus_graph_pos.y,
        panel.graph.zoom_level,
        panel.graph.pan_offset.x, panel.graph.pan_offset.y,
        delta_y
    );

    // Calculate new pan to keep focus point under cursor
    let mut new_pan_offset = Point::new(
        (screen.x / new_zoom) - focus_graph_pos.x,
        (screen.y / new_zoom) - focus_graph_pos.y,
    );

    // Apply temporarily to measure coordinate differences
    let old_zoom = panel.graph.zoom_level;
    let old_pan = panel.graph.pan_offset;

    panel.graph.zoom_level = new_zoom;
    panel.graph.pan_offset = new_pan_offset;

    // Measure screen position after zoom
    let screen_after = graph_to_screen_pos_internal(focus_graph_pos, &panel.graph);
    let diff_x = screen_after.x - screen.x;
    let diff_y = screen_after.y - screen.y;

    // Correct pan to compensate for coordinate system differences
    new_pan_offset.x -= diff_x / new_zoom;
    new_pan_offset.y -= diff_y / new_zoom;

    // Commit corrected values
    panel.graph.zoom_level = new_zoom;
    panel.graph.pan_offset = new_pan_offset;

    #[cfg(debug_assertions)]
    tracing::debug!(
        "[ZOOM] screen_after=({:.2},{:.2}), diff=({:.2},{:.2}), new_zoom={:.3}, new_pan=({:.3},{:.3})",
        screen_after.x, screen_after.y,
        diff_x, diff_y,
        new_zoom,
        new_pan_offset.x, new_pan_offset.y
    );

    cx.notify();
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Internal helper for graph-to-screen conversion used during zoom
/// (Duplicates the public function in coordinates.rs to avoid circular dependency)
fn graph_to_screen_pos_internal(graph_pos: Point<f32>, graph: &BlueprintGraph) -> Point<f32> {
    Point::new(
        (graph_pos.x + graph.pan_offset.x) * graph.zoom_level,
        (graph_pos.y + graph.pan_offset.y) * graph.zoom_level,
    )
}
