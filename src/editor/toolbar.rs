//! Toolbar rendering
//!
//! Top toolbar with compile button, file operations, etc.

use gpui::*;
use ui::{h_flex, ActiveTheme, PixelsExt, StyledExt};
use crate::editor::panel::BlueprintEditorPanel;

pub struct ToolbarRenderer;

impl ToolbarRenderer {
    pub fn render(panel: &BlueprintEditorPanel, cx: &mut Context<BlueprintEditorPanel>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(32.))
            .px_2()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(if let Some(title) = &panel.tab_title {
                        title.clone()
                    } else {
                        "Blueprint Editor".to_string()
                    })
            )
    }
}
