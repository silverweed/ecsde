use super::common::vector::Vec2u;
use super::common::{self, Maybe_Error};
use super::debug;
use super::env::Env_Info;
use super::input;
use super::time;
use crate::audio;
use crate::cfg;
use crate::fs;
use crate::game::gameplay_system;
use crate::gfx;
use crate::resources;
use sdl2::pixels::Color;
use std::convert::TryFrom;

pub struct Config {
    pub title: String,
    pub target_win_size: (u32, u32),
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
    event_pump: sdl2::EventPump,
}

pub struct App<'r> {
    sdl: Sdl,

    window_target_size: common::vector::Vec2u,
    canvas: &'r mut sdl2::render::WindowCanvas,

    time: time::Time,

    should_close: bool,

    env: Env_Info,
    resources: resources::Resources<'r>,

    config: cfg::Config,

    ui_req_tx: Option<std::sync::mpsc::Sender<gfx::ui::UI_Request>>,

    // Engine Systems
    input_system: input::Input_System,
    render_system: gfx::render::Render_System,
    ui_system: gfx::ui::UI_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

pub struct Resource_Loaders {
    pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
    pub sound_loader: audio::sound_loader::Sound_Loader,
}

impl<'r> App<'r> {
    pub fn new(
        cfg: &Config,
        sdl: &sdl2::Sdl,
        canvas: &'r mut sdl2::render::WindowCanvas,
        loaders: &'r Resource_Loaders,
    ) -> Self {
        let event_pump = sdl.event_pump().unwrap();
        let sdl = Sdl { event_pump };
        let resources = resources::Resources::new(
            &loaders.texture_creator,
            &loaders.ttf_context,
            &loaders.sound_loader,
        );

        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new(env.get_cfg_root());

        App {
            sdl,
            window_target_size: Vec2u::new(cfg.target_win_size.0, cfg.target_win_size.1),
            canvas,
            time: time::Time::new(),
            should_close: false,
            env,
            resources,
            config,
            ui_req_tx: None,
            input_system: input::Input_System::new(),
            render_system: gfx::render::Render_System::new(),
            ui_system: gfx::ui::UI_System::new(),
            audio_system: audio::system::Audio_System::new(10),
            gameplay_system: gameplay_system::Gameplay_System::new(),
        }
    }

    pub fn init(&mut self) -> Maybe_Error {
        println!(
            "Working dir = {:?}\nExe = {:?}",
            self.env.get_cwd(),
            self.env.get_exe()
        );

        self.init_all_systems()?;

        Ok(())
    }

    pub fn run(&mut self) -> Maybe_Error {
        let mut fps_debug =
            debug::fps::Fps_Console_Printer::new(&std::time::Duration::from_secs(3));

        while !self.should_close {
            self.time.update();
            self.update_all_systems()?;
            fps_debug.tick(&self.time);
        }

        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        self.render_system.init(gfx::render::Render_System_Config {
            clear_color: Color::RGB(48, 10, 36),
        })?;
        self.gameplay_system
            .init(&self.env, &mut self.resources, &self.config)?;
        self.ui_system.init(&self.env, &mut self.resources)?;

        self.ui_req_tx = Some(self.ui_system.new_request_sender());

        fs::file_watcher::file_watcher_create(
            self.env.get_cfg_root().to_path_buf(),
            self.ui_system.new_request_sender(),
        )?;

        Ok(())
    }

    fn update_all_systems(&mut self) -> Maybe_Error {
        self.handle_actions()?;

        let dt = self.time.dt();
        let real_dt = self.time.real_dt();

        self.input_system.update(&mut self.sdl.event_pump);
        let actions = self.input_system.get_actions();

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.gameplay_system.update(&dt, actions);
        self.render_system.update(
            &mut self.canvas,
            &self.resources,
            &self.gameplay_system.get_renderable_entities(),
        );
        self.ui_system
            .update(&real_dt, &mut self.canvas, &mut self.resources);
        self.audio_system.update();

        self.canvas.present();

        Ok(())
    }

    fn handle_actions(&mut self) -> Maybe_Error {
        use gfx::ui::UI_Request;
        use input::Action;
        use std::time::Duration;

        let actions = self.input_system.get_actions();
        let ui_req_tx = self.ui_req_tx.as_ref().unwrap();

        if actions.has_action(&Action::Quit) {
            self.should_close = true;
        } else {
            for action in actions.iter() {
                match action {
                    Action::Change_Speed(delta) => {
                        let ts = self.time.get_time_scale() + *delta as f32 * 0.01;
                        if ts > 0.0 {
                            self.time.set_time_scale(ts);
                        }
                        ui_req_tx
                            .send(UI_Request::Add_Fadeout_Text(format!(
                                "Time scale: {:.2}",
                                self.time.get_time_scale()
                            )))
                            .unwrap();
                    }
                    Action::Pause_Toggle => {
                        self.time.set_paused(!self.time.is_paused());
                        ui_req_tx
                            .send(UI_Request::Add_Fadeout_Text(String::from(
                                if self.time.is_paused() {
                                    "Paused"
                                } else {
                                    "Resumed"
                                },
                            )))
                            .unwrap();
                    }
                    Action::Step_Simulation => {
                        let target_fps = self.config.get_var_or::<i32, _>("engine/fps", 60);
                        let step_delta = Duration::from_nanos(
                            u64::try_from(1_000_000_000 / i32::from(target_fps)).unwrap(),
                        );
                        ui_req_tx
                            .send(UI_Request::Add_Fadeout_Text(format!(
                                "Stepping of: {:.2} ms",
                                time::to_secs_frac(&step_delta) * 1000.0
                            )))
                            .unwrap();
                        self.time.set_paused(true);
                        self.time.step(&step_delta);
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }
}
