//! Save and load operations for blueprint files
//!
//! This module provides the main save/load functionality for blueprints,
//! including autosave, format detection, and legacy format migration.

use std::path::{Path, PathBuf};
use gpui::*;
use crate::editor::panel::BlueprintEditorPanel;
use super::{formats, legacy};

impl BlueprintEditorPanel {
    /// Save the current blueprint to its file path
    pub fn plugin_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = &self.current_class_path {
            match self.save_to_path(path, window, cx) {
                Ok(()) => {
                    tracing::info!("Blueprint saved successfully to {:?}", path);
                    self.is_dirty = false;
                    cx.notify();
                }
                Err(e) => {
                    tracing::error!("Failed to save blueprint: {}", e);
                    // TODO: Show error notification to user
                }
            }
        } else {
            tracing::warn!("No save path set - cannot save blueprint");
            // TODO: Show save-as dialog
        }
    }

    /// Reload the blueprint from its file path
    pub fn plugin_reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = &self.current_class_path {
            match self.load_from_path(path, window, cx) {
                Ok(()) => {
                    tracing::info!("Blueprint reloaded successfully from {:?}", path);
                    self.is_dirty = false;
                    cx.notify();
                }
                Err(e) => {
                    tracing::error!("Failed to reload blueprint: {}", e);
                    // TODO: Show error notification to user
                }
            }
        } else {
            tracing::warn!("No file path set - cannot reload blueprint");
        }
    }

    /// Save blueprint to a specific path
    pub fn save_to_path(
        &mut self,
        path: &Path,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Result<(), String> {
        // Convert current graph state to BlueprintAsset
        let asset = self.to_blueprint_asset()?;

        // Serialize to JSON with header
        let content = formats::serialize_blueprint_with_header(&asset)?;

        // Write to file
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load blueprint from a specific path
    pub fn load_from_path(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Try to deserialize as current format first
        let asset = match formats::deserialize_blueprint(&content) {
            Ok(asset) => asset,
            Err(_) => {
                // Try legacy format
                tracing::info!("Trying to load as legacy format...");
                let legacy_graph = legacy::try_parse_legacy_format(&content)?;

                // Convert legacy graph to current format
                formats::BlueprintAsset {
                    format_version: formats::current_format_version(),
                    main_graph: legacy_graph,
                    local_macros: Vec::new(),
                    variables: Vec::new(),
                    editor_state: None,
                    blueprint_metadata: Default::default(),
                }
            }
        };

        // Load the asset into the editor
        self.load_blueprint_asset(asset, window, cx)?;

        Ok(())
    }

    /// Convert current editor state to BlueprintAsset
    fn to_blueprint_asset(&self) -> Result<formats::BlueprintAsset, String> {
        // TODO: Convert graph to GraphDescription
        // TODO: Collect local macros
        // TODO: Collect variables
        // TODO: Capture editor state

        // For now, create a minimal asset
        Ok(formats::BlueprintAsset {
            format_version: formats::current_format_version(),
            main_graph: ui::graph::GraphDescription::new("EventGraph"),
            local_macros: self.local_macros.clone(),
            variables: self.variables.clone(),
            editor_state: Some(formats::BlueprintEditorState {
                open_tab_ids: self.open_tabs.iter().map(|tab| tab.id.clone()).collect(),
                active_tab_index: self.active_tab_index,
                graph_view_states: std::collections::HashMap::new(),
            }),
            blueprint_metadata: Default::default(),
        })
    }

    /// Load BlueprintAsset into the editor
    fn load_blueprint_asset(
        &mut self,
        asset: formats::BlueprintAsset,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        // Check format version compatibility
        if !formats::is_version_supported(asset.format_version) {
            return Err(format!(
                "Unsupported blueprint format version: {}",
                asset.format_version
            ));
        }

        // TODO: Convert GraphDescription to BlueprintGraph
        // TODO: Load local macros
        // TODO: Load variables
        // TODO: Restore editor state

        // For now, just load the main graph
        self.graph = crate::core::graph::BlueprintGraph {
            nodes: Vec::new(),
            connections: Vec::new(),
            comments: Vec::new(),
            selected_nodes: Vec::new(),
            selected_comments: Vec::new(),
            zoom_level: 1.0,
            pan_offset: Point::new(0.0, 0.0),
            virtualization_stats: Default::default(),
        };

        self.local_macros = asset.local_macros;
        self.variables = asset.variables;

        cx.notify();
        Ok(())
    }

    /// Autosave - called periodically to save work in progress
    pub fn autosave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_dirty {
            return; // No changes to save
        }

        if let Some(path) = &self.current_class_path {
            // Create autosave path (same location with .autosave extension)
            let autosave_path = path.with_extension("blueprint.autosave");

            match self.save_to_path(&autosave_path, window, cx) {
                Ok(()) => {
                    tracing::debug!("Autosaved to {:?}", autosave_path);
                }
                Err(e) => {
                    tracing::error!("Autosave failed: {}", e);
                }
            }
        }
    }

    /// Check if an autosave file exists for the current path
    pub fn has_autosave(&self) -> bool {
        if let Some(path) = &self.current_class_path {
            let autosave_path = path.with_extension("blueprint.autosave");
            autosave_path.exists()
        } else {
            false
        }
    }

    /// Load from autosave file (recovery)
    pub fn load_autosave(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if let Some(path) = &self.current_class_path {
            let autosave_path = path.with_extension("blueprint.autosave");
            self.load_from_path(&autosave_path, window, cx)?;

            // Delete autosave after successful recovery
            std::fs::remove_file(&autosave_path)
                .map_err(|e| format!("Failed to delete autosave file: {}", e))?;

            Ok(())
        } else {
            Err("No file path set - cannot load autosave".to_string())
        }
    }

    /// Mark the blueprint as dirty (has unsaved changes)
    pub fn mark_dirty(&mut self, cx: &mut Context<Self>) {
        if !self.is_dirty {
            self.is_dirty = true;
            cx.notify();
        }
    }

    /// Export blueprint to a different format or location
    pub fn export_blueprint(
        &self,
        export_path: &Path,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let asset = self.to_blueprint_asset()?;
        let content = formats::serialize_blueprint_with_header(&asset)?;

        std::fs::write(export_path, content)
            .map_err(|e| format!("Failed to export blueprint: {}", e))?;

        tracing::info!("Blueprint exported to {:?}", export_path);
        Ok(())
    }
}

/// Utility functions for file path handling
impl BlueprintEditorPanel {
    /// Get the display name for the current blueprint
    pub fn get_display_name(&self) -> String {
        if let Some(path) = &self.current_class_path {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            "Untitled Blueprint".to_string()
        }
    }

    /// Get the full path as a string
    pub fn get_path_string(&self) -> Option<String> {
        self.current_class_path
            .as_ref()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
    }

    /// Set the current file path
    pub fn set_path(&mut self, path: PathBuf) {
        self.current_class_path = Some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autosave_path_generation() {
        let path = PathBuf::from("/path/to/test.blueprint");
        let autosave_path = path.with_extension("blueprint.autosave");
        assert_eq!(
            autosave_path.to_str().unwrap(),
            "/path/to/test.blueprint.autosave"
        );
    }
}
