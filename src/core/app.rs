use super::common::colors;
use super::common::Maybe_Error;
use super::debug;
use super::env::Env_Info;
use super::input;
use super::msg;
use super::systems;
use super::time;
use super::time_manager;
use crate::audio;
use crate::cfg;
use crate::fs;
use crate::gfx;
use crate::resources;
use crate::states;
use std::cell::RefCell;
use std::rc::Rc;
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
    time: Rc<RefCell<time_manager::Time_Manager>>,

    should_close: bool,

    env: Env_Info,

    config: cfg::Config,

    state_mgr: states::state_manager::State_Manager,

    // Resources
    gfx_resources: resources::gfx::Gfx_Resources<'r>,
    audio_resources: resources::audio::Audio_Resources<'r>,

    systems: systems::Core_Systems,
    dispatcher: msg::Msg_Dispatcher,
}

impl<'r> App<'r> {
    pub fn new(sound_loader: &'r audio::sound_loader::Sound_Loader) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());

        App {
            time: Rc::new(RefCell::new(time_manager::Time_Manager::new())),
            should_close: false,
            env,
            config,
            state_mgr: states::state_manager::State_Manager::new(),
            gfx_resources: resources::gfx::Gfx_Resources::new(),
            audio_resources: resources::audio::Audio_Resources::new(sound_loader),
            systems: systems::Core_Systems::new(),
            dispatcher: msg::Msg_Dispatcher::new(),
        }
    }

    pub fn init(&mut self) -> Maybe_Error {
        println!(
            "Working dir = {:?}\nExe = {:?}",
            self.env.get_cwd(),
            self.env.get_exe()
        );

        self.init_states()?;
        self.init_all_systems()?;
        self.init_dispatcher()?;

        Ok(())
    }

    fn init_states(&mut self) -> Maybe_Error {
        let base_state = Box::new(states::debug_base_state::Debug_Base_State {});
        self.state_mgr.add_persistent_state(base_state);
        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&self.config));
        fs::file_watcher::start_file_watch(
            self.env.get_cfg_root().to_path_buf(),
            vec![config_watcher],
        )?;

        self.systems.gameplay_system.borrow_mut().init(
            &mut self.gfx_resources,
            &self.env,
            &self.config,
        )?;
        self.systems
            .render_system
            .borrow_mut()
            .init(gfx::render_system::Render_System_Config {
                clear_color: colors::rgb(22, 0, 22),
            })?;
        self.systems
            .ui_system
            .borrow_mut()
            .init(&self.env, &mut self.gfx_resources)?;

        Ok(())
    }

    fn init_dispatcher(&mut self) -> Maybe_Error {
        let disp = &mut self.dispatcher;
        disp.register(self.time.clone());
        disp.register(self.systems.ui_system.clone());
        disp.register(self.systems.gameplay_system.clone());
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
            self.time.borrow_mut().time.update();

            let (dt, real_dt) = {
                let time = &self.time.borrow().time;
                (time.dt(), time.real_dt())
            };
            let update_time = Duration::from_millis(
                *self
                    .config
                    .get_var_int_or("engine/gameplay/gameplay_update_tick_ms", 10)
                    as u64,
            );

            execution_time += dt;

            // Update input
            self.systems.input_system.borrow_mut().update(window);
            let actions = self.systems.input_system.borrow().get_action_list();

            if self
                .state_mgr
                .handle_actions(&actions, &self.dispatcher, &self.config)
            {
                self.should_close = true;
                break;
            }

            // Update game systems
            {
                #[cfg(prof_t)]
                let gameplay_start_t = SystemTime::now();

                let mut gameplay_system = self.systems.gameplay_system.borrow_mut();

                gameplay_system.realtime_update(&real_dt, &actions);
                while execution_time > update_time {
                    gameplay_system.update(&update_time, &actions);
                    execution_time -= update_time;
                }

                #[cfg(prof_t)]
                println!(
                    "Gameplay: {} ms",
                    SystemTime::now()
                        .duration_since(gameplay_start_t)
                        .unwrap()
                        .as_millis()
                );
            }

            // Update audio
            self.systems.audio_system.borrow_mut().update();

            // Render
            #[cfg(prof_t)]
            let render_start_t = SystemTime::now();

            self.update_graphics(
                window,
                real_dt,
                time::duration_ratio(&execution_time, &update_time) as f32,
            )?;

            #[cfg(prof_t)]
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
            fps_debug.tick(&real_dt);
        }

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
        self.systems.render_system.borrow_mut().update(
            window,
            &self.gfx_resources,
            &self.systems.gameplay_system.borrow().get_camera(),
            &self
                .systems
                .gameplay_system
                .borrow()
                .get_renderable_entities(),
            frame_lag_normalized,
            smooth_by_extrapolating_velocity,
        );
        self.systems
            .ui_system
            .borrow_mut()
            .update(&real_dt, window, &mut self.gfx_resources);
        gfx::window::display(window);

        Ok(())
    }
}
