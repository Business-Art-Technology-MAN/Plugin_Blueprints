use crate::blueprint_types::{BlueprintNode, NodeType, Pin};
use crate::node_graph::NodeGraphRenderer;
use crate::panel::BlueprintEditorPanel;
use gpui::{prelude::*, Context, AnyElement};
use super::style;

impl NodeGraphRenderer {
    /// Renders a blueprint node with header, separator, and pin rows.
    pub fn render_blueprint_node(
        node: &BlueprintNode,
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> AnyElement {
        if node.node_type == NodeType::Reroute {
            return Self::render_reroute_node(node, panel, cx);
        }

        // ── Category color ────────────────────────────────────────────────────
        let ue_node_color = |node_type: &NodeType| match node_type {
            NodeType::Event         => gpui::Hsla { h: 0.00, s: 0.82, l: 0.38, a: 1.0 },
            NodeType::Logic         => gpui::Hsla { h: 0.61, s: 0.78, l: 0.40, a: 1.0 },
            NodeType::Math          => gpui::Hsla { h: 0.42, s: 0.68, l: 0.36, a: 1.0 },
            NodeType::Object        => gpui::Hsla { h: 0.10, s: 0.72, l: 0.38, a: 1.0 },
            NodeType::Reroute       => gpui::Hsla { h: 0.00, s: 0.00, l: 0.40, a: 1.0 },
            NodeType::MacroEntry    => gpui::Hsla { h: 0.76, s: 0.62, l: 0.36, a: 1.0 },
            NodeType::MacroExit     => gpui::Hsla { h: 0.76, s: 0.62, l: 0.36, a: 1.0 },
            NodeType::MacroInstance => gpui::Hsla { h: 0.76, s: 0.50, l: 0.28, a: 1.0 },
        };
        let node_color = if let Some(ref hex) = node.color {
            Self::parse_hex_color(hex).unwrap_or_else(|| ue_node_color(&node.node_type))
        } else {
            ue_node_color(&node.node_type)
        };

        // ── Geometry ──────────────────────────────────────────────────────────
        let z   = panel.graph.zoom_level;
        let screen = Self::graph_to_screen_pos(node.position, &panel.graph);
        let node_id = node.id.clone();
        let is_dragging = panel.dragging_node.as_ref() == Some(&node.id);
        let scaled_width = node.size.width * z;

        // ── Style ─────────────────────────────────────────────────────────────
        let body_bg          = style::body_bg();
        let title_bg         = style::title_bg(node_color);
        let border_color     = if node.is_selected { gpui::white() } else { style::idle_border() };
        let corner_r         = style::corner_radius(z);
        let header_shadow_grad = style::header_shadow_gradient();

        // Layout constants — MUST match calculate_pin_position above.
        const HEADER_H: f32 = 27.0;
        const SEP_H: f32    =  1.0;

        // ── Node card ─────────────────────────────────────────────────────────
        // The v_flex is the absolutely-positioned card — no wrapper div needed.
        // Height is content-driven so the pin area never gets clipped.
        v_flex()
            .absolute()
            .left(px(screen.x))
            .top(px(screen.y))
            .w(px(scaled_width))
            .bg(body_bg)
            .rounded(corner_r)
            .overflow_hidden()
            .border_color(border_color)
            .when(node.is_selected,  |s| s.border_2().shadow_2xl())
            .when(!node.is_selected, |s| s.border_1().shadow_md())
            .when(is_dragging,       |s| s.opacity(0.92))
            .cursor_pointer()

            // ── Header (confirmed working — do not modify) ────────────────────
            .child(
                div()
                    .w_full()
                    .h(px(HEADER_H * z))
                    .relative()
                    .overflow_hidden()
                    .corner_radii(gpui::Corners {
                        top_left: corner_r,
                        top_right: corner_r,
                        bottom_right: px(0.0),
                        bottom_left: px(0.0),
                    })
                    .bg(title_bg)
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .bg(header_shadow_grad)
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .h_full()
                            .px(px(10.0 * z))
                            .items_center()
                            .gap(px(6.0 * z))
                            .id(ElementId::Name(format!("node-header-{}", node.id).into()))
                            .child(
                                div()
                                    .text_size(px(12.0 * z))
                                    .text_color(gpui::Hsla { h: 0.0, s: 0.0, l: 0.92, a: 1.0 })
                                    .child(node.icon.clone()),
                            )
                            .child(
                                div()
                                    .px(px(5.0 * z))
                                    .py(px(1.5 * z))
                                    .rounded(px(3.0 * z))
                                    .bg(style::title_pill_bg())
                                    .text_size(px(14.0 * z))
                                    .font_semibold()
                                    .text_color(gpui::white())
                                    .child(node.title.clone()),
                            )
                            .when(node.definition_id.starts_with("subgraph:"), |s| {
                                s.child(
                                    div()
                                        .px(px(4.0 * z))
                                        .py(px(1.0 * z))
                                        .rounded(px(3.0 * z))
                                        .bg(gpui::Rgba { r: 0.55, g: 0.30, b: 0.70, a: 0.45 })
                                        .border_1()
                                        .border_color(gpui::Rgba { r: 0.70, g: 0.50, b: 0.85, a: 0.75 })
                                        .text_size(px(9.0 * z))
                                        .text_color(gpui::Rgba { r: 0.90, g: 0.80, b: 1.0, a: 1.0 })
                                        .child("MACRO"),
                                )
                            })
                            .on_mouse_down(gpui::MouseButton::Left, {
                                let node_id = node_id.clone();
                                let node_definition_id = node.definition_id.clone();
                                let node_title = node.title.clone();
                                cx.listener(move |panel, event: &MouseDownEvent, window, cx| {
                                    cx.stop_propagation();
                                    panel.focus_handle().focus(window);

                                    let now = std::time::Instant::now();
                                    let is_subgraph = node_definition_id.starts_with("subgraph:");
                                    let should_open_subgraph = is_subgraph && {
                                        if let (Some(last_t), Some(last_p)) = (panel.last_click_time, panel.last_click_pos) {
                                            if now.duration_since(last_t).as_millis() < 500 {
                                                let ep = Self::window_to_graph_element_pos(event.position, panel);
                                                let cp = gpui::Point::new(ep.x.as_f32(), ep.y.as_f32());
                                                ((cp.x - last_p.x).powi(2) + (cp.y - last_p.y).powi(2)).sqrt() < 10.0
                                            } else { false }
                                        } else { false }
                                    };

                                    if should_open_subgraph {
                                        let subgraph_id = node_definition_id
                                            .strip_prefix("subgraph:")
                                            .unwrap_or(&node_definition_id)
                                            .to_string();
                                        if let Some(library_id) = panel.get_macro_library_id(&subgraph_id) {
                                            let library_name = panel.library_manager.get_libraries()
                                                .get(&library_id)
                                                .map(|lib| lib.name.clone())
                                                .unwrap_or_else(|| library_id.clone());
                                            panel.request_open_engine_library(library_id, library_name, Some(subgraph_id.clone()), Some(node_title.clone()), cx);
                                        } else if let Some(m) = panel.local_macros.iter().find(|m| m.id == subgraph_id) {
                                            panel.open_local_macro(subgraph_id.clone(), m.name.clone(), cx);
                                        } else {
                                            tracing::info!("Macro '{}' not found", node_title);
                                        }
                                        panel.last_click_time = None;
                                        panel.last_click_pos  = None;
                                    } else {
                                        if !panel.graph.selected_nodes.contains(&node_id) {
                                            panel.select_node(Some(node_id.clone()), cx);
                                        }
                                        let ep = Self::window_to_graph_element_pos(event.position, panel);
                                        let gp = Self::screen_to_graph_pos(ep, &panel.graph);
                                        panel.start_drag(node_id.clone(), gp, cx);
                                        panel.last_click_time = Some(now);
                                        panel.last_click_pos  = Some(gpui::Point::new(ep.x.as_f32(), ep.y.as_f32()));
                                    }
                                })
                            }),
                    ),
            )

            // ── Separator ─────────────────────────────────────────────────────
            .child(
                div()
                    .w_full()
                    .h(px(SEP_H * z))
                    .bg(style::separator_bg()),
            )

            // ── Pin body ──────────────────────────────────────────────────────
            .child(Self::render_node_pins(node, z, panel, cx))

            // ── Body mouse handler (select on click) ──────────────────────────
            .on_mouse_down(gpui::MouseButton::Left, {
                let node_id = node_id.clone();
                cx.listener(move |panel, _event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    panel.focus_handle().focus(window);
                    if !panel.graph.selected_nodes.contains(&node_id) {
                        panel.select_node(Some(node_id.clone()), cx);
                    }
                })
            })
            .into_any_element()
    }

    pub fn render_reroute_node(
        node: &BlueprintNode,
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> AnyElement {
        let graph_pos = Self::graph_to_screen_pos(node.position, &panel.graph);
        let node_id = node.id.clone();
        let is_dragging = panel.dragging_node.as_ref() == Some(&node.id);

        // Get the color from the pin data type (reroute nodes have one input and one output of the same type)
        let pin_color = if let Some(input_pin) = node.inputs.first() {
            Self::get_pin_color(&input_pin.data_type, cx)
        } else if let Some(output_pin) = node.outputs.first() {
            Self::get_pin_color(&output_pin.data_type, cx)
        } else {
            cx.theme().accent
        };

        // Reroute node is rendered as a thick colored dot
        let dot_size = 16.0 * panel.graph.zoom_level;
        let clickable_size = 24.0 * panel.graph.zoom_level; // Larger clickable area

        div()
            .absolute()
            .left(px(graph_pos.x - clickable_size / 2.0)) // Center the clickable area
            .top(px(graph_pos.y - clickable_size / 2.0))
            .w(px(clickable_size))
            .h(px(clickable_size))
            .cursor_pointer()
            .on_mouse_down(gpui::MouseButton::Left, {
                let node_id = node_id.clone();
                cx.listener(move |panel, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    panel.focus_handle().focus(window);

                    if !panel.graph.selected_nodes.contains(&node_id) {
                        panel.select_node(Some(node_id.clone()), cx);
                    }

                    let ep = Self::window_to_graph_element_pos(event.position, panel);
                    let gp = Self::screen_to_graph_pos(ep, &panel.graph);
                    panel.start_drag(node_id.clone(), gp, cx);
                })
            })
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .bg(pin_color)
                    .when(is_dragging, |s| s.opacity(0.8))
                    .shadow_md()
            )
            // Invisible pins for connections - positioned at the center
            .children(node.inputs.iter().map(|input_pin| {
                Self::render_reroute_pin(input_pin, true, &node.id, panel, cx)
            }))
            .children(node.outputs.iter().map(|output_pin| {
                Self::render_reroute_pin(output_pin, false, &node.id, panel, cx)
            }))
            .into_any_element()
    }

    pub fn render_reroute_pin(
        pin: &Pin,
        is_input: bool,
        node_id: &str,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let node_id_clone = node_id.to_string();
        let pin_id = pin.id.clone();

        // Check if this pin is compatible with the current drag
        let is_compatible = if let Some(ref drag) = panel.dragging_connection {
            is_input && node_id != drag.source_node && pin.data_type.is_compatible_with(&drag.source_pin_type)
        } else {
            false
        };

        // Invisible pin area at the center of the dot for connections
        div()
            .absolute()
            .inset_0()
            .when(is_compatible, |s| s.bg(gpui::Hsla { h: 0.0, s: 0.0, l: 1.0, a: 0.2 }))
            .on_mouse_down(gpui::MouseButton::Left, {
                cx.listener(move |panel, event: &MouseDownEvent, _window, cx| {
                    cx.stop_propagation();
                    let source_pin_type = panel.graph.nodes
                        .iter()
                        .find(|n| n.id == node_id_clone)
                        .and_then(|n| {
                            if is_input {
                                n.inputs.iter().find(|p| p.id == pin_id).map(|p| p.data_type.clone())
                            } else {
                                n.outputs.iter().find(|p| p.id == pin_id).map(|p| p.data_type.clone())
                            }
                        });

                    if let Some(source_pin_type) = source_pin_type {
                        if let Some(ref drag) = panel.dragging_connection {
                            if is_input && node_id_clone != drag.source_node && source_pin_type.is_compatible_with(&drag.source_pin_type) {
                                panel.complete_connection(
                                    drag.source_node.clone(),
                                    drag.source_pin_id.clone(),
                                    node_id_clone.clone(),
                                    pin_id.clone(),
                                    cx,
                                );
                            }
                        }
                    }
                })
            })
    }

    /// Renders all pin rows for a node body.
    /// Layout constants must stay in sync with `calculate_pin_position`.
    pub fn render_node_pins(
        node: &BlueprintNode,
        z: f32,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        const BODY_PAD: f32  = 8.0;
        const PIN_ROW_H: f32 = 16.0;
        const PIN_GAP: f32   = 4.0;
        const PIN_SIZE: f32  = 12.0;

        let label_color = style::label_color();
        let corner_r = style::corner_radius(z);
        let max_rows = node.inputs.len().max(node.outputs.len());

        div()
            .w_full()
            .bg(gpui::Hsla { h: 0.0, s: 0.0, l: 0.08, a: 1.0 })
            .corner_radii(gpui::Corners {
                top_left: px(0.0),
                top_right: px(0.0),
                bottom_right: corner_r,
                bottom_left: corner_r,
            })
            .px(px(BODY_PAD * z))
            .pt(px(BODY_PAD * z))
            .pb(px(BODY_PAD * z))
            .flex()
            .flex_col()
            .gap(px(PIN_GAP * z))
            .children((0..max_rows).map(|i| {
                div()
                    .w_full()
                    .h(px(PIN_ROW_H * z))
                    .flex()
                    .items_center()
                    // ── Left: input pin + label ───────────────────────────────
                    .child({
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0 * z))
                            .child(if let Some(pin) = node.inputs.get(i) {
                                Self::render_pin(pin, true, &node.id, panel, cx).into_any_element()
                            } else {
                                div().w(px(PIN_SIZE * z)).h(px(PIN_SIZE * z)).into_any_element()
                            })
                            .child(if let Some(pin) = node.inputs.get(i) {
                                if !pin.name.is_empty() {
                                    div()
                                        .text_size(px(11.0 * z))
                                        .text_color(label_color)
                                        .child(pin.name.clone())
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            } else {
                                div().into_any_element()
                            })
                    })
                    // ── Centre spacer ─────────────────────────────────────────
                    .child(div().flex_1())
                    // ── Right: label + output pin ─────────────────────────────
                    .child({
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0 * z))
                            .child(if let Some(pin) = node.outputs.get(i) {
                                if !pin.name.is_empty() {
                                    div()
                                        .text_size(px(11.0 * z))
                                        .text_color(label_color)
                                        .child(pin.name.clone())
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            } else {
                                div().into_any_element()
                            })
                            .child(if let Some(pin) = node.outputs.get(i) {
                                Self::render_pin(pin, false, &node.id, panel, cx).into_any_element()
                            } else {
                                div().w(px(PIN_SIZE * z)).h(px(PIN_SIZE * z)).into_any_element()
                            })
                    })
            }))
    }

    pub fn render_pin(
        pin: &Pin,
        is_input: bool,
        node_id: &str,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let pin_style = pin.data_type.generate_pin_style();
        let pin_color = Self::get_pin_color(&pin.data_type, cx);
        let node_id = node_id.to_string();
        let pin_id = pin.id.clone();
        let z = panel.graph.zoom_level;

        const PIN_SIZE: f32 = 12.0;

        // Check if this pin is compatible with the current drag
        let is_compatible = if let Some(ref drag) = panel.dragging_connection {
            !is_input && node_id != drag.source_node && pin.data_type.is_compatible_with(&drag.source_pin_type)
        } else {
            false
        };

        match pin_style {
            crate::blueprint_types::PinStyle::Circle => {
                div()
                    .w(px(PIN_SIZE * z))
                    .h(px(PIN_SIZE * z))
                    .rounded_full()
                    .bg(pin_color)
                    .when(is_compatible, |s| s.border_2().border_color(gpui::yellow()))
                    .shadow_sm()
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, {
                        cx.listener(move |panel, _event: &MouseDownEvent, _window, cx| {
                            panel.start_connection_drag(
                                node_id.clone(),
                                pin_id.clone(),
                                pin.data_type.clone(),
                                is_input,
                                cx,
                            );
                        })
                    })
                    .into_any_element()
            }
            crate::blueprint_types::PinStyle::Execution => {
                div()
                    .w(px(PIN_SIZE * z))
                    .h(px(PIN_SIZE * z))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .absolute()
                            .w(px(PIN_SIZE * 0.6 * z))
                            .h(px(PIN_SIZE * 0.6 * z))
                            .bg(pin_color)
                            .clip_path("polygon(50% 0%, 100% 38%, 82% 100%, 50% 77%, 18% 100%, 0% 38%)")
                    )
                    .when(is_compatible, |s| s.border_2().border_color(gpui::yellow()))
                    .shadow_sm()
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, {
                        cx.listener(move |panel, _event: &MouseDownEvent, _window, cx| {
                            panel.start_connection_drag(
                                node_id.clone(),
                                pin_id.clone(),
                                pin.data_type.clone(),
                                is_input,
                                cx,
                            );
                        })
                    })
                    .into_any_element()
            }
        }
    }
}
