use crate::core;
use crate::ecs::components as comp;
use crate::resources::resources;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

pub struct Render_System {
    config: Render_System_Config,
}

pub struct Render_System_Config {
    pub clear_color: Color,
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: Color::RGB(0, 0, 0),
        }
    }

    pub fn init(&mut self, cfg: Render_System_Config) -> core::common::Maybe_Error {
        self.config = cfg;
        Ok(())
    }

    pub fn update(
        &mut self,
        canvas: &mut WindowCanvas,
        resources: &resources::Resources,
        renderables: &[(&comp::C_Renderable, &comp::C_Position2D)],
    ) {
        canvas.set_draw_color(self.config.clear_color);
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (rend, pos) in renderables {
            let sprite = resources.get_sprite(rend.sprite);
            let texture = resources.get_texture(sprite.texture);
            let dst = sdl2::rect::Rect::new(
                pos.x as i32,
                pos.y as i32,
                sprite.rect.width(),
                sprite.rect.height(),
            );

            if let Err(msg) = canvas.copy(texture, Some(sprite.rect), dst) {
                eprintln!("Error copying texture to window: {}", msg);
            }
        }
        canvas.present();
    }
}
