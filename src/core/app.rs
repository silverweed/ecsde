use super::app_config::App_Config;
use super::common::colors;
use super::common::Maybe_Error;
use super::env::Env_Info;
use super::time;
use super::world;
use crate::cfg::{self, Cfg_Var};
use crate::fs;
use crate::gfx;
use crate::gfx::align;
use crate::input;
use crate::replay::{recording_system, replay_data, replay_input_provider};
use crate::resources;
use crate::states;
use std::time::Duration;

#[cfg(debug_assertions)]
use super::common::stringid::String_Id;

#[cfg(debug_assertions)]
use crate::debug;

pub struct App<'r> {
    should_close: bool,

    env: Env_Info,
    config: cfg::Config,
    app_config: App_Config,

    state_mgr: states::state_manager::State_Manager,

    gfx_resources: resources::gfx::Gfx_Resources<'r>,
    audio_resources: resources::audio::Audio_Resources<'r>,

    world: world::World,

    replay_recording_system: recording_system::Replay_Recording_System,
    replay_data: Option<replay_data::Replay_Data>,
}

impl<'r> App<'r> {
    pub fn new(cfg: App_Config) -> Self {
        let env = Env_Info::gather().unwrap();
        let config = cfg::Config::new_from_dir(env.get_cfg_root());
        let world = world::World::new(&env);
        let replay_data = maybe_create_replay_data(&cfg);
        let ms_per_frame = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms");

        App {
            should_close: false,
            env,
            config,
            app_config: cfg,
            state_mgr: states::state_manager::State_Manager::new(),
            gfx_resources: resources::gfx::Gfx_Resources::new(),
            audio_resources: resources::audio::Audio_Resources::new(),
            world,
            replay_recording_system: recording_system::Replay_Recording_System::new(
                recording_system::Replay_Recording_System_Config { ms_per_frame },
            ),
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
        #[cfg(debug_assertions)]
        self.init_debug()?;
        self.world.init();

        Ok(())
    }

    fn init_states(&mut self) -> Maybe_Error {
        let base_state = Box::new(states::persistent::engine_base_state::Engine_Base_State {});
        self.state_mgr.add_persistent_state(&self.world, base_state);
        #[cfg(debug_assertions)]
        {
            let debug_base_state =
                Box::new(states::persistent::debug_base_state::Debug_Base_State::new());
            self.state_mgr
                .add_persistent_state(&self.world, debug_base_state);
        }
        Ok(())
    }

    fn init_all_systems(&mut self) -> Maybe_Error {
        let config_watcher = Box::new(cfg::sync::Config_Watch_Handler::new(&mut self.config));
        fs::file_watcher::start_file_watch(
            self.env.get_cfg_root().to_path_buf(),
            vec![config_watcher],
        )?;

        let systems = self.world.get_systems();

        systems.input_system.borrow_mut().init()?;

        systems
            .gameplay_system
            .borrow_mut()
            .init(&mut self.gfx_resources, &self.env)?;

        systems
            .render_system
            .borrow_mut()
            .init(gfx::render_system::Render_System_Config {
                clear_color: colors::rgb(22, 0, 22),
            })?;

        if cfg!(debug_assertions)
            && self.replay_data.is_none()
            && Cfg_Var::<bool>::new("engine/debug/replay/record").read()
        {
            self.replay_recording_system.start_recording_thread()?;
        }

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn init_debug(&mut self) -> Maybe_Error {
        use crate::core::common::vector::Vec2f;
        use debug::{fadeout_overlay, overlay};

        const FONT: &str = "Hack-Regular.ttf";

        let font = self
            .gfx_resources
            .load_font(&resources::gfx::font_path(&self.env, FONT));

        // @Robustness: add font validity check

        let mut debug_system = self.world.get_systems().debug_system.borrow_mut();
        // Debug overlays
        {
            let mut debug_overlay_config = overlay::Debug_Overlay_Config {
                row_spacing: 2.0,
                font_size: 20,
                pad_x: 5.0,
                pad_y: 5.0,
                background: colors::rgba(25, 25, 25, 210),
            };

            let mut joy_overlay = debug_system.create_overlay(
                String_Id::from("joysticks"),
                debug_overlay_config,
                font,
            );
            joy_overlay.horiz_align = align::Align::End;
            joy_overlay.position = Vec2f::new(self.app_config.target_win_size.0 as f32, 0.0);

            debug_overlay_config.font_size = 16;
            let mut time_overlay =
                debug_system.create_overlay(String_Id::from("time"), debug_overlay_config, font);
            time_overlay.horiz_align = align::Align::End;
            time_overlay.vert_align = align::Align::End;
            time_overlay.position = Vec2f::new(
                self.app_config.target_win_size.0 as f32,
                self.app_config.target_win_size.1 as f32,
            );

            let mut fps_overlay =
                debug_system.create_overlay(String_Id::from("fps"), debug_overlay_config, font);
            fps_overlay.vert_align = align::Align::End;
            fps_overlay.position = Vec2f::new(0.0, self.app_config.target_win_size.1 as f32);
        }

        // Debug fadeout overlays
        {
            let fadeout_overlay_config = fadeout_overlay::Fadeout_Debug_Overlay_Config {
                row_spacing: 2.0,
                font_size: 20,
                pad_x: 5.0,
                pad_y: 5.0,
                background: colors::rgba(25, 25, 25, 210),
                fadeout_time: Duration::from_secs(3),
                max_rows: 30,
            };

            let mut fadeout_overlay = debug_system.create_fadeout_overlay(
                String_Id::from("msg"),
                fadeout_overlay_config,
                font,
            );
            fadeout_overlay.horiz_align = align::Align::Begin;
            fadeout_overlay.position = Vec2f::new(0.0, 0.0);
        }

        Ok(())
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
                disable_input_during_replay: Cfg_Var::new(
                    "engine/debug/replay/disable_input_during_replay",
                ),
            };
            Box::new(replay_input_provider::Replay_Input_Provider::new(
                config,
                replay_data,
            ))
        } else {
            Box::new(input::default_input_provider::Default_Input_Provider::default())
        }
    }

    fn start_game_loop(&mut self, window: &mut gfx::window::Window_Handle) -> Maybe_Error {
        #[cfg(debug_assertions)]
        let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");

        let mut execution_time = Duration::new(0, 0);
        let mut input_provider = self.create_input_provider();
        let mut is_replaying = !input_provider.is_realtime_player_input();
        // Cfg vars
        let update_time = Cfg_Var::<i32>::new("engine/gameplay/gameplay_update_tick_ms");
        let smooth_by_extrapolating_velocity =
            Cfg_Var::<bool>::new("engine/rendering/smooth_by_extrapolating_velocity");
        #[cfg(debug_assertions)]
        let extra_frame_sleep_ms = Cfg_Var::<i32>::new("engine/debug/extra_frame_sleep_ms");
        #[cfg(debug_assertions)]
        let record_replay = Cfg_Var::<bool>::new("engine/debug/replay/record");

        #[cfg(debug_assertions)]
        let sid_joysticks = String_Id::from("joysticks");
        #[cfg(debug_assertions)]
        let sid_msg = String_Id::from("msg");
        #[cfg(debug_assertions)]
        let sid_time = String_Id::from("time");
        #[cfg(debug_assertions)]
        let sid_fps = String_Id::from("fps");

        while !self.should_close {
            self.world.update();

            let (dt, real_dt) = (self.world.dt(), self.world.real_dt());
            let systems = self.world.get_systems();

            #[cfg(debug_assertions)]
            update_time_debug_overlay(
                systems.debug_system.borrow_mut().get_overlay(sid_time),
                &self.world.time.borrow(),
            );
            let update_time = Duration::from_millis(update_time.read() as u64);

            execution_time += dt;

            // Check if the replay ended this frame
            if is_replaying && input_provider.is_realtime_player_input() {
                #[cfg(debug_assertions)]
                systems
                    .debug_system
                    .borrow_mut()
                    .get_fadeout_overlay(sid_msg)
                    .add_line("REPLAY HAS ENDED.");
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
                let (real_axes, joy_mask) = input_system.get_all_real_axes();

                #[cfg(debug_assertions)]
                {
                    update_joystick_debug_overlay(
                        systems.debug_system.borrow_mut().get_overlay(sid_joysticks),
                        real_axes,
                        joy_mask,
                    );

                    // Only record replay data if we're not already playing back a replay.
                    if self.replay_recording_system.is_recording()
                        && input_provider.is_realtime_player_input()
                    {
                        let record_replay_data = record_replay.read();
                        if record_replay_data {
                            self.replay_recording_system
                                .update(raw_events, real_axes, joy_mask);
                        }
                    }
                }

                if self.state_mgr.handle_actions(&actions, &self.world) {
                    self.should_close = true;
                    break;
                }

                // Update game systems
                {
                    #[cfg(prof_t)]
                    let gameplay_start_t = std::time::Instant::now();

                    let mut gameplay_system = systems.gameplay_system.borrow_mut();

                    gameplay_system.realtime_update(&real_dt, actions, axes);
                    while execution_time > update_time {
                        gameplay_system.update(&update_time, actions, axes);
                        execution_time -= update_time;
                    }

                    #[cfg(prof_t)]
                    println!("Gameplay: {} ms", gameplay_start_t.elapsed().as_millis());
                }
            }

            // Update audio
            systems.audio_system.borrow_mut().update();

            #[cfg(debug_assertions)]
            update_fps_debug_overlay(
                self.world
                    .get_systems()
                    .debug_system
                    .borrow_mut()
                    .get_overlay(sid_fps),
                &fps_debug,
            );

            // Render
            #[cfg(prof_t)]
            let render_start_t = std::time::Instant::now();

            self.update_graphics(
                window,
                real_dt,
                time::duration_ratio(&execution_time, &update_time) as f32,
                smooth_by_extrapolating_velocity.read(),
            )?;

            #[cfg(prof_t)]
            println!("Render: {} ms", render_start_t.elapsed().as_millis());

            #[cfg(debug_assertions)]
            {
                let sleep = extra_frame_sleep_ms.read() as u64;
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
        smooth_by_extrapolating_velocity: bool,
    ) -> Maybe_Error {
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

        #[cfg(debug_assertions)]
        systems
            .debug_system
            .borrow_mut()
            .update(&real_dt, window, &mut self.gfx_resources);

        gfx::window::display(window);

        Ok(())
    }

    fn on_game_loop_end(&self) -> Maybe_Error {
        Ok(())
    }
}

#[cfg(debug_assertions)]
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

#[cfg(not(debug_assertions))]
fn maybe_create_replay_data(cfg: &App_Config) -> Option<replay_data::Replay_Data> {
    None
}

#[cfg(debug_assertions)]
fn update_joystick_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    real_axes: &[input::joystick_mgr::Real_Axes_Values;
         input::bindings::joystick::JOY_COUNT as usize],
    joy_mask: u8,
) {
    use input::bindings::joystick;
    use std::convert::TryInto;

    debug_overlay.clear();

    for (joy_id, axes) in real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) != 0 {
            debug_overlay.add_line_color(&format!("> Joy {} <", joy_id), colors::rgb(235, 52, 216));

            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis: joystick::Joystick_Axis = i.try_into().unwrap_or_else(|err| {
                    panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                });
                debug_overlay.add_line_color(
                    &format!("{:?}: {:.2}", axis, axes[i as usize]),
                    colors::rgb(255, 255, 0),
                );
            }
        }
    }
}

#[cfg(debug_assertions)]
fn update_time_debug_overlay(debug_overlay: &mut debug::overlay::Debug_Overlay, time: &time::Time) {
    debug_overlay.clear();

    debug_overlay.add_line_color(
        &format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}",
            time.get_game_time(),
            time.get_real_time(),
            time.get_time_scale(),
            if time.is_paused() { "yes" } else { "no" }
        ),
        colors::rgb(100, 200, 200),
    );
}

#[cfg(debug_assertions)]
fn update_fps_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    fps: &debug::fps::Fps_Console_Printer,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!("FPS: {}", fps.get_fps() as u32),
        colors::rgba(180, 180, 180, 200),
    );
}
