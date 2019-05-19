use crate::core::common::Maybe_Error;
use crate::core::common::colors::Color;
use crate::core::common::transform::C_Transform2D;
use crate::core::common::vector::Vec2f;
use crate::core::common::rect::Rect;
use super::render::Sprite;
use crate::gfx;
use crate::ecs::components as comp;
use crate::resources;
use cgmath::Deg;
use std::cell::Ref;
use std::convert::Into;

pub struct Render_System {
    config: Render_System_Config,
    pub camera: C_Transform2D, // TODO figure out where to put this
}

pub struct Render_System_Config {
    pub clear_color: Color,
}


impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
            camera: C_Transform2D::default(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: Color::RGB(0, 0, 0),
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> Maybe_Error {
        self.config = cfg;
        self.camera.translate(150.0, 100.0);
        Ok(())
    }

    pub fn update(
        &mut self,
        window: &mut gfx::window::Window_Handle,
        resources: &resources::Resources,
        renderables: &[(Ref<'_, comp::C_Renderable>, Ref<'_, C_Transform2D>)],
    ) {
        gfx::window::set_clear_color(window, self.config.clear_color);
        gfx::window::clear(window);

        for (rend, transf) in renderables {
            let rend: &comp::C_Renderable = &*rend;
            let comp::C_Renderable {
                texture: tex_id,
                rect: src_rect,
                ..
            } = rend;

            let texture = resources.get_texture(*tex_id);
            let sprite = Sprite {
                texture: texture,
                rect: *src_rect,
            };
            gfx::render::render_sprite(window, &sprite, transf, &self.camera);
        }
    }
}

