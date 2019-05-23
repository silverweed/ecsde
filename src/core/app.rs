use super::common::colors::Color;
use super::common::Maybe_Error;
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
use std::convert::TryFrom;
use std::time::Duration;

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

pub struct App<'r> {
    event_pump: sdl2::EventPump,

    time: time::Time,

    should_close: bool,

    env: Env_Info,
    resources: resources::Resources<'r>,

    config: cfg::Config,
    ui_req_tx: Option<std::sync::mpsc::Sender<gfx::ui::UI_Request>>,

    // Engine Systems
    input_system: input::Input_System,
    render_system: gfx::render_system::Render_System,
    ui_system: gfx::ui::UI_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

pub struct Resource_Loaders {
    pub texture_creator: resources::Texture_Creator,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
    pub sound_loader: audio::sound_loader::Sound_Loader,
}

impl<'r> App<'r> {
    pub fn new(event_pump: sdl2::EventPump, loaders: &'r Resource_Loaders) -> Self {
        let resources = resources::Resources::new(
            &loaders.texture_creator,
            &loaders.ttf_context,
            &loaders.sound_loader,
        );

        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());

        App {
            event_pump,
            time: time::Time::new(),
            should_close: false,
            env,
            resources,
            config,
            ui_req_tx: None,
            input_system: input::Input_System::new(),
            render_system: gfx::render_system::Render_System::new(),
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

    pub fn run(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3));

        let mut execution_time = Duration::new(0, 0);
        while !self.should_close {
            // Update time
            self.time.update();

            let dt = self.time.dt();
            let real_dt = self.time.real_dt();
            let update_time = Duration::from_millis(
                *self
                    .config
                    .get_var_int_or("engine/gameplay/gameplay_update_tick_ms", 10)
                    as u64,
            );

            execution_time += dt;

            // Update input
            self.handle_actions()?;
            self.input_system.update(&mut self.event_pump);

            // Update game systems
            while execution_time > update_time {
                self.update_game_systems(update_time)?;
                execution_time -= update_time;
            }

            // Update audio
            self.audio_system.update();

            // Render
            self.update_graphics(
                window,
                real_dt,
                time::duration_ratio(&execution_time, &update_time) as f32,
            )?;

            #[cfg(debug_assertions)]
            {
                let sleep = *self
                    .config
                    .get_var_int_or("engine/debug/extra_frame_sleep_ms", 0)
                    as u64;
                std::thread::sleep(Duration::from_millis(sleep));
            }

            self.config.update();
            fps_debug.tick(&self.time);
        }

        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        self.render_system
            .init(gfx::render_system::Render_System_Config {
                clear_color: Color::RGB(48, 10, 36),
            })?;
        self.gameplay_system
            .init(&self.env, &mut self.resources, &self.config)?;
        self.ui_system.init(&self.env, &mut self.resources)?;

        self.ui_req_tx = Some(self.ui_system.new_request_sender());

        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&self.config));
        fs::file_watcher::start_file_watch(
            self.env.get_cfg_root().to_path_buf(),
            vec![config_watcher],
        )?;

        Ok(())
    }

    fn update_game_systems(&mut self, dt: Duration) -> Maybe_Error {
        let actions = self.input_system.get_actions();
        self.gameplay_system.update(&dt, actions);

        Ok(())
    }

    fn update_graphics(
        &mut self,
        window: &mut gfx::window::Window_Handle,
        real_dt: Duration,
        frame_lag_normalized: f32,
    ) -> Maybe_Error {
        let smooth_by_extrapolating_velocity = *self
            .config
            .get_var_bool_or("engine/rendering/smooth_by_extrapolating_velocity", false);

        gfx::window::set_clear_color(window, Color::RGB(0, 0, 0));
        gfx::window::clear(window);
        self.render_system.update(
            window,
            &self.resources,
            &self.gameplay_system.get_renderable_entities(),
            frame_lag_normalized,
            smooth_by_extrapolating_velocity,
        );
        self.ui_system.update(&real_dt, window, &mut self.resources);
        gfx::window::display(window);

        Ok(())
    }

    fn handle_actions(&mut self) -> Maybe_Error {
        use gfx::ui::UI_Request;
        use input::Action;

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
                        let target_fps = self.config.get_var_int_or("engine/rendering/fps", 60);
                        let step_delta = Duration::from_nanos(
                            u64::try_from(1_000_000_000 / *target_fps).unwrap(),
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
