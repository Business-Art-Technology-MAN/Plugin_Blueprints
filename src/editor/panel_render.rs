//! Rendering - GPUI render implementation and trait implementations

use gpui::*;
use gpui::prelude::*;
use ui::{dock::{Panel, PanelEvent, PanelState}, h_flex, v_flex, ActiveTheme};

use super::panel::BlueprintEditorPanel;
use super::toolbar::ToolbarRenderer;
use crate::core::events::*;
use crate::features::macros::panel::MacrosRenderer;
use crate::features::variables::rendering::VariablesRenderer;
use crate::rendering::graph::NodeGraphRenderer;

impl Panel for BlueprintEditorPanel {
    fn panel_name(&self) -> &'static str {
        "Blueprint Editor"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        h_flex()
            .gap_2()
            .items_center()
            .child(div().text_sm().child(if let Some(title) = &self.tab_title {
                title.clone()
            } else {
                "Blueprint Editor".to_string()
            }))
            .into_any_element()
    }

    fn dump(&self, _cx: &App) -> PanelState {
        PanelState {
            panel_name: self.panel_name().to_string(),
            ..Default::default()
        }
    }
}

impl Focusable for BlueprintEditorPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for BlueprintEditorPanel {}
impl EventEmitter<OpenEngineLibraryRequest> for BlueprintEditorPanel {}
impl EventEmitter<ShowNodePickerRequest> for BlueprintEditorPanel {}

impl BlueprintEditorPanel {
    /// Render compiler results panel (compilation history and status)
    pub fn render_compiler_results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::core::types::CompilationState;

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .bg(cx.theme().secondary)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(match self.compilation_status.state {
                                CompilationState::Success => gpui::green(),
                                CompilationState::Error => gpui::red(),
                                CompilationState::Compiling => gpui::yellow(),
                                _ => cx.theme().foreground,
                            })
                            .child(match self.compilation_status.state {
                                CompilationState::Idle => "Compiler Output",
                                CompilationState::Compiling => "⟳ Compiling...",
                                CompilationState::Success => "✓ Build Succeeded",
                                CompilationState::Error => "✗ Build Failed",
                            })
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} messages", self.compilation_history.len()))
                    )
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .gap_0p5()
                            .children(self.compilation_history.iter().rev().map(|entry| {
                                h_flex()
                                    .w_full()
                                    .px_2()
                                    .py_1()
                                    .gap_2()
                                    .border_b_1()
                                    .border_color(cx.theme().border.opacity(0.1))
                                    .hover(|s| s.bg(cx.theme().muted.opacity(0.05)))
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_family("JetBrainsMono-Regular")
                                            .text_color(cx.theme().muted_foreground.opacity(0.7))
                                            .child(entry.timestamp.clone())
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .w(px(12.0))
                                            .text_xs()
                                            .text_color(match entry.state {
                                                CompilationState::Success => gpui::green(),
                                                CompilationState::Error => gpui::red(),
                                                _ => cx.theme().muted_foreground,
                                            })
                                            .child(match entry.state {
                                                CompilationState::Success => "✓",
                                                CompilationState::Error => "✗",
                                                _ => "•",
                                            })
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_xs()
                                            .text_color(cx.theme().foreground)
                                            .child(entry.message.clone())
                                    )
                            }))
                            .when(self.compilation_history.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .py(px(32.0))
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No compilation messages yet.")
                                )
                            })
                    )
            )
    }

    fn render_left_panel_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(32.0))
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .items_center()
            .child(self.render_sidebar_tab("Variables", 0, cx))
            .child(self.render_sidebar_tab("Macros", 1, cx))
            .child(div().flex_1())
    }

    fn render_sidebar_tab(
        &self,
        label: &'static str,
        tab_index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.left_top_tab == tab_index;

        h_flex()
            .h_full()
            .items_center()
            .px_3()
            .cursor_pointer()
            .when(is_active, |this| {
                this.border_b_2()
                    .border_color(cx.theme().accent)
                    .bg(cx.theme().background)
            })
            .when(!is_active, |this| {
                this.text_color(cx.theme().muted_foreground)
                    .hover(|s| s.bg(cx.theme().muted.opacity(0.1)))
            })
            .child(
                div()
                    .text_xs()
                    .when(is_active, |s| {
                        s.text_color(cx.theme().foreground)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .child(label)
            )
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |panel, _, _window, cx| {
                panel.left_top_tab = tab_index;
                cx.notify();
            }))
    }

    fn render_right_panel_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(32.0))
            .px_3()
            .items_center()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().foreground)
                    .child("Details")
            )
    }

    fn render_right_panel_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_count = self.graph.selected_nodes.len();

        v_flex()
            .size_full()
            .p_3()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().foreground)
                    .child("Selection")
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(if selected_count == 0 {
                        "No node selected".to_string()
                    } else if selected_count == 1 {
                        "1 node selected".to_string()
                    } else {
                        format!("{} nodes selected", selected_count)
                    })
            )
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Detailed property editing is being migrated and will return here.")
            )
    }
}

impl Render for BlueprintEditorPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .key_context("BlueprintEditor")
            .on_action(cx.listener(|panel, action: &DuplicateNode, _window, cx| {
                panel.duplicate_node(action.node_id.clone(), cx);
            }))
            .on_action(cx.listener(|panel, action: &DeleteNode, _window, cx| {
                panel.delete_node(action.node_id.clone(), cx);
            }))
            .on_action(cx.listener(|panel, action: &CopyNode, _window, cx| {
                panel.copy_node(action.node_id.clone(), cx);
            }))
            .on_action(cx.listener(|panel, _action: &PasteNode, _window, cx| {
                panel.paste_node(cx);
            }))
            .on_action(cx.listener(|panel, action: &DisconnectPin, _window, cx| {
                panel.disconnect_pin(action.node_id.clone(), action.pin_id.clone(), cx);
            }))
            .on_action(cx.listener(|panel, _action: &OpenAddNodeMenu, window, cx| {
                if let Some(bounds) = &panel.graph_element_bounds {
                    let screen_center = Point::new(bounds.center().x, bounds.center().y);
                    let graph_pos = NodeGraphRenderer::screen_to_graph_pos(screen_center, &panel.graph);
                    panel.show_node_picker(graph_pos, window, cx);
                }
            }))
            .child(ToolbarRenderer::render(self, cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(300.0))
                            .min_w(px(220.0))
                            .max_w(px(420.0))
                            .h_full()
                            .bg(cx.theme().sidebar)
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .child(self.render_left_panel_tabs(cx))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .map(|el| match self.left_top_tab {
                                        0 => el.child(VariablesRenderer::render(self, cx)),
                                        1 => el.child(MacrosRenderer::render(self, cx)),
                                        _ => el.child(VariablesRenderer::render(self, cx)),
                                    })
                            )
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .child(NodeGraphRenderer::render(self, cx))
                    )
                    .child(
                        v_flex()
                            .w(px(340.0))
                            .min_w(px(260.0))
                            .max_w(px(480.0))
                            .h_full()
                            .bg(cx.theme().sidebar)
                            .border_l_1()
                            .border_color(cx.theme().border)
                            .child(self.render_right_panel_header(cx))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .child(self.render_right_panel_content(cx))
                            )
                    )
            )
            .child(
                div()
                    .h(px(180.0))
                    .min_h(px(120.0))
                    .max_h(px(320.0))
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(self.render_compiler_results(cx))
            )
    }
}
