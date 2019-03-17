use super::common;
use super::env::Env_Info;
use super::input;
use super::time;
use crate::audio;
use crate::game::gameplay_system;
use crate::gfx;
use crate::resources::resources;
use sfml::graphics as sfgfx;
use sfml::graphics::RenderTarget;
use sfml::system as sfsys;

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

pub struct App<'a> {
    time: time::Time,
    should_close: bool,
    env: Env_Info,
    resources: resources::Resources<'a>,
    window: gfx::window::Window,
    input_system: input::Input_System,
    render_system: gfx::render::Render_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

impl<'a> App<'a> {
    pub fn new(cfg: &Config) -> Self {
        let app = App {
            time: time::Time::new(),
            should_close: false,
            env: Env_Info::gather().unwrap(),
            resources: resources::Resources::new(),
            window: gfx::window::create_render_window(cfg.target_win_size, &cfg.title),
            input_system: input::Input_System::new(),
            render_system: gfx::render::Render_System::new(),
            audio_system: audio::system::Audio_System::new(),
            gameplay_system: gameplay_system::Gameplay_System::new(),
        };
        app
    }

    pub fn init(&mut self) -> common::Maybe_Error {
        println!(
            "Working dir = {}\nExe = {}",
            self.env.get_cwd(),
            self.env.get_exe()
        );

        self.init_all_systems()?;

        Ok(())
    }

    pub fn run(&mut self) -> common::Maybe_Error {
        while !self.should_close {
            self.time.update();
            self.update_all_systems()?;
        }
        Ok(())
    }

    fn init_all_systems(&mut self) -> common::Maybe_Error {
        self.render_system.init(gfx::render::Render_System_Config {
            clear_color: sfgfx::Color::rgb(48, 10, 36),
        })?;
        self.gameplay_system.init(&self.env, &mut self.resources)?;

        Ok(())
    }

    fn update_all_systems(&mut self) -> common::Maybe_Error {
        let dt = &self.time.dt();

        self.input_system.update(&mut self.window.sf_win);
        self.gameplay_system.update();
        self.render_system.update(&mut self.window.sf_win);
        self.audio_system.update();

        self.handle_actions()
    }

    fn handle_actions(&mut self) -> common::Maybe_Error {
        if self.input_system.has_action(&input::Action::Quit) {
            // If we're ask to close, don't bother processing other actions.
            self.should_close = true;
            return Ok(());
        }

        for action in self.input_system.get_actions() {
            match action {
                input::Action::Resize(width, height) => {
                    self.window.sf_win.set_view(&gfx::window::keep_ratio(
                        &sfsys::Vector2u::new(*width, *height),
                        &self.window.target_size,
                    ));
                }
                _ => {}
            }
        }

        Ok(())
    }
}
