use super::common;
use super::env::Env_Info;
use super::input;
use super::resources;
use super::system::System;
use super::time;
use crate::audio;
use crate::game::gameplay_system;
use crate::gfx;
use sfml::graphics as sfgfx;
use std::cell::RefCell;
use std::rc::Rc;

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

pub struct App {
    time: time::Time,
    should_close: bool,
    env: Env_Info,
    resources: resources::Resources,
    window: Rc<RefCell<gfx::window::Window>>,
    input_system: input::Input_System,
    render_system: gfx::render::Render_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

impl App {
    pub fn new(cfg: &Config) -> App {
        let app = App {
            time: time::Time::new(),
            should_close: false,
            env: Env_Info::gather().unwrap(),
            resources: resources::Resources::new(),
            window: Rc::new(RefCell::new(gfx::window::create_render_window(
                cfg.target_win_size,
                &cfg.title,
            ))),
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
        self.input_system.init(())?;
        self.render_system.init(gfx::render::Render_System_Config {
            clear_color: sfgfx::Color::rgb(48, 10, 36),
        })?;
        self.audio_system.init(())?;
        self.gameplay_system.init(())?;

        Ok(())
    }

    fn update_all_systems(&mut self) -> common::Maybe_Error {
        let dt = &self.time.dt();

        self.input_system.update(input::Input_System_Update_Params {
            window: Rc::clone(&self.window),
        });
        self.gameplay_system.update(());
        self.render_system
            .update(gfx::render::Render_System_Update_Params {
                window: Rc::clone(&self.window),
            });
        self.audio_system.update(());

        if self.input_system.has_action(&input::Action::Quit) {
            self.should_close = true;
        }

        Ok(())
    }
}
