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
    ui_req_tx: Option<mpsc::Sender<gfx::ui::UI_Request>>,

    // Resources
    audio_resources: resources::audio::Audio_Resources<'r>,

    // Engine Systems
    render_thread: Option<JoinHandle<()>>,
    render_thread_quit: Option<mpsc::Sender<()>>,
    input_actions_rx: Option<mpsc::Receiver<input::Action_List>>,
    audio_system: audio::system::Audio_System,
    gameplay_system: gameplay_system::Gameplay_System,
}

impl<'r> App<'r> {
    pub fn new(sound_loader: &'r audio::sound_loader::Sound_Loader) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());

        App {
            time: time::Time::new(),
            should_close: false,
            env,
            config,
            ui_req_tx: None,
            audio_resources: resources::audio::Audio_Resources::new(sound_loader),
            render_thread: None,
            render_thread_quit: None,
            input_actions_rx: None,
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

        Ok(())
    }

    fn start_render_thread(
        &mut self,
        entity_transform_rx: mpsc::Receiver<(Entity, C_Transform2D)>,
        camera_transform_rx: mpsc::Receiver<C_Camera2D>,
    ) {
        let (input_tx, input_rx) = mpsc::channel();
        self.input_actions_rx = Some(input_rx);

        let (ui_tx, ui_rx) = mpsc::channel();
        self.ui_req_tx = Some(ui_tx);

        let (quit_tx, quit_rx) = mpsc::channel();
        self.render_thread_quit = Some(quit_tx);

        self.render_thread = Some(gfx::render_system::start_render_thread(
            self.env.clone(),
            input_tx,
            ui_rx,
            entity_transform_rx,
            camera_transform_rx,
            quit_rx,
            gfx::render_system::Render_System_Config {
                clear_color: colors::rgb(48, 10, 36),
            },
        ));
    }

    pub fn run(&mut self) -> Maybe_Error {
        let (et_tx, et_rx) = mpsc::channel();
        let (cam_tx, cam_rx) = mpsc::channel();
        self.gameplay_system.init(&self.config, et_tx, cam_tx)?; // @Temporary workaround
        self.start_render_thread(et_rx, cam_rx);

        self.start_game_loop()?;
        Ok(())
    }

    fn start_game_loop(&mut self) -> Maybe_Error {
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");
        let input_actions_rx = self.input_actions_rx.take().unwrap();
        let mut execution_time = Duration::new(0, 0);

        while !self.should_close {
            let frame_start_t = std::time::SystemTime::now();

            // Update time
            self.time.update();

            let dt = self.time.dt();

            let update_time = Duration::from_nanos(
                (*self
                    .config
                    .get_var_float_or("engine/gameplay/gameplay_update_tick_ms", 10.0)
                    * 1_000_000.0) as u64,
            );

            // Update input
            // Note: due to SFML limitations, the event loop is run on the render thread.
            let actions = if let Ok(new_actions) = input_actions_rx.try_recv() {
                new_actions
            } else {
                input::Action_List::default()
            };

            self.handle_actions(&actions)?;

            execution_time += dt;

            while execution_time > update_time {
                // Update game systems
                self.update_game_systems(update_time, &actions)?;
                execution_time -= update_time;
            }

            // Update audio
            self.audio_system.update();

            self.config.update();

            let frame_duration = std::time::SystemTime::now()
                .duration_since(frame_start_t)
                .unwrap();

            fps_debug.tick(&self.time.real_dt());

            if frame_duration < update_time {
                std::thread::sleep(update_time - frame_duration);
            } else {
                eprintln!(
                    "[ WARNING ] Game loop took {} ms, which is more than the requested {} ms.",
                    frame_duration.as_millis(),
                    update_time.as_millis()
                );
            }
        }

        self.render_thread_quit
            .as_mut()
            .unwrap()
            .send(())
            .expect("[ ERR ] Failed to send quit message to render thread!");
        self.render_thread
            .take()
            .unwrap()
            .join()
            .expect("[ ERR ] Failed to join render thread!");

        Ok(())
    }

    fn update_game_systems(&mut self, dt: Duration, actions: &input::Action_List) -> Maybe_Error {
        self.gameplay_system.update(&dt, actions);

        Ok(())
    }

    fn handle_actions(&mut self, actions: &input::Action_List) -> Maybe_Error {
        use gfx::ui::UI_Request;
        use input::Action;

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
