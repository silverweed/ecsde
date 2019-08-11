use super::app_config::App_Config;
use super::common::colors;
use super::common::Maybe_Error;
use super::env::Env_Info;
use super::msg::Msg_Responder;
use super::time;
use super::world;
use crate::audio;
use crate::cfg;
use crate::fs;
use crate::gfx;
use crate::input;
use crate::replay::{recording_system, replay_data, replay_input_provider};
use crate::resources;
use crate::states;
use std::time::Duration;

#[cfg(debug_assertions)]
use crate::debug;

pub struct App<'r> {
    should_close: bool,

    env: Env_Info,
    config: cfg::Config,

    state_mgr: states::state_manager::State_Manager,

    gfx_resources: resources::gfx::Gfx_Resources<'r>,
    audio_resources: resources::audio::Audio_Resources<'r>,

    world: world::World,

    replay_recording_system: recording_system::Replay_Recording_System,
    replay_data: Option<replay_data::Replay_Data>,

    #[cfg(debug_assertions)]
    debug_overlay: debug::overlay::Debug_Overlay,
}

impl<'r> App<'r> {
    pub fn new(cfg: &App_Config, sound_loader: &'r audio::sound_loader::Sound_Loader) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());
        let world = world::World::new(&env);
        let replay_data = maybe_create_replay_data(cfg);
        let ms_per_frame = *config
            .get_var_int("engine/gameplay/gameplay_update_tick_ms")
            .expect("[ FATAL ] engine/gameplay/gameplay_update_tick_ms not found in config file!")
            as u16;

        App {
            should_close: false,
            env,
            config,
            state_mgr: states::state_manager::State_Manager::new(),
            gfx_resources: resources::gfx::Gfx_Resources::new(),
            audio_resources: resources::audio::Audio_Resources::new(sound_loader),
            world,
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config { ms_per_frame },
            ),
            replay_data,

            #[cfg(debug_assertions)]
            debug_overlay: debug::overlay::Debug_Overlay::new(),
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
        #[cfg(debug_assertions)]
        self.init_debug()?;
        self.world.init();

        Ok(())
    }

    fn init_states(&mut self) -> Maybe_Error {
        let base_state = Box::new(states::persistent::engine_base_state::Engine_Base_State {});
        self.state_mgr.add_persistent_state(&self.world, base_state);
        let debug_base_state =
            Box::new(states::persistent::debug_base_state::Debug_Base_State::new());
        self.state_mgr
            .add_persistent_state(&self.world, debug_base_state);
        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&mut self.config));
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

        if self.replay_data.is_none()
            && *self
                .config
                .get_var_bool_or("engine/debug/replay/record", false)
        {
            self.replay_recording_system
                .start_recording_thread(&self.config)?;
        }

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn init_debug(&mut self) -> Maybe_Error {
        self.debug_overlay.init(&self.env, &mut self.gfx_resources)
    }

    pub fn run(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        self.start_game_loop(window)?;
        Ok(())
    }

    fn create_input_provider(&mut self) -> Box<dyn input::provider::Input_Provider> {
        // Consumes self.replay_data!
        let replay_data = self.replay_data.take();
        if let Some(replay_data) = replay_data {
            let config = replay_input_provider::Replay_Input_Provider_Config {
                disable_input_during_replay: self
                    .config
                    .get_var_bool_or("engine/debug/replay/disable_input_during_replay", false),
            };
            Box::new(replay_input_provider::Replay_Input_Provider::new(
                config,
                replay_data,
            ))
        } else {
            Box::new(input::default_input_provider::Default_Input_Provider::new())
        }
    }

    fn start_game_loop(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        #[cfg(debug_assertions)]
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");

        let mut execution_time = Duration::new(0, 0);
        let mut input_provider = self.create_input_provider();
        let mut is_replaying = !input_provider.is_realtime_player_input();

        while !self.should_close {
            self.world.update();
            let (dt, real_dt) = (self.world.dt(), self.world.real_dt());
            let systems = self.world.get_systems();

            let update_time = Duration::from_millis(
                *self
                    .config
                    .get_var_int("engine/gameplay/gameplay_update_tick_ms")
                    .expect("[ FATAL ] engine/gameplay/gameplay_update_tick_ms not found in config file!")
                    as u64,
            );

            execution_time += dt;

            // Check if the replay ended this frame
            if is_replaying && input_provider.is_realtime_player_input() {
                systems
                    .ui_system
                    .borrow_mut()
                    .send_message(gfx::ui::UI_Request::Add_Fadeout_Text(String::from(
                        "REPLAY HAS ENDED.",
                    )));
                is_replaying = false;
            }

            // Update input
            systems
                .input_system
                .borrow_mut()
                .update(window, &mut *input_provider);

            // Handle actions
            if Self::handle_core_actions(systems.input_system.borrow().get_core_actions(), window) {
                self.should_close = true;
                break;
            }

            {
                let input_system = systems.input_system.borrow();
                let actions = input_system.get_game_actions();
                let axes = input_system.get_virtual_axes();
                let raw_events = input_system.get_raw_events();
                // @Temporary :joystick:
                let real_axes = input_system
                    .get_real_axes(crate::input::bindings::joystick::Joystick {
                        id: 0,
                        joy_type: crate::input::bindings::joystick::Joystick_Type::XBox360,
                    })
                    .unwrap();

                #[cfg(debug_assertions)]
                update_debug_overlay(&mut self.debug_overlay, real_axes);

                // Only record replay data if we're not already playing back a replay.
                if self.replay_recording_system.is_recording()
                    && input_provider.is_realtime_player_input()
                {
                    let record_replay_data = *self
                        .config
                        .get_var_bool_or("engine/debug/replay/record", false);
                    if record_replay_data {
                        self.replay_recording_system.update(raw_events, real_axes);
                    }
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

                    gameplay_system.realtime_update(&real_dt, actions, axes);
                    while execution_time > update_time {
                        gameplay_system.update(&update_time, actions, axes);
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

            #[cfg(debug_assertions)]
            fps_debug.tick(&real_dt);
        }

        self.on_game_loop_end()?;

        Ok(())
    }

    fn handle_core_actions(
        actions: &[input::core_actions::Core_Action],
        window: &mut gfx::window::Window_Handle,
    ) -> bool {
        use input::core_actions::Core_Action;

        for action in actions.iter() {
            match action {
                Core_Action::Quit => return true,
                Core_Action::Resize(new_width, new_height) => {
                    gfx::window::resize_keep_ratio(window, *new_width, *new_height)
                }
            }
        }

        false
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

        #[cfg(debug_assertions)]
        self.debug_overlay.draw(window, &mut self.gfx_resources);

        gfx::window::display(window);

        Ok(())
    }

    fn on_game_loop_end(&self) -> Maybe_Error {
        Ok(())
    }
}

fn maybe_create_replay_data(cfg: &App_Config) -> Option<replay_data::Replay_Data> {
    if let Some(path) = &cfg.replay_file {
        match replay_data::Replay_Data::from_file(&path) {
            Ok(data) => Some(data),
            Err(err) => {
                eprintln!(
                    "[ ERROR ] Failed to load replay data from {:?}: {}",
                    path, err
                );
                None
            }
        }
    } else {
        None
    }
}

#[cfg(debug_assertions)]
fn update_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    real_axes: &input::joystick_mgr::Real_Axes_Values,
) {
    use input::bindings::joystick;
    use std::convert::TryInto;

    debug_overlay.clear();
    for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
        let axis: joystick::Joystick_Axis = i.try_into().unwrap_or_else(|err| {
            panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
        });
        debug_overlay.add_line_col(
            &format!("{:?}: {:.2}", axis, real_axes[i as usize]),
            colors::rgb(255, 255, 0),
        );
    }
}
