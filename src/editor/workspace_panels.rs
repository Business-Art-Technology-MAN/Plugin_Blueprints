//! Dedicated panel components for the workspace docking system
//!
//! These panels wrap the editor entity and render specific content.

use gpui::*;
use ui::{ActiveTheme, dock::{Panel, PanelEvent}};

use crate::editor::panel::BlueprintEditorPanel;
use crate::features::macros::panel::MacrosRenderer;
use crate::features::variables::rendering::VariablesRenderer;
use crate::rendering::graph::NodeGraphRenderer;

/// Variables Panel
pub struct VariablesPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl VariablesPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for VariablesPanel {}

impl Render for VariablesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .size_full()
                .bg(cx.theme().sidebar)
                .child(
                    editor.update(cx, |editor, cx| {
                        VariablesRenderer::render(editor, cx)
                    })
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for VariablesPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for VariablesPanel {
    fn panel_name(&self) -> &'static str {
        "variables"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Variables".into_any_element()
    }
}

/// Macros Panel
pub struct MacrosPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl MacrosPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for MacrosPanel {}

impl Render for MacrosPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .size_full()
                .bg(cx.theme().sidebar)
                .child(
                    editor.update(cx, |editor, cx| {
                        MacrosRenderer::render(editor, cx)
                    })
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for MacrosPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for MacrosPanel {
    fn panel_name(&self) -> &'static str {
        "macros"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Macros".into_any_element()
    }
}

/// Compiler Panel
pub struct CompilerPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl CompilerPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for CompilerPanel {}

impl Render for CompilerPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .size_full()
                .child(
                    editor.update(cx, |editor, cx| {
                        editor.render_compiler_results(cx)
                    })
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for CompilerPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for CompilerPanel {
    fn panel_name(&self) -> &'static str {
        "compiler"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Compiler".into_any_element()
    }
}

/// Find Panel
pub struct FindPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl FindPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for FindPanel {}

impl Render for FindPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .size_full()
                .child(
                    editor.update(cx, |editor, cx| {
                        editor.render_find_panel(cx)
                    })
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for FindPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FindPanel {
    fn panel_name(&self) -> &'static str {
        "find"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Find".into_any_element()
    }
}

/// Properties Panel
pub struct PropertiesPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl PropertiesPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for PropertiesPanel {}

impl Render for PropertiesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .size_full()
                .bg(cx.theme().sidebar)
                .child(
                    editor.update(cx, |editor, cx| {
                        let selected_count = editor.graph.selected_nodes.len();

                        div()
                            .size_full()
                            .p_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Details")
                            )
                            .child(
                                div()
                                    .mt_2()
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
                    })
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for PropertiesPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PropertiesPanel {
    fn panel_name(&self) -> &'static str {
        "properties"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Details".into_any_element()
    }
}

/// Palette Panel placeholder
pub struct PalettePanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl PalettePanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for PalettePanel {}

impl Render for PalettePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.editor.upgrade().is_some() {
            div()
                .size_full()
                .bg(cx.theme().sidebar)
                .p_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Palette panel is being migrated")
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for PalettePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PalettePanel {
    fn panel_name(&self) -> &'static str {
        "palette"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Palette".into_any_element()
    }
}

/// Main graph canvas panel with tab bar
pub struct GraphCanvasPanel {
    editor: WeakEntity<BlueprintEditorPanel>,
    focus_handle: FocusHandle,
}

impl GraphCanvasPanel {
    pub fn new(editor: WeakEntity<BlueprintEditorPanel>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for GraphCanvasPanel {}

impl Render for GraphCanvasPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(editor) = self.editor.upgrade() {
            div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    editor.update(cx, |editor, cx| {
                        editor.render_tab_bar(cx)
                    })
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .child(
                            editor.update(cx, |editor, cx| {
                                NodeGraphRenderer::render(editor, cx)
                            })
                        )
                )
        } else {
            div().child("Editor not available")
        }
    }
}

impl Focusable for GraphCanvasPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for GraphCanvasPanel {
    fn panel_name(&self) -> &'static str {
        "graph-canvas"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Event Graph".into_any_element()
    }
}
