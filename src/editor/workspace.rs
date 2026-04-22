//! Workspace initialization and layout
//!
//! Handles setting up the docking workspace with sidebar panels

use gpui::*;
use crate::editor::panel::BlueprintEditorPanel;

impl BlueprintEditorPanel {
    /// Initialize the docking workspace with panels
    pub fn initialize_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Create workspace if it doesn't exist
        if self.workspace.is_none() {
            let workspace = cx.entity(|cx| ui::workspace::Workspace::new(window, cx));
            self.workspace = Some(workspace);
        }
    }
}
