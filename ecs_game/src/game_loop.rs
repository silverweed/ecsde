use super::Game_State;
use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::app::{self, Engine_State};
use ecs_engine::core::common::colors;
use ecs_engine::core::common::stringid::String_Id;
use ecs_engine::core::common::Maybe_Error;
use ecs_engine::core::time;
use ecs_engine::gfx;
use ecs_engine::input;
use std::time::Duration;

#[cfg(debug_assertions)]
use ecs_engine::debug;

pub fn tick_game(game_state: &mut Game_State) -> Result<bool, Box<dyn std::error::Error>> {
    // @Speed: these should all be computed at compile time.
    // Probably will do that when either const fn or proc macros/syntax extensions are stable.
    #[cfg(debug_assertions)]
    let (sid_joysticks, sid_msg, sid_time, sid_fps) = (
        String_Id::from("joysticks"),
        String_Id::from("msg"),
        String_Id::from("time"),
        String_Id::from("fps"),
    );

    let window = &mut game_state.window;
    let engine_state = &mut game_state.engine_state;
    engine_state.time.update();

    let (dt, real_dt) = (engine_state.time.dt(), engine_state.time.real_dt());
    let systems = &mut engine_state.systems;
    let debug_systems = &mut engine_state.debug_systems;

    #[cfg(debug_assertions)]
    update_time_debug_overlay(
        debug_systems.debug_ui_system.get_overlay(sid_time),
        &engine_state.time,
    );
    let update_time =
        Duration::from_millis(game_state.update_time.read(&engine_state.config) as u64);

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
    systems.input_system.update(
        window,
        &mut *game_state.input_provider,
        &engine_state.config,
    );

    // Handle actions
    if app::handle_core_actions(systems.input_system.get_core_actions(), window) {
        engine_state.should_close = true;
        return Ok(false);
    }

    {
        let input_system = &systems.input_system;
        let actions = input_system.get_game_actions();
        let axes = input_system.get_virtual_axes();
        let raw_events = input_system.get_raw_events();
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

        //if game_state.state_mgr.handle_actions(&actions, engine_state) {
        //engine_state.should_close = true;
        //return Ok(false);
        //}

        // Update game systems
        {
            #[cfg(prof_t)]
            let gameplay_start_t = std::time::Instant::now();

            let gameplay_system = &mut game_state.gameplay_system;

            gameplay_system.realtime_update(&real_dt, actions, axes, &engine_state.config);
            while game_state.execution_time > update_time {
                gameplay_system.update(&update_time, actions, axes, &engine_state.config);
                game_state.execution_time -= update_time;
            }

            #[cfg(prof_t)]
            println!("Gameplay: {} ms", gameplay_start_t.elapsed().as_millis());
        }
    }

    // Update audio
    systems.audio_system.update();

    #[cfg(debug_assertions)]
    update_fps_debug_overlay(
        debug_systems.debug_ui_system.get_overlay(sid_fps),
        &game_state.fps_debug,
    );

    // Render
    #[cfg(prof_t)]
    let render_start_t = std::time::Instant::now();

    let smooth_by_extrapolating_velocity = game_state
        .smooth_by_extrapolating_velocity
        .read(&engine_state.config);
    update_graphics(
        game_state,
        real_dt,
        time::duration_ratio(&game_state.execution_time, &update_time) as f32,
        smooth_by_extrapolating_velocity,
    )?;

    #[cfg(prof_t)]
    println!("Render: {} ms", render_start_t.elapsed().as_millis());

    #[cfg(debug_assertions)]
    {
        let sleep = game_state
            .extra_frame_sleep_ms
            .read(&game_state.engine_state.config) as u64;
        std::thread::sleep(Duration::from_millis(sleep));
    }

    game_state.engine_state.config.update();

    #[cfg(debug_assertions)]
    game_state.fps_debug.tick(&real_dt);

    Ok(true)
}

fn update_graphics(
    game_state: &mut Game_State,
    real_dt: Duration,
    frame_lag_normalized: f32,
    smooth_by_extrapolating_velocity: bool,
) -> Maybe_Error {
    let window = &mut game_state.window;
    gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
    gfx::window::clear(window);
    game_state.render_system.update(
        window,
        &game_state.engine_state.gfx_resources,
        &game_state.gameplay_system.get_camera(),
        &game_state.gameplay_system.get_renderable_entities(),
        frame_lag_normalized,
        smooth_by_extrapolating_velocity,
    );

    #[cfg(debug_assertions)]
    game_state
        .engine_state
        .debug_systems
        .debug_ui_system
        .update(&real_dt, window, &mut game_state.engine_state.gfx_resources);

    gfx::window::display(window);

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
