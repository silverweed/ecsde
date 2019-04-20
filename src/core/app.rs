use super::common;
use super::common::vector::Vec2u;
use super::debug;
use super::env::Env_Info;
use super::input;
use super::time;
use crate::audio;
use crate::game::gameplay_system;
use crate::gfx;
use crate::resources::resources;
use sdl2::pixels::Color;

pub struct Config {
    title: String,
    target_win_size: (u32, u32),
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Config {
        let mut cfg = Config {
            title: String::from("Unnamed app"),
            target_win_size: (800, 600),
        };

        // Consume program name
        args.next();

        while let Some(arg) = args.next() {
            match arg.as_ref() {
                "--title" => {
                    if let Some(title) = args.next() {
                        cfg.title = title;
                    } else {
                        eprintln!("Expected an argument after --title flag.");
                    }
                }
                _ => eprintln!("Unknown argument {}", arg),
            }
        }

        cfg
    }
}

struct Sdl {
    context: sdl2::Sdl,
    event_pump: sdl2::EventPump,
    video_subsystem: sdl2::VideoSubsystem,
}

pub struct App {
    sdl: Sdl,

    window_target_size: common::vector::Vec2u,
    canvas: sdl2::render::WindowCanvas,

    time: time::Time,

    should_close: bool,

    env: Env_Info,
    resources: resources::Resources,

    // Engine Systems
    input_system: input::Input_System,
    render_system: gfx::render::Render_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

impl App {
    pub fn new(cfg: &Config) -> Self {
        let sdl = sdl2::init().unwrap();
        let event_pump = sdl.event_pump().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let canvas =
            gfx::window::create_render_canvas(&video_subsystem, cfg.target_win_size, &cfg.title);
        let texture_creator = canvas.texture_creator();

        let app = App {
            sdl: Sdl {
                context: sdl,
                event_pump,
                video_subsystem,
            },
            window_target_size: Vec2u::new(cfg.target_win_size.0, cfg.target_win_size.1),
            canvas,
            time: time::Time::new(),
            should_close: false,
            env: Env_Info::gather().unwrap(),
            resources: resources::Resources::new(texture_creator),
            input_system: input::Input_System::new(),
            render_system: gfx::render::Render_System::new(),
            audio_system: audio::system::Audio_System::new(),
            gameplay_system: gameplay_system::Gameplay_System::new(),
        };
        app
    }

    pub fn init(&mut self) -> common::Maybe_Error {
        println!(
            "Working dir = {:?}\nExe = {:?}",
            self.env.get_cwd(),
            self.env.get_exe()
        );

        self.init_all_systems()?;

        Ok(())
    }

    pub fn run(&mut self) -> common::Maybe_Error {
        let mut fps_debug =
            debug::fps::Fps_Console_Printer::new(&std::time::Duration::from_secs(3));

        while !self.should_close {
            self.time.update();
            self.update_all_systems()?;
            fps_debug.tick(&self.time);
        }

        Ok(())
    }

    fn init_all_systems(&mut self) -> common::Maybe_Error {
        self.render_system.init(gfx::render::Render_System_Config {
            clear_color: Color::RGB(48, 10, 36),
        })?;
        self.gameplay_system.init(&self.env, &mut self.resources)?;

        Ok(())
    }

    fn update_all_systems(&mut self) -> common::Maybe_Error {
        let dt = self.time.dt();

        self.input_system.update(&mut self.sdl.event_pump);
        let actions = self.input_system.get_actions();

        self.gameplay_system.update(&dt, actions);
        self.render_system.update(
            &mut self.canvas,
            &self.resources,
            &self.gameplay_system.get_renderable_entities(),
        );
        self.audio_system.update();

        self.handle_actions()
    }

    fn handle_actions(&mut self) -> common::Maybe_Error {
        let actions = self.input_system.get_actions();

        if actions.has_action(&input::Action::Quit) {
            self.should_close = true;
        } else {
            for action in actions.iter() {
                match action {
                    input::Action::ChangeSpeed(delta) => {
                        self.time
                            .set_time_scale(self.time.get_time_scale() + *delta as f32 * 0.01);
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }
}
