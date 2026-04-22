use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui::prelude::*;
use ui::{Colorize, PixelsExt};
use ui::{button::{Button, ButtonVariants}, h_flex, v_flex, ActiveTheme as _, IconName, Sizable, StyledExt, tooltip::Tooltip};

use crate::node_rendering::{layout, style};
use super::panel::BlueprintEditorPanel;
use super::{BlueprintNode, BlueprintGraph, Pin, NodeType, Connection};
use ui::graph::DataType;

pub struct NodeGraphRenderer;

/// Helper to create simple text tooltip for pins (still using gpui's built-in tooltip)
fn create_text_tooltip(text: &'static str) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    move |window, cx| {
        Tooltip::new(text).build(window, cx)
    }
}

impl NodeGraphRenderer {
    pub fn render(
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let focus_handle = panel.focus_handle().clone();

        let graph_id = "blueprint-graph";
        let panel_entity = cx.entity().clone();

        div()
            .size_full()
            .flex() // Enable flexbox
            .flex_col() // Column direction
            .relative()
            .bg(cx.theme().muted.opacity(0.1))
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius)
            .overflow_hidden()
            .track_focus(&focus_handle)
            .key_context("BlueprintGraph")
            .on_children_prepainted({
                let panel_entity = panel_entity.clone();
                move |children_bounds, _window, cx| {
                    // children_bounds are in WINDOW coordinates!
                    // Calculate the bounding box of all children to get our element's window-relative bounds
                    if !children_bounds.is_empty() {
                        let mut min_x = f32::MAX;
                        let mut min_y = f32::MAX;
                        let mut max_x = f32::MIN;
                        let mut max_y = f32::MIN;

                        for child_bounds in &children_bounds {
                            min_x = min_x.min(child_bounds.origin.x.as_f32());
                            min_y = min_y.min(child_bounds.origin.y.as_f32());
                            max_x = max_x.max((child_bounds.origin.x + child_bounds.size.width).as_f32());
                            max_y = max_y.max((child_bounds.origin.y + child_bounds.size.height).as_f32());
                        }

                        let origin = gpui::Point { x: px(min_x), y: px(min_y) };
                        let size = gpui::Size {
                            width: px(max_x - min_x),
                            height: px(max_y - min_y),
                        };

                        // Store the graph element's bounds derived from children (which are in window coords)
                        panel_entity.update(cx, |panel, _cx| {
                            panel.graph_element_bounds = Some(gpui::Bounds { origin, size });
                        });
                    }
                }
            })
            .id(graph_id)
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |panel, event, window, cx| {
                // Focus on click to enable keyboard events
                panel.focus_handle().focus(window);

                // If editing a comment, clicking outside should save and exit edit mode
                if panel.editing_comment.is_some() {
                    panel.finish_comment_editing(cx);
                }



                // Close variable drop menu if it's open
                if panel.variable_drop_menu_position.is_some() {
                    panel.variable_drop_menu_position = None;
                    cx.notify();
                }
            }))
            .child(Self::render_comments(panel, cx))
            .child(Self::render_connections(panel, cx))
            .child(Self::render_nodes(panel, cx))
            .child(Self::render_selection_box(panel, cx))
            .child(Self::render_viewport_bounds_debug(panel, cx))
            .when(panel.show_debug_overlay, |this| {
                this.child(Self::render_debug_overlay(panel, cx))
            })
            .when(panel.show_graph_controls, |this| {
                this.child(Self::render_graph_controls(panel, cx))
            })
            .when(panel.show_minimap, |this| {
                this.child(super::minimap::MinimapRenderer::render(panel, cx))
            })
            .on_mouse_down(
                gpui::MouseButton::Right,
                cx.listener(|panel, event: &MouseDownEvent, _window, cx| {
                    // Convert window coordinates to element coordinates
                    let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                    let mouse_pos = Point::new(element_pos.x.as_f32(), element_pos.y.as_f32());

                    // Store right-click start position for gesture detection
                    if panel.dragging_connection.is_none() && panel.dragging_node.is_none() {
                        panel.right_click_start = Some(mouse_pos);
                        // Don't show menu immediately - wait for mouse up or movement
                    }
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|panel, event: &MouseDownEvent, _window, cx| {
                    // Debug: Print raw event position and calculated offset
                    tracing::info!("[MOUSE] Raw window position: x={}, y={}", event.position.x.as_f32(), event.position.y.as_f32());
                    tracing::info!("[MOUSE] Stored element bounds: {:?}", panel.graph_element_bounds);

                    // Convert window-relative coordinates to element-relative coordinates
                    let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                    tracing::info!("[MOUSE] Calculated element-relative position: x={}, y={}", element_pos.x.as_f32(), element_pos.y.as_f32());

                    // Expected: if you click at the top-left corner of the graph, element_pos should be close to (0, 0)
                    // If not, our offset is wrong!

                    // Convert element coordinates to graph coordinates
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                    let mouse_pos = Point::new(element_pos.x.as_f32(), element_pos.y.as_f32());

                    tracing::info!("[MOUSE] Converted to graph pos: x={}, y={}", graph_pos.x, graph_pos.y);
                    tracing::info!("[MOUSE] Pan offset: x={}, y={}", panel.graph.pan_offset.x, panel.graph.pan_offset.y);
                    tracing::info!("[MOUSE] Zoom level: {}", panel.graph.zoom_level);

                    // Node picker handles its own dismissal

                    // Check if clicking on a node (check ALL nodes, not just rendered ones)
                    let clicked_node = panel.graph.nodes.iter().find(|node| {
                        let node_left = node.position.x;
                        let node_top = node.position.y;
                        let node_right = node.position.x + node.size.width;
                        let node_bottom = node.position.y + node.size.height;

                        let is_inside = graph_pos.x >= node_left
                            && graph_pos.x <= node_right
                            && graph_pos.y >= node_top
                            && graph_pos.y <= node_bottom;

                        if is_inside {
                            tracing::info!("[MOUSE] Clicked on node '{}' at graph pos ({}, {})", node.title, node.position.x, node.position.y);
                        }

                        is_inside
                    });

                    if let Some(node) = clicked_node {
                        // Only change selection if this node is not already selected
                        // This allows dragging multiple selected nodes
                        if !panel.graph.selected_nodes.contains(&node.id) {
                            panel.select_node(Some(node.id.clone()), cx);
                        }
                    } else {
                        
                        // Check for double-click on connection (for creating reroute nodes)
                        let handled_double_click = panel.handle_empty_space_click(graph_pos, cx);

                        // Only start selection drag if we didn't handle a double-click
                        if !handled_double_click {
                            // Don't clear selection immediately - only when dragging or on mouse up
                            panel.start_selection_drag(graph_pos, event.modifiers.control, cx);
                        }
                    }
                }),
            )
            .on_mouse_move(cx.listener(|panel, event: &MouseMoveEvent, _window, cx| {
                // Convert window coordinates to element coordinates
                let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                let mouse_pos = Point::new(element_pos.x.as_f32(), element_pos.y.as_f32());

                // Check if right-click drag should start panning
                if let Some(right_start) = panel.right_click_start {
                    let distance = ((mouse_pos.x - right_start.x).powi(2) + (mouse_pos.y - right_start.y).powi(2)).sqrt();
                    if distance > panel.right_click_threshold {
                        // Start panning if we've moved beyond threshold
                        panel.start_panning(right_start, cx);
                        panel.right_click_start = None; // Clear the right-click state

                    }
                }

                if panel.dragging_comment.is_some() {
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                    panel.update_comment_drag(graph_pos, cx);
                } else if panel.resizing_comment.is_some() {
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                    panel.update_comment_resize(graph_pos, cx);
                } else if panel.dragging_node.is_some() {
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                    panel.update_drag(graph_pos, cx);
                } else if panel.dragging_connection.is_some() {
                    // Update mouse position for drag line rendering
                    panel.update_connection_drag(mouse_pos, cx);
                } else if panel.is_selecting() {
                    // Update selection drag
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                    panel.update_selection_drag(graph_pos, cx);
                } else if panel.is_panning() && panel.dragging_node.is_none() {
                    // Only update panning if we're not dragging a node
                    panel.update_pan(mouse_pos, cx);
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|panel, event: &MouseUpEvent, _window, cx| {
                    if panel.dragging_comment.is_some() {
                        panel.end_comment_drag(cx);
                    } else if panel.resizing_comment.is_some() {
                        panel.end_comment_resize(cx);
                    } else if panel.dragging_node.is_some() {
                        panel.end_drag(cx);
                    } else if panel.dragging_variable.is_some() {
                        // Variable dropped on canvas - show Get/Set context menu
                        let graph_pos = Self::screen_to_graph_pos(event.position, &panel.graph);
                        panel.finish_dragging_variable(graph_pos, cx);
                    } else if panel.dragging_connection.is_some() {
                        // Show node creation menu when dropping connection on empty space
                        let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                        let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                        panel.show_node_picker(graph_pos, _window, cx);
                        panel.cancel_connection_drag(cx);
                    } else if panel.is_selecting() {
                        // End selection drag
                        panel.end_selection_drag(cx);
                    } else if panel.is_panning() {
                        panel.end_panning(cx);
                    }
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Right,
                cx.listener(|panel, event: &MouseUpEvent, _window, cx| {
                    if panel.is_panning() {
                        panel.end_panning(cx);
                    } else if panel.right_click_start.is_some() {
                        // Right-click released without dragging - show context menu
                        panel.right_click_start = None;
                        let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                        let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);
                        
                        panel.show_node_picker(graph_pos, _window, cx);
                    }
                }),
            )
            .on_scroll_wheel(cx.listener(|panel, event: &ScrollWheelEvent, _window, cx| {
                // Zoom with scroll wheel
                let delta_y = match event.delta {
                    ScrollDelta::Pixels(p) => p.y.as_f32(),
                    ScrollDelta::Lines(l) => l.y * 20.0, // Convert lines to pixels
                };

                // Perform zoom centered on the mouse
                // Convert to element coordinates first
                let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                panel.handle_zoom(delta_y, element_pos, cx);
            }))
            .on_key_down(cx.listener(|panel, event: &KeyDownEvent, window, cx| {
                tracing::info!("Key pressed: {:?}", event.keystroke.key);

                let key_lower = event.keystroke.key.to_lowercase();

                if panel.editing_comment.is_some() {
                    // Handle comment editing keys
                    if key_lower == "escape" {
                        // Cancel editing without saving
                        panel.editing_comment = None;
                        cx.notify();
                    } else if key_lower == "enter" && event.keystroke.modifiers.control {
                        // Ctrl+Enter saves the comment
                        panel.finish_comment_editing(cx);
                    }
                } else if key_lower == "escape" {
                    // Escape key dismisses menus and cancels operations
                    if panel.variable_drop_menu_position.is_some() {
                        panel.variable_drop_menu_position = None;
                        cx.notify();
                    } else if panel.dragging_connection.is_some() {
                        panel.cancel_connection_drag(cx);
                    }
                } else if key_lower == "delete" || key_lower == "backspace" {
                    tracing::info!(
                        "Delete key pressed! Selected nodes: {:?}",
                        panel.graph.selected_nodes
                    );
                    panel.delete_selected_nodes(cx);
                } else if key_lower == "c" && event.keystroke.modifiers.control {
                    // Ctrl+C creates a new comment
                    panel.create_comment_at_center(window, cx);
                }
            }))
    }


    /// # WARNING!
    /// 
    /// For reasons uninvestigated this causes EXTREME performance degradation at some zoom levels
    fn render_grid_background(panel: &BlueprintEditorPanel, cx: &mut Context<BlueprintEditorPanel>) -> impl IntoElement {
        // Multi-scale grid system that shows/hides based on zoom level
        // Grid scales: 50px (fine), 200px (medium), 1000px (coarse)
        let zoom = panel.graph.zoom_level;
        let pan = &panel.graph.pan_offset;

        // Define grid scales and their visibility thresholds
        let grids = [
            (50.0, 0.5, 1.5, 0.15),   // Fine grid: visible between 0.5x and 1.5x zoom, low opacity
            (200.0, 0.3, 2.0, 0.25),  // Medium grid: visible between 0.3x and 2.0x zoom
            (1000.0, 0.1, 10.0, 0.35), // Coarse grid: always visible, higher opacity
        ];

        let mut grid_layers = Vec::new();

        for (grid_size, min_zoom, max_zoom, base_opacity) in grids {
            // Skip grids outside their zoom range
            if zoom < min_zoom || zoom > max_zoom {
                continue;
            }

            // Fade in/out at edges of zoom range
            let fade_range = 0.2_f32;
            let fade_in = ((zoom - min_zoom) / (min_zoom * fade_range)).min(1.0_f32);
            let fade_out = ((max_zoom - zoom) / (max_zoom * fade_range)).min(1.0_f32);
            let fade = fade_in.min(fade_out).max(0.0_f32);
            let opacity = base_opacity * fade;

            if opacity > 0.01 {
                grid_layers.push(Self::render_grid_layer(grid_size, opacity, pan, zoom, cx));
            }
        }

        div().absolute().inset_0()
            .bg(cx.theme().muted.opacity(0.05))
            .children(grid_layers)
    }

    fn render_grid_layer(
        grid_size: f32,
        opacity: f32,
        pan: &Point<f32>,
        zoom: f32,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        // Calculate visible grid range
        let scaled_grid_size = grid_size * zoom;

        // Calculate grid offset based on pan
        let offset_x = (pan.x * zoom) % scaled_grid_size;
        let offset_y = (pan.y * zoom) % scaled_grid_size;

        // Render grid dots
        let viewport_width = 3840.0;
        let viewport_height = 2160.0;

        let grid_color = cx.theme().border.opacity(opacity);
        let dot_size = 2.0;

        let mut dots = Vec::new();

        // Calculate number of grid lines needed
        let num_cols = (viewport_width / scaled_grid_size).ceil() as i32 + 2;
        let num_rows = (viewport_height / scaled_grid_size).ceil() as i32 + 2;

        for col in 0..num_cols {
            for row in 0..num_rows {
                let x = offset_x + (col as f32 * scaled_grid_size);
                let y = offset_y + (row as f32 * scaled_grid_size);

                if x >= -scaled_grid_size && x <= viewport_width + scaled_grid_size
                    && y >= -scaled_grid_size && y <= viewport_height + scaled_grid_size {
                    dots.push(
                        div()
                            .absolute()
                            .left(px(x - dot_size / 2.0))
                            .top(px(y - dot_size / 2.0))
                            .w(px(dot_size))
                            .h(px(dot_size))
                            .bg(grid_color)
                            .rounded_full()
                    );
                }
            }
        }

        div()
            .absolute()
            .inset_0()
            .children(dots)
    }

    fn render_comments(
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let visible_comments: Vec<super::BlueprintComment> = panel
            .graph
            .comments
            .iter()
            .map(|comment| {
                let mut comment = comment.clone();
                comment.is_selected = panel.graph.selected_comments.contains(&comment.id);
                comment
            })
            .collect();

        div().absolute().inset_0().children(
            visible_comments
                .into_iter()
                .map(|comment| Self::render_comment(&comment, panel, cx)),
        )
    }

    fn render_comment(
        comment: &super::BlueprintComment,
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> AnyElement {
        let graph_pos = Self::graph_to_screen_pos(comment.position, &panel.graph);
        let comment_id = comment.id.clone();
        let is_dragging = panel.dragging_comment.as_ref() == Some(&comment.id);
        let is_resizing = panel.resizing_comment.as_ref().map(|(id, _)| id) == Some(&comment.id);

        // Scale comment size with zoom level
        let scaled_width = comment.size.width * panel.graph.zoom_level;
        let scaled_height = comment.size.height * panel.graph.zoom_level;

        let resize_handle_size = 12.0 * panel.graph.zoom_level;

        div()
            .absolute()
            .left(px(graph_pos.x))
            .top(px(graph_pos.y))
            .w(px(scaled_width))
            .h(px(scaled_height))
            .child(
                div()
                    .size_full()
                    .bg(comment.color)
                    .border_2()
                    .border_color(if comment.is_selected {
                        gpui::yellow()
                    } else {
                        comment.color.lighten(0.2)
                    })
                    .rounded(px(8.0 * panel.graph.zoom_level))
                    .when(is_dragging || is_resizing, |style| style.opacity(0.8))
                    .shadow_md()
                    .overflow_hidden()
                    .child({
                        let is_editing = panel.editing_comment.as_ref() == Some(&comment.id);

                        if is_editing {
                            // Show text input for editing
                            div()
                                .p(px(12.0 * panel.graph.zoom_level))
                                .size_full()
                                .font_family("JetBrainsMono-Regular")
                                .font_weight(gpui::FontWeight::default())
                                .child(
                                    ui::input::TextInput::new(&panel.comment_text_input)
                                )
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|_panel, _event: &MouseDownEvent, _window, cx| {
                                    cx.stop_propagation();
                                }))
                                .on_mouse_move(cx.listener(|_panel, _event: &MouseMoveEvent, _window, cx| {
                                    cx.stop_propagation();
                                }))
                                .into_any_element()
                        } else {
                            // Show static text
                            div()
                                .p(px(12.0 * panel.graph.zoom_level))
                                .size_full()
                                .text_size(px(14.0 * panel.graph.zoom_level))
                                .text_color(gpui::white())
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(comment.text.clone())
                                .on_mouse_down(gpui::MouseButton::Left, {
                                    let comment_id = comment_id.clone();
                                    cx.listener(move |panel, event: &MouseDownEvent, _window, cx| {
                                        cx.stop_propagation();

                                        // Select comment
                                        if !panel.graph.selected_comments.contains(&comment_id) {
                                            panel.graph.selected_comments.clear();
                                            panel.graph.selected_comments.push(comment_id.clone());
                                        }

                                        // Check for double-click to start editing
                                        let now = std::time::Instant::now();
                                        let should_edit = if let Some(last_click) = panel.last_click_time {
                                            if now.duration_since(last_click).as_millis() < 500 {
                                                if let Some(last_pos) = panel.last_click_pos {
                                                    let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                                                    let current_pos = Point::new(element_pos.x.as_f32(), element_pos.y.as_f32());
                                                    let distance = ((current_pos.x - last_pos.x).powi(2) + (current_pos.y - last_pos.y).powi(2)).sqrt();
                                                    distance < 10.0
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        };

                                        if should_edit {
                                            // Start editing
                                            panel.editing_comment = Some(comment_id.clone());

                                            // Load current comment text into input
                                            if let Some(comment) = panel.graph.comments.iter().find(|c| c.id == comment_id) {
                                                panel.comment_text_input.update(cx, |state, cx| {
                                                    state.set_value(comment.text.clone(), _window, cx);
                                                });
                                            }

                                            panel.last_click_time = None;
                                        } else {
                                            // Start dragging
                                            let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                                            let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);

                                            // Calculate drag offset (same as node dragging)
                                            if let Some(comment) = panel.graph.comments.iter().find(|c| c.id == comment_id) {
                                                panel.dragging_comment = Some(comment_id.clone());
                                                panel.drag_offset = Point::new(
                                                    graph_pos.x - comment.position.x,
                                                    graph_pos.y - comment.position.y,
                                                );
                                            }

                                            // Update click tracking
                                            let current_pos = Point::new(element_pos.x.as_f32(), element_pos.y.as_f32());
                                            panel.last_click_time = Some(now);
                                            panel.last_click_pos = Some(current_pos);
                                        }

                                        cx.notify();
                                    })
                                })
                                .into_any_element()
                        }
                    })
                    // Resize handles
                    .children([
                        Self::render_resize_handle(super::panel::ResizeHandle::TopLeft, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::TopRight, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::BottomLeft, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::BottomRight, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::Top, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::Bottom, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::Left, &comment_id, resize_handle_size, panel, cx),
                        Self::render_resize_handle(super::panel::ResizeHandle::Right, &comment_id, resize_handle_size, panel, cx),
                    ])
                    // Color picker button (only when selected)
                    .when(comment.is_selected, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(8.0 * panel.graph.zoom_level))
                                .right(px(8.0 * panel.graph.zoom_level))
                                .child(
                                    ui::color_picker::ColorPicker::new(
                                        comment.color_picker_state.as_ref().expect("Color picker state")
                                    )
                                    .size(ui::Size::Small)
                                )
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|_panel, _event: &MouseDownEvent, _window, cx| {
                                    cx.stop_propagation();
                                }))
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_resize_handle(
        handle: super::panel::ResizeHandle,
        comment_id: &str,
        size: f32,
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let (left, top, cursor) = match handle {
            super::panel::ResizeHandle::TopLeft => (Some(px(0.0)), Some(px(0.0)), CursorStyle::ResizeUpLeftDownRight),
            super::panel::ResizeHandle::TopRight => (None, Some(px(0.0)), CursorStyle::ResizeUpRightDownLeft),
            super::panel::ResizeHandle::BottomLeft => (Some(px(0.0)), None, CursorStyle::ResizeUpRightDownLeft),
            super::panel::ResizeHandle::BottomRight => (None, None, CursorStyle::ResizeUpLeftDownRight),
            super::panel::ResizeHandle::Top => (None, Some(px(0.0)), CursorStyle::ResizeUpDown),
            super::panel::ResizeHandle::Bottom => (None, None, CursorStyle::ResizeUpDown),
            super::panel::ResizeHandle::Left => (Some(px(0.0)), None, CursorStyle::ResizeLeftRight),
            super::panel::ResizeHandle::Right => (None, None, CursorStyle::ResizeLeftRight),
        };

        let comment_id = comment_id.to_string();

        div()
            .absolute()
            .when_some(left, |this, l| this.left(l))
            .when(left.is_none(), |this| this.right(px(0.0)))
            .when_some(top, |this, t| this.top(t))
            .when(top.is_none(), |this| this.bottom(px(0.0)))
            .when(matches!(handle, super::panel::ResizeHandle::Top | super::panel::ResizeHandle::Bottom), |this| {
                this.left_0().right_0().h(px(size))
            })
            .when(matches!(handle, super::panel::ResizeHandle::Left | super::panel::ResizeHandle::Right), |this| {
                this.top_0().bottom_0().w(px(size))
            })
            .when(!matches!(handle, super::panel::ResizeHandle::Top | super::panel::ResizeHandle::Bottom | super::panel::ResizeHandle::Left | super::panel::ResizeHandle::Right), |this| {
                this.size(px(size))
            })
            .bg(gpui::transparent_black())
            .cursor(cursor)
            .on_mouse_down(gpui::MouseButton::Left, {
                cx.listener(move |panel, event: &MouseDownEvent, _window, cx| {
                    cx.stop_propagation();

                    let element_pos = Self::window_to_graph_element_pos(event.position, panel);
                    let graph_pos = Self::screen_to_graph_pos(element_pos, &panel.graph);

                    panel.resizing_comment = Some((comment_id.clone(), handle.clone()));
                    panel.drag_offset = graph_pos;

                    cx.notify();
                })
            })
    }

    fn render_nodes(
        panel: &mut BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        let _render_start = std::time::Instant::now();

        // Only render nodes that are visible within the viewport (we'll calculate bounds in the element)
        let visible_nodes: Vec<BlueprintNode> = panel
            .graph
            .nodes
            .iter()
            .filter(|node| Self::is_node_visible_simple(node, &panel.graph))
            .map(|node| {
                let mut node = node.clone();
                node.is_selected = panel.graph.selected_nodes.contains(&node.id);
                node
            })
            .collect();

        // Note: We can't mutate panel here since it's borrowed immutably
        // Virtualization stats will be updated in a different way

        // Debug info for virtualization
        if cfg!(debug_assertions) && panel.graph.nodes.len() != visible_nodes.len() {
            tracing::info!(
                "[BLUEPRINT-VIRTUALIZATION] Rendering {} of {} nodes (saved {:.1}%)",
                visible_nodes.len(),
                panel.graph.nodes.len(),
                (1.0 - visible_nodes.len() as f32 / panel.graph.nodes.len() as f32) * 100.0
            );
        }

        div().absolute().inset_0().children(
            visible_nodes
                .into_iter()
                .map(|node| Self::render_blueprint_node(&node, panel, cx)),
        )
    }



    fn render_connections(
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

    fn build_dragging_connection_shape(
        drag: &super::panel::ConnectionDrag,
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

    fn paint_bezier_line(
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

    fn paint_bezier_stroke(
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

    fn get_pin_color(data_type: &DataType, _cx: &mut Context<BlueprintEditorPanel>) -> gpui::Hsla {
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

    fn calculate_pin_position(
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

    fn render_bezier_connection(
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

    fn bezier_point(
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

    fn render_selection_box(
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        if let (Some(start), Some(end)) = (panel.selection_start, panel.selection_end) {
            // Convert selection bounds to screen coordinates
            let start_screen = Self::graph_to_screen_pos(start, &panel.graph);
            let end_screen = Self::graph_to_screen_pos(end, &panel.graph);

            let left = start_screen.x.min(end_screen.x);
            let top = start_screen.y.min(end_screen.y);
            let width = (end_screen.x - start_screen.x).abs();
            let height = (end_screen.y - start_screen.y).abs();

            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .absolute()
                        .left(px(left))
                        .top(px(top))
                        .w(px(width))
                        .h(px(height))
                        .border_1()
                        .border_color(gpui::Hsla { h: 0.58, s: 0.7, l: 0.6, a: 0.7 })
                        .bg(gpui::Hsla { h: 0.58, s: 0.5, l: 0.5, a: 0.08 })
                        .rounded(px(3.0)),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }

    fn render_viewport_bounds_debug(
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        if !cfg!(debug_assertions) {
            return div().into_any_element();
        }

        // Calculate the exact same viewport bounds used by the culling system
        let screen_to_graph_origin =
            Self::screen_to_graph_pos(Point::new(px(0.0), px(0.0)), &panel.graph);
        let screen_to_graph_end =
            Self::screen_to_graph_pos(Point::new(px(3840.0), px(2160.0)), &panel.graph);
        let padding_in_graph_space = 200.0 / panel.graph.zoom_level;

        let visible_left = screen_to_graph_origin.x - padding_in_graph_space;
        let visible_top = screen_to_graph_origin.y - padding_in_graph_space;
        let visible_right = screen_to_graph_end.x + padding_in_graph_space;
        let visible_bottom = screen_to_graph_end.y + padding_in_graph_space;

        // Convert back to screen coordinates for rendering
        let top_left_screen =
            Self::graph_to_screen_pos(Point::new(visible_left, visible_top), &panel.graph);
        let bottom_right_screen =
            Self::graph_to_screen_pos(Point::new(visible_right, visible_bottom), &panel.graph);

        let width = bottom_right_screen.x - top_left_screen.x;
        let height = bottom_right_screen.y - top_left_screen.y;

        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .left(px(top_left_screen.x))
                    .top(px(top_left_screen.y))
                    .w(px(width))
                    .h(px(height))
                    .border_2()
                    .border_color(gpui::yellow()), // Debug overlay - shows viewport bounds for culling
            )
            .into_any_element()
    }

    fn render_debug_overlay(
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        // Always show debug overlay for now to help diagnose viewport issues

        // Calculate all the viewport metrics
        let screen_to_graph_origin =
            Self::screen_to_graph_pos(Point::new(px(0.0), px(0.0)), &panel.graph);
        let screen_to_graph_end =
            Self::screen_to_graph_pos(Point::new(px(3840.0), px(2160.0)), &panel.graph);
        let padding_in_graph_space = 200.0 / panel.graph.zoom_level;

        let visible_left = screen_to_graph_origin.x - padding_in_graph_space;
        let visible_top = screen_to_graph_origin.y - padding_in_graph_space;
        let visible_right = screen_to_graph_end.x + padding_in_graph_space;
        let visible_bottom = screen_to_graph_end.y + padding_in_graph_space;

        // Calculate viewport dimensions
        let viewport_width = visible_right - visible_left;
        let viewport_height = visible_bottom - visible_top;

        // Count visible vs culled nodes and connections
        let visible_node_count = panel
            .graph
            .nodes
            .iter()
            .filter(|node| Self::is_node_visible_simple(node, &panel.graph))
            .count();
        let culled_node_count = panel.graph.nodes.len() - visible_node_count;

        let visible_connection_count = panel
            .graph
            .connections
            .iter()
            .filter(|connection| Self::is_connection_visible_simple(connection, &panel.graph))
            .count();
        let culled_connection_count = panel.graph.connections.len() - visible_connection_count;

        // Get actual container dimensions (approximation)
        let container_width = 3840.0; // Using our fixed screen bounds
        let container_height = 2160.0;

        div()
            .absolute()
            .top_4()
            .left_4()
            .w(px(280.0)) // Hardcoded width to prevent inheritance issues
            .child(
                div()
                    .w(px(280.0)) // Fixed width for compactness
                    .p_3()
                    .bg(cx.theme().background.opacity(0.95))
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .shadow_lg()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_bold()
                                            .text_color(cx.theme().accent)
                                            .child("Blueprint Viewport Debug"),
                                    )
                                    .child(
                                        Button::new("close_debug_overlay")
                                            .icon(IconName::X)
                                            .ghost()
                                            .xsmall()
                                            .on_click(cx.listener(|panel, _, _, cx| {
                                                panel.show_debug_overlay = false;
                                                cx.notify();
                                            }))
                                    )
                            )
                            .child(div().h(px(1.0)).bg(cx.theme().border).my_1())
                            .child(div().text_xs().text_color(cx.theme().info).child(format!(
                                "Container: {:.0}×{:.0}px",
                                container_width, container_height
                            )))
                            .child(div().text_xs().text_color(cx.theme().info).child(format!(
                                "Render Bounds: {:.0}×{:.0}",
                                viewport_width, viewport_height
                            )))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "Origin: ({:.0}, {:.0})",
                                        visible_left, visible_top
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "End: ({:.0}, {:.0})",
                                        visible_right, visible_bottom
                                    )),
                            )
                            .child(div().h(px(1.0)).bg(cx.theme().border).my_1())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().success)
                                    .child(format!("Nodes Rendered: {}", visible_node_count)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().danger)
                                    .child(format!("Nodes Culled: {}", culled_node_count)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("Total Nodes: {}", panel.graph.nodes.len())),
                            )
                            .child(div().h(px(1.0)).bg(cx.theme().border).my_1())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().success)
                                    .child(format!(
                                        "Connections Rendered: {}",
                                        visible_connection_count
                                    )),
                            )
                            .child(
                                div().text_xs().text_color(cx.theme().danger).child(format!(
                                    "Connections Culled: {}",
                                    culled_connection_count
                                )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "Total Connections: {}",
                                        panel.graph.connections.len()
                                    )),
                            )
                            .child(div().h(px(1.0)).bg(cx.theme().border).my_1())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().warning)
                                    .child(format!("Zoom: {:.2}x", panel.graph.zoom_level)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().warning)
                                    .child(format!(
                                        "Pan: ({:.0}, {:.0})",
                                        panel.graph.pan_offset.x, panel.graph.pan_offset.y
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().warning)
                                    .child(format!("Padding: {:.0}", padding_in_graph_space)),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_graph_controls(
        panel: &BlueprintEditorPanel,
        cx: &mut Context<BlueprintEditorPanel>,
    ) -> impl IntoElement {
        div()
            .absolute()
            .bottom_4()
            .right_4()
            .w(px(280.0)) // Hardcoded width to prevent inheritance issues
            .child(
                v_flex()
                    .gap_2()
                    .items_end()
                    .w(px(280.0)) // Hardcoded width
                    // Simplified controls since we have comprehensive debug overlay in top-left
                    .child(
                        h_flex()
                            .gap_2()
                            .p_2()
                            .w_full()
                            .bg(cx.theme().background.opacity(0.9))
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_color(cx.theme().border)
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("Zoom: {:.0}%", panel.graph.zoom_level * 100.0)),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("zoom_fit")
                                            .icon(IconName::BadgeCheck)
                                            .tooltip("Fit to View")
                                            .on_click(cx.listener(|panel, _, _window, cx| {
                                                let graph = panel.get_graph_mut();
                                                graph.zoom_level = 1.0;
                                                graph.pan_offset = Point::new(0.0, 0.0);
                                                cx.notify();
                                            }))
                                    )
                                    .child(
                                        Button::new("close_graph_controls")
                                            .icon(IconName::X)
                                            .ghost()
                                            .xsmall()
                                            .on_click(cx.listener(|panel, _, _, cx| {
                                                panel.show_graph_controls = false;
                                                cx.notify();
                                            }))
                                    )
                            )
                    )
            )
    }

    // Virtualization helper functions using viewport-aware culling
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

    // Helper functions for coordinate conversion
    pub fn graph_to_screen_pos(graph_pos: Point<f32>, graph: &BlueprintGraph) -> Point<f32> {
        Point::new(
            (graph_pos.x + graph.pan_offset.x) * graph.zoom_level,
            (graph_pos.y + graph.pan_offset.y) * graph.zoom_level,
        )
    }

    /// Convert window-relative coordinates to graph element coordinates
    /// For graph operations: clicking nodes, selection box, dragging, etc.
    ///
    /// Mouse events from GPUI are relative to window origin.
    /// We already have the graph element's bounds captured during events.
    /// Simple math: element_pos = window_pos - element_origin
    pub fn window_to_graph_element_pos(window_pos: Point<Pixels>, panel: &BlueprintEditorPanel) -> Point<Pixels> {
        if let Some(bounds) = &panel.graph_element_bounds {
            // Direct subtraction: mouse relative to element = mouse relative to window - element origin relative to window
            Point::new(
                window_pos.x - bounds.origin.x,
                window_pos.y - bounds.origin.y,
            )
        } else {
            // On first event before bounds captured, just return window pos as-is
            // This will be corrected on the next event after bounds are set
            window_pos
        }
    }

    /// Convert window-relative coordinates to panel coordinates
    /// For UI elements positioned at panel level: menus, tooltips, etc.
    pub fn window_to_panel_pos(window_pos: Point<Pixels>, panel: &BlueprintEditorPanel) -> Point<Pixels> {
        // Same calculation as graph element since they share the same coordinate space
        Self::window_to_graph_element_pos(window_pos, panel)
    }

    pub fn screen_to_graph_pos(screen_pos: Point<Pixels>, graph: &BlueprintGraph) -> Point<f32> {
        Point::new(
            (screen_pos.x.as_f32() / graph.zoom_level) - graph.pan_offset.x,
            (screen_pos.y.as_f32() / graph.zoom_level) - graph.pan_offset.y,
        )
    }

    /// Snaps a position to the appropriate grid size based on zoom level
    pub fn snap_to_grid(pos: Point<f32>, zoom_level: f32) -> Point<f32> {
        // Choose grid size based on zoom level
        // Use finer grids when zoomed in, coarser grids when zoomed out
        let grid_size = if zoom_level >= 1.5 {
            50.0  // Fine grid
        } else if zoom_level >= 0.5 {
            50.0  // Fine grid
        } else if zoom_level >= 0.3 {
            200.0 // Medium grid
        } else {
            1000.0 // Coarse grid
        };

        Point::new(
            (pos.x / grid_size).round() * grid_size,
            (pos.y / grid_size).round() * grid_size,
        )
    }

    /// Parses a hex color string (e.g., "#4A90E2") into a GPUI Hsla color
    fn parse_hex_color(hex: &str) -> Option<gpui::Hsla> {
        let hex = hex.trim_start_matches('#');

        // Parse RGB values
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

            let rgba = gpui::Rgba { r, g, b, a: 1.0 };
            Some(gpui::Hsla::from(rgba))
        } else if hex.len() == 8 {
            // Support RGBA format as well
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;

            let rgba = gpui::Rgba { r, g, b, a };
            Some(gpui::Hsla::from(rgba))
        } else {
            None
        }
    }
}
