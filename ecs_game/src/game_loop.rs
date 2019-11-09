pub fn start_game_loop(
    engine_state: &mut Engine_State<'_>,
    window: &mut gfx::window::Window_Handle,
) -> Maybe_Error {
    #[cfg(debug_assertions)]
    let mut fps_debug = debug::fps::Fps_Console_Printer::new(&Duration::from_secs(3), "main");

    let mut execution_time = Duration::new(0, 0);
    let mut input_provider = create_input_provider(&mut engine_state.replay_data);
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

    while !engine_state.should_close {
        engine_state.time.update();

        let (dt, real_dt) = (engine_state.time.dt(), engine_state.time.real_dt());
        let systems = &mut engine_state.systems;
        let debug_systems = &mut engine_state.debug_systems;

        #[cfg(debug_assertions)]
        update_time_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_time),
            &engine_state.time,
        );
        let update_time = Duration::from_millis(update_time.read() as u64);

        execution_time += dt;

        // Check if the replay ended this frame
        if is_replaying && input_provider.is_realtime_player_input() {
            #[cfg(debug_assertions)]
            debug_systems
                .debug_ui_system
                .get_fadeout_overlay(sid_msg)
                .add_line("REPLAY HAS ENDED.");
            is_replaying = false;
        }

        // Update input
        systems.input_system.update(window, &mut *input_provider);

        // Handle actions
        if handle_core_actions(systems.input_system.get_core_actions(), window) {
            engine_state.should_close = true;
            break;
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
                    && input_provider.is_realtime_player_input()
                {
                    let record_replay_data = record_replay.read();
                    if record_replay_data {
                        debug_systems
                            .replay_recording_system
                            .update(raw_events, real_axes, joy_mask);
                    }
                }
            }

            //if self.state_mgr.handle_actions(&actions, &self.world) {
            //should_close = true;
            //break;
            //}

            // Update game systems
            {
                #[cfg(prof_t)]
                let gameplay_start_t = std::time::Instant::now();

                let gameplay_system = &mut systems.gameplay_system;

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
        systems.audio_system.update();

        #[cfg(debug_assertions)]
        update_fps_debug_overlay(
            debug_systems.debug_ui_system.get_overlay(sid_fps),
            &fps_debug,
        );

        // Render
        #[cfg(prof_t)]
        let render_start_t = std::time::Instant::now();

        update_graphics(
            window,
            engine_state,
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

        engine_state.config.update();

        #[cfg(debug_assertions)]
        fps_debug.tick(&real_dt);
    }

    Ok(())
}

fn update_graphics(
    window: &mut gfx::window::Window_Handle,
    engine_state: &mut Engine_State,
    real_dt: Duration,
    frame_lag_normalized: f32,
    smooth_by_extrapolating_velocity: bool,
) -> Maybe_Error {
    gfx::window::set_clear_color(window, colors::rgb(0, 0, 0));
    gfx::window::clear(window);
    let systems = &mut engine_state.systems;
    systems.render_system.update(
        window,
        &engine_state.gfx_resources,
        &systems.gameplay_system.get_camera(),
        &systems.gameplay_system.get_renderable_entities(),
        frame_lag_normalized,
        smooth_by_extrapolating_velocity,
    );

    #[cfg(debug_assertions)]
    engine_state.debug_systems.debug_ui_system.update(
        &real_dt,
        window,
        &mut engine_state.gfx_resources,
    );

    gfx::window::display(window);

    Ok(())
}
