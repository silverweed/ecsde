use super::render::Sprite;
use crate::core::common::colors::{self, Color};
use crate::core::common::rect::Rect;
use crate::core::common::Maybe_Error;
use crate::core::debug::fps::Fps_Console_Printer;
use crate::core::env::Env_Info;
use crate::core::time::Time;
use crate::ecs::components::base::C_Spatial2D;
use crate::ecs::components::gfx::C_Renderable;
use crate::gfx;
use crate::resources;
use std::cell::Ref;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Render_System {
    config: Render_System_Config,
    pub camera: C_Spatial2D, // TODO figure out where to put this
}

pub struct Render_System_Config {
    pub clear_color: Color,
}

pub fn start_render_thread(
    env: Env_Info,
    sdl: &sdl2::Sdl,
    cfg: Render_System_Config,
) -> JoinHandle<()> {
    let mut camera = C_Spatial2D::default();
    camera.transform.translate(150.0, 100.0);

    thread::Builder::new()
        .name(String::from("render_thread"))
        .spawn(move || {
            render_loop(cfg, env, sdl, camera);
        })
        .unwrap()
}

fn render_loop(cfg: Render_System_Config, env: Env_Info, sdl: &sdl2::Sdl, camera: C_Spatial2D) {
    let video_subsystem = sdl.video().unwrap();
    let window = gfx::window::create_render_window(&(), (800, 600), "Unnamed app");

    let mut fps_debug = Fps_Console_Printer::new(&Duration::from_secs(3), "render");
    let mut time = Time::new();

    let texture_creator = window.texture_creator();
    let ttf = sdl2::ttf::init().unwrap();
    let mut gres = resources::gfx::Gfx_Resources::new(&texture_creator, &ttf);

    let yv_tex_h = gres.load_texture(&resources::gfx::tex_path(&env, "yv.png"));
    let yv_tex = gres.get_texture(yv_tex_h);

    gfx::window::set_clear_color(&mut window, cfg.clear_color);
    loop {
        time.update();

        gfx::window::clear(&mut window);
        gfx::render::render_texture(&mut window, &yv_tex, Rect::new(0, 0, 100, 100));
        gfx::window::display(&mut window);

        fps_debug.tick(&time);
    }
}

impl Render_System {
    pub fn new() -> Self {
        Render_System {
            config: Self::default_config(),
            camera: C_Spatial2D::default(),
        }
    }

    fn default_config() -> Render_System_Config {
        Render_System_Config {
            clear_color: colors::rgb(0, 0, 0),
        }
    }

    pub fn update(
        &mut self,
        window: &mut gfx::window::Window_Handle,
        resources: &resources::gfx::Gfx_Resources,
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

            gfx::render::render_sprite(
                window,
                &mut sprite,
                &rend_transform,
                &self.camera.transform,
            );
        }
    }
}
