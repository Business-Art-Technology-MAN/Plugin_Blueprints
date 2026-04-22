//! Rendering - GPUI render implementation and trait implementations

use gpui::*;
use gpui::prelude::*;
use ui::{dock::{Panel, PanelEvent, PanelState}, h_flex, v_flex, ActiveTheme, PixelsExt};
use super::panel::BlueprintEditorPanel;
use super::toolbar::ToolbarRenderer;
use crate::core::events::*;

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

// Plugin-related methods (called by BlueprintEditorWrapper)
impl BlueprintEditorPanel {
    pub fn plugin_save(&mut self) -> Result<(), plugin_editor_api::PluginError> {
        if let Some(path) = self.current_class_path.clone() {
            // Clone the path to avoid borrow checker issues
            let path_str = path.to_str().unwrap().to_string();
            self.save_blueprint(&path_str)
                .map_err(|e| plugin_editor_api::PluginError::FileSaveError {
                    path: path.clone(),
                    message: e.to_string(),
                })
        } else {
            Err(plugin_editor_api::PluginError::Other {
                message: "No file path set".into(),
            })
        }
    }

    pub fn plugin_reload(&mut self) -> Result<(), plugin_editor_api::PluginError> {
        // TODO: Implement reload functionality
        // For now, return an error indicating it's not implemented
        Err(plugin_editor_api::PluginError::Other {
            message: "Reload not yet implemented for blueprint editor".into(),
        })
    }

    /// Render compiler results panel (compilation history and status)
    pub fn render_compiler_results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::core::types::CompilationState;
        use ui::{button::{Button, ButtonVariants}, IconName};

        v_flex()
            .size_full()
            .child(
                // Header with current status
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
                // Scrollable history list
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .gap_0p5()
                            .children(
                                self.compilation_history.iter().rev().map(|entry| {
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
                                })
                            )
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
}

impl Render for BlueprintEditorPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Initialize workspace if needed
        if self.workspace.is_none() {
            self.initialize_workspace(window, cx);
        }

        use ui::{button::{Button, ButtonVariants}, IconName};

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
                // Open node menu at center of visible graph area
                if let Some(bounds) = &panel.graph_element_bounds {
                    let screen_center = Point::new(
                        bounds.center().x,
                        bounds.center().y,
                    );
                    let graph_pos = crate::rendering::graph::NodeGraphRenderer::screen_to_graph_pos(
                        screen_center,
                        &panel.graph
                    );
                    panel.show_node_picker(graph_pos, window, cx);
                }
            }))
            .child(ToolbarRenderer::render(self, cx))
            .child(
                // Modular workspace with fully dockable panels
                div()
                    .flex_1()
                    .min_h_0()
                    .map(|el| {
                        if let Some(workspace) = &self.workspace {
                            el.child(workspace.clone())
                        } else {
                            el.child(div().child("Initializing workspace..."))
                        }
                    })
            )
    }
}
