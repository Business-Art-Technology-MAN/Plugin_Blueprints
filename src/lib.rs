//! # Blueprint Editor Plugin
//!
//! This plugin provides visual scripting capabilities through the Blueprint Editor.
//! It supports .class files (folder-based) that contain node graphs for visual programming.
//!
//! ## File Types
//!
//! - **Blueprint Class** (.class folder)
//!   - Contains `graph_save.json` with the node graph
//!   - Contains `events/` folder for event handlers
//!   - Appears as a single file in the file drawer
//!
//! ## Editors
//!
//! - **Blueprint Editor**: Visual node-based scripting interface

use plugin_editor_api::*;
use serde_json::json;
use std::alloc::GlobalAlloc;
use std::{path::PathBuf, sync::Arc};
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// Blueprint Editor modules
mod blueprint_types;
mod events;
mod node_graph;
mod toolbar;
mod properties;
mod variables;
mod file_drawer;
mod node_creation_menu;
mod macros;
mod minimap;
mod hoverable_tooltip;
mod node_palette;
mod node_library;

// Panel module (main editor implementation)
pub mod panel;

// Re-export main types
pub use blueprint_types::*;
pub use panel::BlueprintEditorPanel;
pub use events::*;

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<BlueprintEditorWrapper>,
}

/// The Blueprint Editor Plugin
pub struct BlueprintEditorPlugin {
    /// CRITICAL: Plugin owns ALL editor instances to prevent memory leaks!
    /// The main app only gets raw pointers - it NEVER owns the Arc or Box.
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for BlueprintEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for BlueprintEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.blueprint-editor"),
            name: "Blueprint Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Visual scripting editor for creating blueprint classes".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![folder_file_type(
            "blueprint-class",
            "class",
            "Blueprint Class",
            FileIcon::Component,
            "graph_save.json",
            vec![
                PathTemplate::Folder {
                    path: "events".into(),
                },
            ],
            json!({
                "graph": {
                    "nodes": [],
                    "connections": [],
                    "comments": [],
                    "metadata": {
                        "version": "0.1.0"
                    }
                }
            }),
        )]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("blueprint-editor"),
            display_name: "Blueprint Editor".into(),
            supported_file_types: vec![FileTypeId::new("blueprint-class")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<
        (Arc<dyn PanelView>, Box<dyn EditorInstance>),
        PluginError
    > {

        logger.info("BP EDITOR LOADED!!!!!");

        logger.info(&format!("Creating editor with ID: {}", editor_id.as_str()));
        if editor_id.as_str() == "blueprint-editor" {
            // Clone file_path before moving into closure
            let file_path_clone = file_path.clone();

            // Create a view context for the panel
            let panel = cx.new(|cx| {
                match panel::BlueprintEditorPanel::new_with_path(file_path_clone.clone(), window, cx) {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("Failed to create blueprint panel: {}", e);
                        // Return a default panel on error
                        panel::BlueprintEditorPanel::new(window, cx)
                    }
                }
            });

            // Wrap the panel in Arc - will be shared with main app
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());

            // Clone file_path for logging
            let file_path_for_log = file_path.clone();

            // Create the wrapper for EditorInstance
            let wrapper = Box::new(BlueprintEditorWrapper {
                panel: panel.into(),
                file_path,
            });

            // Generate unique ID for this editor
            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            // CRITICAL: Store Arc and Box in plugin's HashMap to keep them alive!
            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created blueprint editor instance {} for {:?}", id, file_path_for_log);

            // Return Arc (main app will clone it) and Box for EditorInstance
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    // fn destroy_editor(&mut self, editor_instance: *mut dyn EditorInstance) {
    //     let mut editors = self.editors.lock().unwrap();

    //     // Find the editor by comparing wrapper pointers
    //     let editor_id = editors.iter().find_map(|(id, storage)| {
    //         let stored_ref: &dyn EditorInstance = &*storage.wrapper;
    //         let stored_ptr: *const dyn EditorInstance = stored_ref;
    //         if stored_ptr == editor_instance as *const _ {
    //             Some(*id)
    //         } else {
    //             None
    //         }
    //     });

    //     if let Some(id) = editor_id {
    //         editors.remove(&id);
    //         log::info!("Destroyed blueprint editor instance {}", id);
    //     } else {
    //         log::warn!("Attempted to destroy unknown editor instance");
    //     }
    // }

    fn on_load(&mut self) {
        log::info!("Blueprint Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        // Clear all editors when plugin unloads
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("Blueprint Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

/// Wrapper to bridge Entity<BlueprintEditorPanel> to EditorInstance trait
#[derive(Clone)]
pub struct BlueprintEditorWrapper {
    panel: Entity<BlueprintEditorPanel>,
    file_path: std::path::PathBuf,
}



impl plugin_editor_api::EditorInstance for BlueprintEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, _window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, _cx| {
            panel.plugin_save()
        })
    }

    fn reload(&mut self, _window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, _cx| {
            panel.plugin_reload()
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Export the plugin using the provided macro
export_plugin!(BlueprintEditorPlugin);
