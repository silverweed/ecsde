use super::render::Sprite;
use crate::core::common::colors::{self, Color};
use crate::core::common::rect::Rect;
use crate::core::common::Maybe_Error;
use crate::core::debug::fps::Fps_Console_Printer;
use crate::core::env::Env_Info;
use crate::core::input::{Action_List, Input_System};
use crate::core::time::Time;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::{C_Camera2D, C_Renderable};
use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::Entity;
use crate::gfx;
use crate::gfx::ui::{UI_Request, UI_System};
use crate::resources;
use std::cell::Ref;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Render_System_Config {
    pub clear_color: Color,
}

pub struct Render_System {
    config: Render_System_Config,
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: colors::rgb(0, 0, 0),
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    pub fn update(
        &mut self,
        window: &mut gfx::window::Window_Handle,
        resources: &resources::gfx::Gfx_Resources,
        camera: &C_Camera2D,
        renderables: &[(Ref<'_, C_Renderable>, Ref<'_, C_Spatial2D>)],
        frame_lag_normalized: f32,
        smooth_by_extrapolating_velocity: bool,
    ) {
        gfx::window::set_clear_color(window, self.config.clear_color);
        gfx::window::clear(window);

        for (rend, spatial) in renderables {
            let rend: &C_Renderable = &*rend;
            let C_Renderable {
                texture: tex_id,
                rect: src_rect,
                ..
            } = rend;

            let texture = resources.get_texture(*tex_id);
            let mut sprite = gfx::render::create_sprite(texture, *src_rect);

            let mut rend_transform = spatial.transform;
            if smooth_by_extrapolating_velocity {
                let v = spatial.velocity;
                rend_transform.translate(v.x * frame_lag_normalized, v.y * frame_lag_normalized);
            }

            gfx::render::render_sprite(window, &mut sprite, &rend_transform, &camera.transform);
        }
    }
}
