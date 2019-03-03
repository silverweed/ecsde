use super::common;
use super::input;
use super::system::System;
use super::time;
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
    input_system: input::Input_System,
    render_system: gfx::render::Render_System,
}

impl App {
    pub fn new(cfg: &Config) -> App {
        let window = Rc::new(RefCell::new(gfx::window::create_render_window(
            cfg.target_win_size,
            &cfg.title,
        )));
        App {
            time: time::Time::new(),
            should_close: false,
            input_system: input::Input_System::new(Rc::clone(&window)),
            render_system: gfx::render::Render_System::new(Rc::clone(&window)),
        }
    }

    pub fn init(&mut self) -> common::Maybe_Error {
        println!("Hello Sailor!");

        self.input_system.init(())?;
        self.render_system.init(gfx::render::Render_System_Config {
            clear_color: sfgfx::Color::rgb(48, 10, 36),
        })?;

        Ok(())
    }

    pub fn run(&mut self) -> common::Maybe_Error {
        while !self.should_close {
            self.time.update();
            self.update_all_systems()?;
        }
        Ok(())
    }

    fn update_all_systems(&mut self) -> common::Maybe_Error {
        let dt = &self.time.dt();

        self.input_system.update(dt);
        self.render_system.update(dt);

        if self.input_system.has_action(&input::Action::Quit) {
            self.should_close = true;
        }

        Ok(())
    }
}
