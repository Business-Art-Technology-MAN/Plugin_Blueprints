//! Connection rendering - bezier curves and visual presentation

use gpui::*;
use crate::panel::BlueprintEditorPanel;
use crate::core::types::{Connection, BlueprintNode, NodeType, Size};
use crate::core::graph::BlueprintGraph;
use ui::graph::DataType;
use super::operations::ConnectionDrag;

impl BlueprintEditorPanel {
    /// Render all connections in the graph
    pub fn render_connections(
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let mut connection_shapes: Vec<(Point<f32>, Point<f32>, gpui::Hsla)> = Vec::new();

        // Only render connections that connect to visible nodes
        let visible_connections: Vec<&Connection> = panel
            .graph
            .connections
            .iter()
            .filter(|connection| Self::is_connection_visible_simple(connection, &panel.graph))
            .collect();

        // Note: We can't mutate panel here since it's borrowed immutably
        // Connection virtualization stats will be updated in a different way

        // Debug info for connection virtualization
        if cfg!(debug_assertions) && panel.graph.connections.len() != visible_connections.len() {
            tracing::info!(
                "[BLUEPRINT-VIRTUALIZATION] Rendering {} of {} connections (saved {:.1}%)",
                visible_connections.len(),
                panel.graph.connections.len(),
                if panel.graph.connections.len() > 0 {
                    (1.0 - visible_connections.len() as f32 / panel.graph.connections.len() as f32)
                        * 100.0
                } else {
                    0.0
                }
            );
        }

        for connection in visible_connections {
            if let Some((from, to, color)) = Self::build_connection_shape(connection, panel, cx) {
                connection_shapes.push((from, to, color));
            }
        }

        let dragging_shape = panel
            .dragging_connection
            .as_ref()
            .and_then(|drag| Self::build_dragging_connection_shape(drag, panel, cx));

        let zoom_level = panel.graph.zoom_level;

        gpui::canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _prepaint_state, window, _cx| {
                let offset_x = bounds.origin.x.as_f32();
                let offset_y = bounds.origin.y.as_f32();

                for (from, to, color) in &connection_shapes {
                    Self::paint_bezier_line(
                        window,
                        Point::new(from.x + offset_x, from.y + offset_y),
                        Point::new(to.x + offset_x, to.y + offset_y),
                        *color,
                        zoom_level,
                    );
                }
                if let Some((from, to, color)) = &dragging_shape {
                    Self::paint_bezier_line(
                        window,
                        Point::new(from.x + offset_x, from.y + offset_y),
                        Point::new(to.x + offset_x, to.y + offset_y),
                        *color,
                        zoom_level,
                    );
                }
            },
        )
        .absolute()
        .inset_0()
        .size_full()
    }

    /// Build connection shape for rendering
    fn build_connection_shape(
        connection: &Connection,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> Option<(Point<f32>, Point<f32>, gpui::Hsla)> {
        let from_node = panel
            .graph
            .nodes
            .iter()
            .find(|n| n.id == connection.source_node);
        let to_node = panel
            .graph
            .nodes
            .iter()
            .find(|n| n.id == connection.target_node);

        if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
            if let (Some(from_pin_pos), Some(to_pin_pos)) = (
                Self::calculate_pin_position(
                    from_node,
                    &connection.source_pin,
                    false,
                    &panel.graph,
                ),
                Self::calculate_pin_position(to_node, &connection.target_pin, true, &panel.graph),
            ) {
                let pin_color = if let Some(pin) = from_node
                    .outputs
                    .iter()
                    .find(|p| p.id == connection.source_pin)
                {
                    Self::get_pin_color(&pin.data_type, cx)
                } else {
                    cx.theme().primary
                };

                Some((from_pin_pos, to_pin_pos, pin_color))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Build dragging connection shape for rendering
    fn build_dragging_connection_shape(
        drag: &ConnectionDrag,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> Option<(Point<f32>, Point<f32>, gpui::Hsla)> {
        if let Some(from_node) = panel.graph.nodes.iter().find(|n| n.id == drag.source_node) {
            if let Some(from_pin_pos) =
                Self::calculate_pin_position(from_node, &drag.source_pin, false, &panel.graph)
            {
                let pin_color = Self::get_pin_color(&drag.source_pin_type, cx);
                let end_pos = if let Some((target_node_id, target_pin_id)) = &drag.target_pin {
                    if let Some(target_node) = panel.graph.nodes.iter().find(|n| n.id == *target_node_id) {
                        Self::calculate_pin_position(target_node, target_pin_id, true, &panel.graph)
                            .unwrap_or(drag.current_mouse_pos)
                    } else {
                        drag.current_mouse_pos
                    }
                } else {
                    drag.current_mouse_pos
                };

                Some((from_pin_pos, end_pos, pin_color))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Paint a bezier line with glow effect
    pub fn paint_bezier_line(
        window: &mut gpui::Window,
        from_pos: Point<f32>,
        to_pos: Point<f32>,
        color: gpui::Hsla,
        zoom: f32,
    ) {
        let distance = ((to_pos.x - from_pos.x).powi(2) + (to_pos.y - from_pos.y).powi(2)).sqrt();
        let horiz_dist = (to_pos.x - from_pos.x).abs();
        let control_offset = (horiz_dist * 0.45).max(60.0).min(200.0);
        let control1 = Point::new(from_pos.x + control_offset, from_pos.y);
        let control2 = Point::new(to_pos.x - control_offset, to_pos.y);
        let thickness = 2.8_f32 * zoom;
        let segments = ((distance / 14.0).ceil() as usize).clamp(28, 80);

        // Paint soft outer glow first (wider, transparent)
        let glow_color = gpui::Hsla { h: color.h, s: color.s, l: color.l, a: 0.12 };
        let glow_thickness = thickness * 3.0;
        Self::paint_bezier_stroke(window, from_pos, to_pos, control1, control2, glow_color, glow_thickness, segments);

        // Paint the main wire
        Self::paint_bezier_stroke(window, from_pos, to_pos, control1, control2, color, thickness, segments);

        // Paint bright center highlight for a glossy wire look
        let highlight = gpui::Hsla { h: color.h, s: color.s * 0.5, l: (color.l + 0.25).min(0.95), a: 0.5 };
        let highlight_thickness = thickness * 0.35;
        Self::paint_bezier_stroke(window, from_pos, to_pos, control1, control2, highlight, highlight_thickness, segments);
    }

    /// Paint a single bezier stroke
    pub fn paint_bezier_stroke(
        window: &mut gpui::Window,
        from_pos: Point<f32>,
        to_pos: Point<f32>,
        control1: Point<f32>,
        control2: Point<f32>,
        color: gpui::Hsla,
        thickness: f32,
        segments: usize,
    ) {
        let mut previous_point = from_pos;
        for index in 1..=segments {
            let t = index as f32 / segments as f32;
            let current_point = Self::bezier_point(from_pos, control1, control2, to_pos, t);

            let dx = current_point.x - previous_point.x;
            let dy = current_point.y - previous_point.y;
            let len = (dx * dx + dy * dy).sqrt();

            if len > 0.1 {
                let px_offset = -dy / len * thickness / 2.0;
                let py_offset = dx / len * thickness / 2.0;

                let mut builder = gpui::PathBuilder::fill();
                builder.move_to(gpui::point(
                    gpui::px(previous_point.x + px_offset),
                    gpui::px(previous_point.y + py_offset),
                ));
                builder.line_to(gpui::point(
                    gpui::px(current_point.x + px_offset),
                    gpui::px(current_point.y + py_offset),
                ));
                builder.line_to(gpui::point(
                    gpui::px(current_point.x - px_offset),
                    gpui::px(current_point.y - py_offset),
                ));
                builder.line_to(gpui::point(
                    gpui::px(previous_point.x - px_offset),
                    gpui::px(previous_point.y - py_offset),
                ));
                builder.close();

                if let Ok(path) = builder.build() {
                    window.paint_path(path, color);
                }
            }

            previous_point = current_point;
        }
    }

    /// Calculate a point on a cubic bezier curve
    pub fn bezier_point(
        p0: Point<f32>,
        p1: Point<f32>,
        p2: Point<f32>,
        p3: Point<f32>,
        t: f32,
    ) -> Point<f32> {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        Point::new(
            uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
            uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
        )
    }

    /// Legacy bezier connection rendering (for compatibility)
    pub fn render_bezier_connection(
        from_pos: Point<f32>,
        to_pos: Point<f32>,
        color: gpui::Hsla,
        _cx: &mut Context<BlueprintEditorPanel>,
    ) -> AnyElement {
        let distance = (to_pos.x - from_pos.x).abs();
        let control_offset = (distance * 0.4).max(50.0).min(150.0);
        let control1 = Point::new(from_pos.x + control_offset, from_pos.y);
        let control2 = Point::new(to_pos.x - control_offset, to_pos.y);

        // Render as a thicker curve using overlapping circles for better visibility
        let segments = 40;
        let mut line_segments = Vec::new();

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let point = Self::bezier_point(from_pos, control1, control2, to_pos, t);

            // Create a thicker line by using overlapping circles
            line_segments.push(
                div()
                    .absolute()
                    .left(px(point.x - 2.0))
                    .top(px(point.y - 2.0))
                    .w(px(4.0))
                    .h(px(4.0))
                    .bg(color)
                    .rounded_full(),
            );
        }

        div()
            .absolute()
            .inset_0()
            .children(line_segments)
            .into_any_element()
    }

    /// Check if a connection is visible in the current viewport
    fn is_connection_visible_simple(connection: &Connection, graph: &BlueprintGraph) -> bool {
        // A connection is visible if either of its nodes is visible
        let from_node = graph.nodes.iter().find(|n| n.id == connection.source_node);
        let to_node = graph.nodes.iter().find(|n| n.id == connection.target_node);

        match (from_node, to_node) {
            (Some(from), Some(to)) => {
                Self::is_node_visible_simple(from, graph) || Self::is_node_visible_simple(to, graph)
            }
            _ => false, // If either node doesn't exist, don't render the connection
        }
    }

    /// Check if a node is visible in the current viewport (simplified culling)
    fn is_node_visible_simple(node: &BlueprintNode, graph: &BlueprintGraph) -> bool {
        // Calculate node position in screen coordinates
        let node_screen_pos = Self::graph_to_screen_pos(node.position, graph);
        let node_screen_size = Size::new(
            node.size.width * graph.zoom_level,
            node.size.height * graph.zoom_level,
        );

        // Calculate the visible area based on the inverse of current pan/zoom
        // This creates a dynamic culling frustum that properly accounts for viewport transformations

        // Convert screen bounds back to graph space for accurate culling
        let screen_to_graph_origin = Self::screen_to_graph_pos(Point::new(px(0.0), px(0.0)), graph);
        let screen_to_graph_end =
            Self::screen_to_graph_pos(Point::new(px(3840.0), px(2160.0)), graph); // 4K bounds

        // Add generous padding in graph space to prevent premature culling
        let padding_in_graph_space = 200.0 / graph.zoom_level; // Padding scales with zoom

        let visible_left = screen_to_graph_origin.x - padding_in_graph_space;
        let visible_top = screen_to_graph_origin.y - padding_in_graph_space;
        let visible_right = screen_to_graph_end.x + padding_in_graph_space;
        let visible_bottom = screen_to_graph_end.y + padding_in_graph_space;

        // Check if node intersects with visible bounds in graph space
        let node_left = node.position.x;
        let node_top = node.position.y;
        let node_right = node.position.x + node.size.width;
        let node_bottom = node.position.y + node.size.height;

        !(node_left > visible_right
            || node_right < visible_left
            || node_top > visible_bottom
            || node_bottom < visible_top)
    }

    /// Calculate the position of a pin in screen coordinates
    pub fn calculate_pin_position(
        node: &BlueprintNode,
        pin_id: &str,
        is_input: bool,
        graph: &BlueprintGraph,
    ) -> Option<Point<f32>> {
        // Reroute nodes are a single dot at their graph position.
        if node.node_type == NodeType::Reroute {
            return Some(Self::graph_to_screen_pos(node.position, graph));
        }

        // These MUST match the values used in render_blueprint_node / render_node_pins.
        const HEADER_H: f32 = 27.0;
        const SEP_H: f32    =  1.0;
        const BODY_PAD: f32 =  8.0;
        const PIN_ROW_H: f32 = 16.0;
        const PIN_GAP: f32  =  4.0;

        let z   = graph.zoom_level;
        let nsp = Self::graph_to_screen_pos(node.position, graph);

        let row = if is_input {
            node.inputs.iter().position(|p| p.id == pin_id)?
        } else {
            node.outputs.iter().position(|p| p.id == pin_id)?
        };

        // Y: top of node → header → separator → body padding → row center
        let pin_y = nsp.y
            + (HEADER_H + SEP_H + BODY_PAD) * z
            + row as f32 * (PIN_ROW_H + PIN_GAP) * z
            + PIN_ROW_H * 0.5 * z;

        // X: input pins sit on the left edge, output pins on the right edge,
        // both inset by BODY_PAD so the circle center is inside the node.
        let pin_x = if is_input {
            nsp.x + BODY_PAD * z
        } else {
            nsp.x + (node.size.width - BODY_PAD) * z
        };

        Some(Point::new(pin_x, pin_y))
    }

    /// Get the color for a pin based on its data type
    pub fn get_pin_color(data_type: &DataType, _cx: &mut Context<BlueprintEditorPanel>) -> gpui::Hsla {
        // Use the new type system to generate pin colors
        let pin_style = data_type.generate_pin_style();
        // Convert RGB to HSLA using the proper GPUI color API
        let rgba = gpui::Rgba {
            r: pin_style.color.r,
            g: pin_style.color.g,
            b: pin_style.color.b,
            a: pin_style.color.a,
        };
        gpui::Hsla::from(rgba)
    }

    /// Convert graph coordinates to screen coordinates
    pub fn graph_to_screen_pos(graph_pos: Point<f32>, graph: &BlueprintGraph) -> Point<f32> {
        Point::new(
            (graph_pos.x + graph.pan_offset.x) * graph.zoom_level,
            (graph_pos.y + graph.pan_offset.y) * graph.zoom_level,
        )
    }

    /// Convert screen coordinates to graph coordinates
    pub fn screen_to_graph_pos(screen_pos: Point<Pixels>, graph: &BlueprintGraph) -> Point<f32> {
        Point::new(
            (screen_pos.x.as_f32() / graph.zoom_level) - graph.pan_offset.x,
            (screen_pos.y.as_f32() / graph.zoom_level) - graph.pan_offset.y,
        )
    }
}
