//! Workspace initialization and layout
//!
//! Handles setting up the docking workspace with sidebar panels

use gpui::*;
use std::sync::Arc;
use ui::dock::DockItem;
use ui::workspace::Workspace;

use crate::editor::panel::BlueprintEditorPanel;
use crate::editor::workspace_panels::{
    CompilerPanel,
    FindPanel,
    GraphCanvasPanel,
    MacrosPanel,
    PalettePanel,
    PropertiesPanel,
    VariablesPanel,
};

impl BlueprintEditorPanel {
    /// Initialize the docking workspace with panels
    pub fn initialize_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.workspace.is_some() {
            return;
        }

        let editor_weak = cx.entity().downgrade();

        let workspace = cx.new(|cx| {
            Workspace::new_with_channel(
                "blueprint-editor-workspace",
                ui::dock::DockChannel(1),
                window,
                cx,
            )
        });

        workspace.update(cx, |workspace, cx| {
            let dock_area_weak = workspace.dock_area().downgrade();

            let variables_panel = cx.new(|cx| VariablesPanel::new(editor_weak.clone(), cx));
            let macros_panel = cx.new(|cx| MacrosPanel::new(editor_weak.clone(), cx));
            let compiler_panel = cx.new(|cx| CompilerPanel::new(editor_weak.clone(), cx));
            let find_panel = cx.new(|cx| FindPanel::new(editor_weak.clone(), cx));
            let properties_panel = cx.new(|cx| PropertiesPanel::new(editor_weak.clone(), cx));
            let palette_panel = cx.new(|cx| PalettePanel::new(editor_weak.clone(), cx));
            let center_panel = cx.new(|cx| GraphCanvasPanel::new(editor_weak.clone(), cx));

            let center = DockItem::tabs(
                vec![Arc::new(center_panel)],
                None,
                &dock_area_weak,
                window,
                cx,
            );

            let left = DockItem::tabs(
                vec![Arc::new(variables_panel), Arc::new(macros_panel)],
                None,
                &dock_area_weak,
                window,
                cx,
            );

            let right = DockItem::tabs(
                vec![Arc::new(properties_panel), Arc::new(palette_panel)],
                None,
                &dock_area_weak,
                window,
                cx,
            );

            let bottom = DockItem::tabs(
                vec![Arc::new(compiler_panel), Arc::new(find_panel)],
                None,
                &dock_area_weak,
                window,
                cx,
            );

            workspace.initialize(center, Some(left), Some(right), Some(bottom), window, cx);
        });

        self.workspace = Some(workspace);
    }
}
