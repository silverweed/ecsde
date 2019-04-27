use crate::core;
use crate::core::common::transform::C_Transform2D;
use crate::core::common::vector::Vec2f;
use crate::ecs::components as comp;
use crate::resources;
use cgmath::Deg;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
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

    pub fn init(&mut self, cfg: Render_System_Config) -> core::common::Maybe_Error {
        self.config = cfg;
        self.camera.translate(150.0, 100.0);
        Ok(())
    }

    pub fn update(
        &mut self,
        canvas: &mut WindowCanvas,
        resources: &resources::Resources,
        renderables: &[(Ref<'_, comp::C_Renderable>, Ref<'_, C_Transform2D>)],
    ) {
        canvas.set_draw_color(self.config.clear_color);
        canvas.clear();

        // DEBUG
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        let (out_x, out_y) = canvas.output_size().unwrap();
        canvas.fill_rect(Rect::new(0, 0, out_x, out_y));
        // END DEBUG

        let Vec2f { x: cam_x, y: cam_y } = self.camera.position();
        let Vec2f {
            x: cam_sx,
            y: cam_sy,
        } = self.camera.scale();
        canvas.set_scale(cam_sx, cam_sy).unwrap();

        for (rend, transf) in renderables {
            let rend: &comp::C_Renderable = &*rend;
            let comp::C_Renderable {
                texture: tex_id,
                rect: src_rect,
                ..
            } = rend;

            let texture = resources.get_texture(*tex_id);

            let pos = transf.position();
            let Deg(angle) = transf.rotation().into();
            let scale = transf.scale();

            let dst_rect = Rect::new(
                (pos.x - cam_x) as i32,
                (pos.y - cam_y) as i32,
                (scale.x * (src_rect.width() as f32)) as u32,
                (scale.y * (src_rect.height() as f32)) as u32,
            );

            if let Err(msg) = canvas.copy_ex(
                texture,
                Some(*src_rect),
                dst_rect,
                f64::from(angle), // degrees!
                None,
                false,
                false,
            ) {
                eprintln!("Error copying texture to window: {}", msg);
            }
        }
        canvas.present();
    }
}
