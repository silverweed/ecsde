use super::app_config::App_Config;
use super::common::colors;
use super::common::Maybe_Error;
use super::debug;
use super::env::Env_Info;
use super::time;
use super::world;
use crate::audio;
use crate::cfg;
use crate::fs;
use crate::gfx;
use crate::replay::{replay_data, replay_system};
use crate::resources;
use crate::states;
use std::path;
use std::time::Duration;

pub struct App<'r> {
    should_close: bool,

    env: Env_Info,
    config: cfg::Config,

    state_mgr: states::state_manager::State_Manager,

    gfx_resources: resources::gfx::Gfx_Resources<'r>,
    audio_resources: resources::audio::Audio_Resources<'r>,

    world: world::World,

    replay_system: replay_system::Replay_System,
    replay_data: Option<replay_data::Replay_Data>,
}

impl<'r> App<'r> {
    pub fn new(cfg: &App_Config, sound_loader: &'r audio::sound_loader::Sound_Loader) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());
        let replay_data = if let Some(path) = &cfg.replay_file {
            Some(
                replay_data::Replay_Data::from_serialized(&path)
                    .expect("Failed to load replay data!"),
            )
        } else {
            None
        };

        App {
            should_close: false,
            env,
            config,
            state_mgr: states::state_manager::State_Manager::new(),
            gfx_resources: resources::gfx::Gfx_Resources::new(),
            audio_resources: resources::audio::Audio_Resources::new(sound_loader),
            world: world::World::new(),
            replay_system: replay_system::Replay_System::new(),
            replay_data,
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
        self.world.init();

        Ok(())
    }

    fn init_states(&mut self) -> Maybe_Error {
        let base_state = Box::new(states::debug_base_state::Debug_Base_State {});
        self.state_mgr.add_persistent_state(&self.world, base_state);
        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&self.config));
        fs::file_watcher::start_file_watch(
            self.env.get_cfg_root().to_path_buf(),
            vec![config_watcher],
        )?;

        let systems = self.world.get_systems();
        systems.gameplay_system.borrow_mut().init(
            &mut self.gfx_resources,
            &self.env,
            &self.config,
        )?;
        systems
            .render_system
            .borrow_mut()
            .init(gfx::render_system::Render_System_Config {
                clear_color: colors::rgb(22, 0, 22),
            })?;
        systems
            .ui_system
            .borrow_mut()
            .init(&self.env, &mut self.gfx_resources)?;

        Ok(())
    }

    pub fn run(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        self.start_game_loop(window)?;
        Ok(())
    }

    fn start_game_loop(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");
        let mut execution_time = Duration::new(0, 0);
        let mut cur_frame = 0u64;

        // Consumes self.replay_data!
        let replay_data = self.replay_data.take();
        let mut replay_data_iter = if let Some(replay_data) = &replay_data {
            Some(replay_data.iter())
        } else {
            None
        };

        while !self.should_close {
            cur_frame += 1;

            self.world.update();
            let (dt, real_dt) = (self.world.dt(), self.world.real_dt());
            let systems = self.world.get_systems();

            let update_time = Duration::from_millis(
                *self
                    .config
                    .get_var_int_or("engine/gameplay/gameplay_update_tick_ms", 10)
                    as u64,
            );

            execution_time += dt;

            // Update input
            if let Some(mut iter) = replay_data_iter.as_mut() {
                systems
                    .input_system
                    .borrow_mut()
                    .update_from_replay(cur_frame, &mut iter);
            } else {
                systems.input_system.borrow_mut().update(window);
            }
            let actions = systems.input_system.borrow().get_action_list();

            if replay_data_iter.is_none() {
                self.replay_system.update(&actions);
            }

            if self
                .state_mgr
                .handle_actions(&actions, &self.world, &self.config)
            {
                self.should_close = true;
                break;
            }

            // Update game systems
            {
                #[cfg(prof_t)]
                let gameplay_start_t = SystemTime::now();

                let mut gameplay_system = systems.gameplay_system.borrow_mut();

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
            systems.audio_system.borrow_mut().update();

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

        self.on_close()?;

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

        let systems = self.world.get_systems();

        gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
        gfx::window::clear(window);
        systems.render_system.borrow_mut().update(
            window,
            &self.gfx_resources,
            &systems.gameplay_system.borrow().get_camera(),
            &systems.gameplay_system.borrow().get_renderable_entities(),
            frame_lag_normalized,
            smooth_by_extrapolating_velocity,
        );
        systems
            .ui_system
            .borrow_mut()
            .update(&real_dt, window, &mut self.gfx_resources);
        gfx::window::display(window);

        Ok(())
    }

    fn on_close(&self) -> Maybe_Error {
        let mut path = path::PathBuf::from(self.env.get_cwd());
        path.push("replay.dat");
        self.replay_system.serialize(&path)
    }
}
