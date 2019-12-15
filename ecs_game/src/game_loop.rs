use super::{Game_Resources, Game_State};
use crate::gfx::render_system;
use ecs_engine::core::app;
use ecs_engine::core::common::colors;
use ecs_engine::core::common::Maybe_Error;
use ecs_engine::core::time;
use ecs_engine::gfx;
use ecs_engine::resources::gfx::Gfx_Resources;
use std::time::Duration;

#[cfg(debug_assertions)]
use ecs_engine::core::common::stringid::String_Id;
#[cfg(debug_assertions)]
use ecs_engine::debug;
#[cfg(debug_assertions)]
use ecs_engine::input;

pub fn tick_game<'a>(
    game_state: &'a mut Game_State<'a>,
    game_resources: &'a mut Game_Resources<'a>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let tracer = clone_tracer!(game_state.engine_state.tracer);
    trace!("tick_game", tracer);

    // @Speed: these should all be computed at compile time.
    // Probably will do that when either const fn or proc macros/syntax extensions are stable.
    #[cfg(debug_assertions)]
    let (sid_joysticks, sid_msg, sid_time, sid_fps, sid_entities) = (
        String_Id::from("joysticks"),
        String_Id::from("msg"),
        String_Id::from("time"),
        String_Id::from("fps"),
        String_Id::from("entities"),
    );

    let window = &mut game_state.window;
    let engine_state = &mut game_state.engine_state;
    engine_state.time.update();

    let dt = engine_state.time.dt();
    let real_dt = engine_state.time.real_dt();
    let systems = &mut engine_state.systems;
    #[cfg(debug_assertions)]
    let debug_systems = &mut engine_state.debug_systems;

    #[cfg(debug_assertions)]
    update_time_debug_overlay(
        debug_systems.debug_ui_system.get_overlay(sid_time),
        &engine_state.time,
    );
    let update_time = Duration::from_millis(
        game_state
            .gameplay_update_tick_ms
            .read(&engine_state.config) as u64,
    );

    game_state.execution_time += dt;

    // Check if the replay ended this frame
    if game_state.is_replaying && game_state.input_provider.is_realtime_player_input() {
        #[cfg(debug_assertions)]
        debug_systems
            .debug_ui_system
            .get_fadeout_overlay(sid_msg)
            .add_line("REPLAY HAS ENDED.");
        game_state.is_replaying = false;
    }

    // Update input
    {
        trace!("input_system::update", tracer);
        systems.input_system.update(
            window,
            &mut *game_state.input_provider,
            &engine_state.config,
        );
    }

    // Handle actions
    {
        trace!("app::handle_core_actions", tracer);
        if app::handle_core_actions(&systems.input_system.extract_core_actions(), window) {
            engine_state.should_close = true;
            return Ok(false);
        }
    }

    {
        let actions = systems.input_system.extract_game_actions();

        #[cfg(debug_assertions)]
        let input_system = &systems.input_system;
        #[cfg(debug_assertions)]
        let raw_events = input_system.get_raw_events();
        #[cfg(debug_assertions)]
        let (real_axes, joy_mask) = input_system.get_all_real_axes();

        #[cfg(debug_assertions)]
        {
            update_joystick_debug_overlay(
                debug_systems.debug_ui_system.get_overlay(sid_joysticks),
                real_axes,
                joy_mask,
            );

            // Only record replay data if we're not already playing back a replay.
            if debug_systems.replay_recording_system.is_recording()
                && game_state.input_provider.is_realtime_player_input()
            {
                let record_replay_data = game_state.record_replay.read(&engine_state.config);
                if record_replay_data {
                    debug_systems
                        .replay_recording_system
                        .update(raw_events, real_axes, joy_mask);
                }
            }
        }

        {
            trace!("state_mgr::handle_actions", tracer);
            if game_state.state_mgr.handle_actions(
                &actions,
                engine_state,
                &mut game_state.gameplay_system,
            ) {
                engine_state.should_close = true;
                return Ok(false);
            }
        }

        // Update game systems
        {
            trace!("game_update", tracer);

            let axes = engine_state.systems.input_system.get_virtual_axes();
            #[cfg(feature = "prof_gameplay")]
            let gameplay_start_t = std::time::Instant::now();
            #[cfg(feature = "prof_gameplay")]
            let mut n_gameplay_updates = 0;

            let gameplay_system = &mut game_state.gameplay_system;

            gameplay_system.realtime_update(
                &real_dt,
                &actions,
                axes,
                &engine_state.config,
                clone_tracer!(tracer),
            );
            while game_state.execution_time > update_time {
                gameplay_system.update(
                    &update_time,
                    &actions,
                    axes,
                    &engine_state.config,
                    clone_tracer!(tracer),
                );
                game_state.execution_time -= update_time;
                #[cfg(feature = "prof_gameplay")]
                {
                    n_gameplay_updates += 1;
                }
            }

            #[cfg(feature = "prof_gameplay")]
            println!(
                "[prof_gameplay] gameplay update took {} ms ({} updates, avg = {})",
                gameplay_start_t.elapsed().as_millis(),
                n_gameplay_updates,
                gameplay_start_t.elapsed().as_millis() / n_gameplay_updates,
            );
        }
    }

    // Update audio
    {
        trace!("audio_system_update", tracer);
        engine_state.systems.audio_system.update();
    }

    #[cfg(debug_assertions)]
    {
        let debug_systems = &mut engine_state.debug_systems;
        update_fps_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_fps),
            &game_state.fps_debug,
        );
        update_entities_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_entities),
            &game_state.gameplay_system.ecs_world,
        );
    }

    // Render
    #[cfg(feature = "prof_game_render")]
    let render_start_t = std::time::Instant::now();

    update_graphics(
        game_state,
        &mut game_resources.gfx,
        real_dt,
        time::duration_ratio(&game_state.execution_time, &update_time) as f32,
    )?;

    #[cfg(feature = "prof_game_render")]
    println!(
        "[prof_game_render] rendering took {} ms",
        render_start_t.elapsed().as_millis()
    );

    #[cfg(debug_assertions)]
    {
        let sleep = game_state
            .extra_frame_sleep_ms
            .read(&game_state.engine_state.config) as u64;
        std::thread::sleep(Duration::from_millis(sleep));

        game_state.engine_state.config.update();
        game_state.fps_debug.tick(&real_dt);
    }

    Ok(true)
}

fn update_graphics(
    game_state: &mut Game_State,
    gres: &mut Gfx_Resources,
    real_dt: Duration,
    frame_lag_normalized: f32,
) -> Maybe_Error {
    trace!("update_graphics", game_state.engine_state.tracer);

    let window = &mut game_state.window;
    let cfg = &game_state.engine_state.config;
    gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
    gfx::window::clear(window);
    let render_cfg = render_system::Render_System_Config {
        clear_color: colors::color_from_hex(game_state.clear_color.read(cfg) as u32),
        smooth_by_extrapolating_velocity: game_state.smooth_by_extrapolating_velocity.read(cfg),
        #[cfg(debug_assertions)]
        draw_sprites_bg: game_state.draw_sprites_bg.read(cfg),
        #[cfg(debug_assertions)]
        draw_sprites_bg_color: colors::color_from_hex(
            game_state.draw_sprites_bg_color.read(cfg) as u32
        ),
    };
    render_system::update(
        window,
        gres,
        &game_state.gameplay_system.get_camera(),
        game_state.gameplay_system.get_renderable_entities(),
        &game_state.gameplay_system.ecs_world,
        frame_lag_normalized,
        render_cfg,
        clone_tracer!(game_state.engine_state.tracer),
    );

    #[cfg(debug_assertions)]
    {
        trace!("debug_ui_system::update", game_state.engine_state.tracer);
        game_state
            .engine_state
            .debug_systems
            .debug_ui_system
            .update(&real_dt, window, gres);
    }

    {
        trace!("vsync", game_state.engine_state.tracer);
        gfx::window::display(window);
    }

    Ok(())
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
    use ecs_engine::core::time::to_secs_frac;

    debug_overlay.clear();

    debug_overlay.add_line_color(
        &format!(
            "[time] game: {:.2}, real: {:.2}, scale: {:.2}, paused: {}",
            to_secs_frac(&time.get_game_time()),
            to_secs_frac(&time.get_real_time()),
            time.time_scale,
            if time.paused { "yes" } else { "no" }
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

#[cfg(debug_assertions)]
fn update_entities_debug_overlay(
    debug_overlay: &mut debug::overlay::Debug_Overlay,
    ecs_world: &crate::ecs::entity_manager::Ecs_World,
) {
    debug_overlay.clear();
    debug_overlay.add_line_color(
        &format!("Entities: {}", ecs_world.entity_manager.n_live_entities()),
        colors::rgba(220, 100, 180, 220),
    );
}
