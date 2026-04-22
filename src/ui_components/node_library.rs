//! Node library panel renderer.
//!
//! Provides a categorized list of available nodes and lets users add a node
//! to the graph by clicking it.

use gpui::*;
use ui::{h_flex, v_flex, ActiveTheme as _, StyledExt};

use crate::core::definitions::{NodeCategory, NodeDefinitions};
use crate::core::types::BlueprintNode;
use crate::editor::panel::BlueprintEditorPanel;
use crate::rendering::graph::NodeGraphRenderer;

pub struct NodeLibraryRenderer;

impl NodeLibraryRenderer {
    pub fn render(panel: &BlueprintEditorPanel, cx: &mut Context<BlueprintEditorPanel>) -> impl IntoElement {
        let node_definitions = NodeDefinitions::load();

        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .child(
                h_flex()
                    .w_full()
                    .px_3()
                    .py_2()
                    .bg(cx.theme().secondary)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("Palette")
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} categories", node_definitions.categories.len()))
                    )
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .p_3()
                            .gap_3()
                            .scrollable(Axis::Vertical)
                            .children(
                                node_definitions
                                    .categories
                                    .iter()
                                    .map(|category| Self::render_category(category, cx))
                            )
                    )
            )
    }

    fn render_category(category: &NodeCategory, cx: &mut Context<BlueprintEditorPanel>) -> impl IntoElement {
        v_flex()
            .gap_1p5()
            .child(
                h_flex()
                    .w_full()
                    .px_2()
                    .py_1()
                    .rounded(px(4.0))
                    .bg(cx.theme().muted.opacity(0.2))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(category.name.clone())
                    )
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(category.nodes.iter().map(|node_def| {
                        let def = node_def.clone();

                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1p5()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.25)))
                            .child(
                                div()
                                    .text_sm()
                                    .child(def.icon.clone())
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child(def.name.clone())
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(def.description.clone())
                                    )
                            )
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |panel, _event, _window, cx| {
                                let screen_pos = if let Some(bounds) = panel.graph_element_bounds {
                                    bounds.center()
                                } else {
                                    Point::new(px(640.0), px(360.0))
                                };

                                let graph_pos = NodeGraphRenderer::screen_to_graph_pos(screen_pos, &panel.graph);
                                let stagger = (panel.graph.nodes.len() % 8) as f32 * 18.0;
                                let place_pos = Point::new(graph_pos.x + stagger, graph_pos.y + stagger);

                                let node = BlueprintNode::from_definition(&def, place_pos);
                                panel.add_node(node, cx);
                            }))
                    }))
            )
    }
}
