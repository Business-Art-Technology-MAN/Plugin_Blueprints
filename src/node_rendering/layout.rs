use gpui::Point;

use crate::{BlueprintGraph, BlueprintNode, NodeType};

pub const HEADER_H: f32 = 27.0;
pub const SEP_H: f32 = 1.0;
pub const BODY_PAD: f32 = 8.0;
pub const PIN_ROW_H: f32 = 16.0;
pub const PIN_GAP: f32 = 4.0;
pub const PIN_SIZE: f32 = 12.0;

/// Base node height for one pin row:
/// HEADER_H + SEP_H + BODY_PAD*2 + PIN_ROW_H = 44.
pub const NODE_BASE_H: f32 = HEADER_H + SEP_H + BODY_PAD * 2.0;

pub fn node_height_for_pin_rows(pin_rows: usize) -> f32 {
    let rows = pin_rows.max(1) as f32;
    NODE_BASE_H + rows * PIN_ROW_H + ((rows - 1.0).max(0.0)) * PIN_GAP
}

pub fn calculate_pin_anchor(
    node: &BlueprintNode,
    pin_id: &str,
    is_input: bool,
    graph: &BlueprintGraph,
) -> Option<Point<f32>> {
    if node.node_type == NodeType::Reroute {
        return Some(graph_to_screen_pos(node.position, graph));
    }

    let z = graph.zoom_level;
    let nsp = graph_to_screen_pos(node.position, graph);

    let row = if is_input {
        node.inputs.iter().position(|p| p.id == pin_id)?
    } else {
        node.outputs.iter().position(|p| p.id == pin_id)?
    };

    let pin_y = nsp.y
        + (HEADER_H + SEP_H + BODY_PAD) * z
        + row as f32 * (PIN_ROW_H + PIN_GAP) * z
        + PIN_ROW_H * 0.5 * z;

    let pin_x = if is_input {
        nsp.x + BODY_PAD * z
    } else {
        nsp.x + (node.size.width - BODY_PAD) * z
    };

    Some(Point::new(pin_x, pin_y))
}

fn graph_to_screen_pos(graph_pos: Point<f32>, graph: &BlueprintGraph) -> Point<f32> {
    Point::new(
        graph_pos.x * graph.zoom_level + graph.pan_offset.x,
        graph_pos.y * graph.zoom_level + graph.pan_offset.y,
    )
}
