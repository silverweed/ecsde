use super::fadeout_overlay;
use super::graph;
use super::overlay;
use crate::common::stringid::String_Id;
use crate::gfx::window::Window_Handle;
use crate::prelude::*;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct Debug_Ui_System_Config {
    pub ui_scale: f32,
}

impl Default for Debug_Ui_System_Config {
    fn default() -> Self {
        Self { ui_scale: 1.0 }
    }
}

#[derive(Default)]
pub struct Debug_Ui_System {
    overlays: HashMap<String_Id, overlay::Debug_Overlay>,
    fadeout_overlays: HashMap<String_Id, fadeout_overlay::Fadeout_Debug_Overlay>,
    disabled_overlays: HashMap<String_Id, overlay::Debug_Overlay>,
    graphs: HashMap<String_Id, graph::Debug_Graph_View>,
    cfg: Debug_Ui_System_Config,
}

// @Cleanup: this needs refactoring!
impl Debug_Ui_System {
    pub fn new() -> Debug_Ui_System {
        Debug_Ui_System {
            overlays: HashMap::new(),
            fadeout_overlays: HashMap::new(),
            disabled_overlays: HashMap::new(),
            graphs: HashMap::new(),
            cfg: Debug_Ui_System_Config::default(),
        }
    }

    pub fn init(&mut self, cfg: Debug_Ui_System_Config) {
        self.cfg = cfg;
    }

    pub fn config(&self) -> &Debug_Ui_System_Config {
        &self.cfg
    }

    pub fn create_overlay(
        &mut self,
        id: String_Id,
        config: overlay::Debug_Overlay_Config,
        font: Font_Handle,
    ) -> &mut overlay::Debug_Overlay {
        match self.overlays.entry(id) {
            Entry::Occupied(e) => {
                lwarn!("Overlay {} already exists: won't overwrite.", id);
                e.into_mut()
            }
            Entry::Vacant(v) => v.insert(overlay::Debug_Overlay::new(config, font)),
        }
    }

    pub fn create_fadeout_overlay(
        &mut self,
        id: String_Id,
        config: fadeout_overlay::Fadeout_Debug_Overlay_Config,
        font: Font_Handle,
    ) -> &mut fadeout_overlay::Fadeout_Debug_Overlay {
        match self.fadeout_overlays.entry(id) {
            Entry::Occupied(e) => {
                lwarn!("Overlay {} already exists: won't overwrite.", id);
                e.into_mut()
            }
            Entry::Vacant(v) => v.insert(fadeout_overlay::Fadeout_Debug_Overlay::new(config, font)),
        }
    }

    pub fn create_graph(
        &mut self,
        id: String_Id,
        config: graph::Debug_Graph_View_Config,
        font: Font_Handle,
    ) -> &mut graph::Debug_Graph_View {
        match self.graphs.entry(id) {
            Entry::Occupied(e) => {
                lwarn!("Graph {} already exists: won't overwrite.", id);
                e.into_mut()
            }
            Entry::Vacant(v) => v.insert(graph::Debug_Graph_View::new(config, font)),
        }
    }

    pub fn get_overlay(&mut self, id: String_Id) -> &mut overlay::Debug_Overlay {
        self.overlays
            .get_mut(&id)
            .unwrap_or_else(|| fatal!("Invalid debug overlay: {}", id))
    }

    pub fn get_fadeout_overlay(
        &mut self,
        id: String_Id,
    ) -> &mut fadeout_overlay::Fadeout_Debug_Overlay {
        self.fadeout_overlays
            .get_mut(&id)
            .unwrap_or_else(|| fatal!("Invalid fadout debug overlay: {}", id))
    }

    pub fn get_graph(&mut self, id: String_Id) -> &mut graph::Debug_Graph_View {
        self.graphs
            .get_mut(&id)
            .unwrap_or_else(|| fatal!("Invalid debug graph: {}", id))
    }

    pub fn set_overlay_enabled(&mut self, id: String_Id, enabled: bool) {
        let (source_map, target_map, action) = if enabled {
            (&mut self.disabled_overlays, &mut self.overlays, "enable")
        } else {
            (&mut self.overlays, &mut self.disabled_overlays, "disable")
        };

        if let Some(overlay) = source_map.remove(&id) {
            assert!(target_map.get(&id).is_none());
            target_map.insert(id, overlay);
        } else {
            lwarn!(
                "Failed to {} overlay {}: either already in that state or not existing.",
                action,
                id
            );
        }
    }

    pub fn update(
        &mut self,
        dt: &Duration,
        window: &mut Window_Handle,
        gres: &mut Gfx_Resources,
        _tracer: Debug_Tracer,
    ) {
        for (_, graph) in self.graphs.iter_mut() {
            graph.data.remove_points_before_x_range();
            graph.draw(window, gres, clone_tracer!(_tracer));
        }

        for (_, overlay) in self.overlays.iter_mut() {
            overlay.draw(window, gres, clone_tracer!(_tracer));
        }

        for (_, overlay) in self.fadeout_overlays.iter_mut() {
            overlay.update(dt);
            overlay.draw(window, gres, clone_tracer!(_tracer));
        }
    }
}
