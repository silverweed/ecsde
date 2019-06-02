use super::common::colors;
use super::common::Maybe_Error;
use super::debug;
use super::env::Env_Info;
use super::input;
use super::time;
use crate::audio;
use crate::cfg;
use crate::ecs::components::gfx::C_Camera2D;
use crate::ecs::components::transform::C_Transform2D;
use crate::ecs::entity_manager::Entity;
use crate::fs;
use crate::game::gameplay_system;
use crate::gfx;
use crate::resources;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::SystemTime;

pub struct App_Config {
    pub title: String,
    pub target_win_size: (u32, u32),
}

impl App_Config {
    pub fn new(mut args: std::env::Args) -> App_Config {
        let mut cfg = App_Config {
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
    time: time::Time,

    should_close: bool,

    env: Env_Info,

    config: cfg::Config,
    ui_req_tx: mpsc::Sender<gfx::ui::UI_Request>,

    // Resources
    gfx_resources: resources::gfx::Gfx_Resources<'r>,
    audio_resources: resources::audio::Audio_Resources<'r>,

    // Engine Systems
    input_system: input::Input_System,
    input_actions_rx: mpsc::Receiver<input::Action_List>,
    render_system: gfx::render_system::Render_System,
    ui_system: gfx::ui::UI_System,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

impl<'r> App<'r> {
    pub fn new(sound_loader: &'r audio::sound_loader::Sound_Loader) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());

        let (input_tx, input_rx) = mpsc::channel();
        let (ui_tx, ui_rx) = mpsc::channel();

        App {
            time: time::Time::new(),
            should_close: false,
            env,
            config,
            ui_req_tx: ui_tx,
            input_system: input::Input_System::new(input_tx),
            input_actions_rx: input_rx,
            gfx_resources: resources::gfx::Gfx_Resources::new(),
            audio_resources: resources::audio::Audio_Resources::new(sound_loader),
            render_system: gfx::render_system::Render_System::new(),
            ui_system: gfx::ui::UI_System::new(ui_rx),
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

    fn init_all_systems(&mut self) -> Maybe_Error {
        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&self.config));
        fs::file_watcher::start_file_watch(
            self.env.get_cfg_root().to_path_buf(),
            vec![config_watcher],
        )?;

        self.gameplay_system
            .init(&mut self.gfx_resources, &self.env, &self.config)?;
        self.render_system
            .init(gfx::render_system::Render_System_Config {
                clear_color: colors::rgb(22, 0, 22),
            })?;

        Ok(())
    }

    pub fn run(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        self.start_game_loop(window)?;
        Ok(())
    }

    fn start_game_loop(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");
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
            self.input_system.update(window);
            let actions = self.input_system.get_action_list();
            self.handle_actions(&actions)?;

            // Update game systems
            let gameplay_start_t = SystemTime::now();
            while execution_time > update_time {
                self.update_game_systems(update_time, &actions)?;
                execution_time -= update_time;
            }
            println!(
                "Gameplay: {} ms",
                SystemTime::now()
                    .duration_since(gameplay_start_t)
                    .unwrap()
                    .as_millis()
            );

            // Update audio
            self.audio_system.update();

            // Render
            let render_start_t = SystemTime::now();
            self.update_graphics(
                window,
                real_dt,
                time::duration_ratio(&execution_time, &update_time) as f32,
            )?;
            println!(
                "Render: {} ms",
                SystemTime::now()
                    .duration_since(render_start_t)
                    .unwrap()
                    .as_millis()
            );

            #[cfg(debug_assertions)]
            {
                let sleep = *self
                    .config
                    .get_var_int_or("engine/debug/extra_frame_sleep_ms", 0)
                    as u64;
                std::thread::sleep(Duration::from_millis(sleep));
            }

            self.config.update();
            fps_debug.tick(&dt);
        }

        Ok(())
    }

    fn update_game_systems(&mut self, dt: Duration, actions: &input::Action_List) -> Maybe_Error {
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

        gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
        gfx::window::clear(window);
        self.render_system.update(
            window,
            &self.gfx_resources,
            &self.gameplay_system.get_camera(),
            &self.gameplay_system.get_renderable_entities(),
            frame_lag_normalized,
            smooth_by_extrapolating_velocity,
        );
        self.ui_system
            .update(&real_dt, window, &mut self.gfx_resources);
        gfx::window::display(window);

        Ok(())
    }

    fn handle_actions(&mut self, actions: &input::Action_List) -> Maybe_Error {
        use gfx::ui::UI_Request;
        use input::Action;

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
                        self.ui_req_tx
                            .send(UI_Request::Add_Fadeout_Text(format!(
                                "Time scale: {:.2}",
                                self.time.get_time_scale()
                            )))
                            .unwrap();
                    }
                    Action::Pause_Toggle => {
                        self.time.set_paused(!self.time.is_paused());
                        self.ui_req_tx
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
                        self.ui_req_tx
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
