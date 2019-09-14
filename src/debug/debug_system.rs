use super::fadeout_overlay;
use super::overlay;
use crate::core::common::stringid::String_Id;
use crate::core::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Duration;

pub struct Debug_System {
    overlays: HashMap<String_Id, overlay::Debug_Overlay>,
    fadeout_overlays: HashMap<String_Id, fadeout_overlay::Fadeout_Debug_Overlay>,
}

impl Debug_System {
    pub fn new() -> Debug_System {
        Debug_System {
            overlays: HashMap::new(),
            fadeout_overlays: HashMap::new(),
        }
    }

    pub fn init(&mut self, _env: &Env_Info, _gres: &mut Gfx_Resources) -> Maybe_Error {
        Ok(())
    }

    pub fn create_overlay(
        &mut self,
        id: String_Id,
        config: overlay::Debug_Overlay_Config,
        font: Font_Handle,
    ) -> &mut overlay::Debug_Overlay {
        match self.overlays.entry(id) {
            Entry::Occupied(e) => {
                eprintln!(
                    "[ WARNING ] Overlay {} already exists: won't overwrite.",
                    id
                );
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
                eprintln!(
                    "[ WARNING ] Overlay {} already exists: won't overwrite.",
                    id
                );
                e.into_mut()
            }
            Entry::Vacant(v) => v.insert(fadeout_overlay::Fadeout_Debug_Overlay::new(config, font)),
        }
    }

    pub fn get_overlay(&mut self, id: String_Id) -> &mut overlay::Debug_Overlay {
        self.overlays
            .get_mut(&id)
            .unwrap_or_else(|| panic!("Invalid debug overlay: {}", id))
    }

    pub fn get_fadeout_overlay(
        &mut self,
        id: String_Id,
    ) -> &mut fadeout_overlay::Fadeout_Debug_Overlay {
        self.fadeout_overlays
            .get_mut(&id)
            .unwrap_or_else(|| panic!("Invalid fadout debug overlay: {}", id))
    }

    pub fn update(&mut self, dt: &Duration, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        for (_, overlay) in self.overlays.iter_mut() {
            overlay.draw(window, gres);
        }

        for (_, overlay) in self.fadeout_overlays.iter_mut() {
            overlay.update(dt);
            overlay.draw(window, gres);
        }
    }
}
