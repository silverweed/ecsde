use crate::states::state::{Game_State_Args, Persistent_Game_State};
use inle_cfg::{self, Cfg_Value, Cfg_Var};
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_math::math;
use inle_math::vector::Vec2f;
use inle_win::window;
use std::convert::TryFrom;
use std::time::Duration;

pub struct Debug_Base_State {
    // @Cleanup: this cfg_var is already in Game_State!
    // Should we perhaps move it into Engine_State and read it from handle_actions?
    gameplay_update_tick_ms: Cfg_Var<f32>,
    digging: bool,
    t: Duration,
}

const CHANGE_SPEED_DELTA: f32 = 0.1;

impl Debug_Base_State {
    pub fn new(cfg: &inle_cfg::Config) -> Debug_Base_State {
        Debug_Base_State {
            gameplay_update_tick_ms: Cfg_Var::new("engine/gameplay/update_tick_ms", cfg),
            digging: false,
            t: Duration::default(),
        }
    }
}

macro_rules! add_msg {
    ($engine_state: expr, $msg: expr) => {
        $engine_state
            .debug_systems
            .debug_ui
            .get_overlay(sid!("msg"))
            .add_line($msg)
    };
}

impl Persistent_Game_State for Debug_Base_State {
    //fn update(&mut self, args: &mut Game_State_Args, dt: &Duration, _real_dt: &Duration) {
    //self.t += *dt;

    // self.blink_lights(args);
    //}

    fn handle_actions(&mut self, actions: &[Game_Action], args: &mut Game_State_Args) {
        let Game_State_Args {
            engine_state,
            gameplay_system: gs,
            game_resources,
            window,
            ..
        } = args;
        // @Speed: eventually we want to replace all the *name == sid with a const sid function, to allow doing
        // (sid!("game_speed_up"), Action_Kind::Pressed) => { ... }
        for action in actions {
            match action {
                (name, Action_Kind::Pressed)
                    if *name == sid!("game_speed_up") || *name == sid!("game_speed_down") =>
                {
                    let mut ts = engine_state.time.time_scale;
                    if action.0 == sid!("game_speed_up") {
                        ts *= 2.0;
                    } else {
                        ts *= 0.5;
                    }
                    ts = math::clamp(ts, 0.001, 32.0);
                    if ts > 0.0 {
                        engine_state.time.time_scale = ts;
                    }
                    add_msg!(
                        engine_state,
                        &format!("Time scale: {:.3}", engine_state.time.time_scale)
                    );
                }
                (name, Action_Kind::Pressed) if *name == sid!("pause_toggle") => {
                    engine_state.time.pause_toggle();
                    window::set_key_repeat_enabled(window, engine_state.time.paused);
                    add_msg!(
                        engine_state,
                        if engine_state.time.paused {
                            "Paused"
                        } else {
                            "Resumed"
                        }
                    );
                }
                (name, Action_Kind::Pressed) if *name == sid!("step_sim") => {
                    let step_delta = Duration::from_millis(
                        self.gameplay_update_tick_ms.read(&engine_state.config) as u64,
                    );
                    add_msg!(
                        engine_state,
                        &format!("Stepping of: {:.2} ms", step_delta.as_secs_f32() * 1000.0)
                    );
                    engine_state.time.paused = true;
                    engine_state.time.step(&step_delta);
                    gs.step(&step_delta, engine_state, game_resources, window);
                    gs.levels.foreach_active_level(|level| {
                        use inle_physics::physics;
                        let mut _ignored = physics::Collision_System_Debug_Data::default();
                        physics::update_collisions(
                            &mut level.world,
                            &level.chunks,
                            &mut level.phys_world,
                            &engine_state.systems.physics_settings,
                            &mut engine_state.systems.evt_register,
                            &mut engine_state.frame_alloc,
                            &mut _ignored,
                        );
                        let mut moved =
                            inle_alloc::temp::excl_temp_array(&mut engine_state.frame_alloc);
                        crate::movement_system::update(
                            &step_delta,
                            &mut level.world,
                            &level.phys_world,
                            &mut moved,
                        );
                        let moved = unsafe { moved.into_read_only() };
                        for mov in &moved {
                            level.chunks.update_collider(
                                mov.handle,
                                mov.prev_pos,
                                mov.new_pos,
                                mov.extent,
                                &mut engine_state.frame_alloc,
                            );
                        }
                    });
                }
                (name, Action_Kind::Pressed) if *name == sid!("print_em_debug_info") => {
                    //gs.print_debug_info();
                }
                (name, Action_Kind::Pressed) if *name == sid!("toggle_trace_overlay") => {
                    let show_trace = engine_state.debug_systems.show_overlay;
                    let show_trace = show_trace != inle_app::systems::Overlay_Shown::Trace;
                    if show_trace {
                        engine_state.debug_systems.show_overlay =
                            inle_app::systems::Overlay_Shown::Trace;
                    } else {
                        engine_state.debug_systems.show_overlay =
                            inle_app::systems::Overlay_Shown::None;
                    }
                    engine_state
                        .debug_systems
                        .debug_ui
                        .set_overlay_enabled(sid!("trace"), show_trace);
                }
                (name, Action_Kind::Pressed) if *name == sid!("toggle_threads_overlay") => {
                    let show_threads = engine_state.debug_systems.show_overlay;
                    let show_threads = show_threads != inle_app::systems::Overlay_Shown::Threads;
                    if show_threads {
                        engine_state
                            .debug_systems
                            .debug_ui
                            .set_overlay_enabled(sid!("trace"), false);
                        engine_state.debug_systems.show_overlay =
                            inle_app::systems::Overlay_Shown::Threads;
                    } else {
                        engine_state.debug_systems.show_overlay =
                            inle_app::systems::Overlay_Shown::None;
                    }
                }
                (name, Action_Kind::Pressed) if *name == sid!("move_camera_to_origin") => {
                    gs.levels
                        .foreach_active_level(|level| level.move_camera_to(Vec2f::new(0., 0.)));
                    add_msg!(engine_state, "Moved camera to origin");
                }
                (name, Action_Kind::Pressed) if *name == sid!("debug_dig") => {
                    self.digging = true;
                }
                (name, Action_Kind::Released) if *name == sid!("debug_dig") => {
                    self.digging = false;
                }
                (name, Action_Kind::Released) if *name == sid!("toggle_camera_on_player") => {
                    let cur: bool = bool::try_from(
                        engine_state
                            .config
                            .read_cfg(sid!("game/camera/on_player"))
                            .unwrap()
                            .clone(),
                    )
                    .unwrap();
                    engine_state
                        .config
                        .write_cfg(sid!("game/camera/on_player"), Cfg_Value::from(!cur))
                        .unwrap();
                    if cur {
                        add_msg!(engine_state, "Camera is now free.");
                    } else {
                        add_msg!(engine_state, "Camera is now following player.");
                    }
                }
                _ => {}
            }
        }
    }
}

/*
impl Debug_Base_State {
    fn update_digging(&mut self, args: &mut Game_State_Args) {
        use crate::systems::pixel_collision_system::C_Texture_Collider;
        use inle_common::colors;
        use inle_ecs::components::base::C_Spatial2D;
        use inle_ecs::entity_stream::new_entity_stream;
        use inle_gfx::{render, render_window};
        use inle_input::mouse;
        use inle_math::rect::Rect;
        use inle_math::vector::Vec2i;
        use std::time::Duration;

        if !self.digging {
            return;
        }

        let mouse_state = &args.engine_state.input_state.raw.mouse_state;
        let mleft = mouse::is_mouse_btn_pressed(mouse_state, mouse::Mouse_Button::Left);
        let mright = mouse::is_mouse_btn_pressed(mouse_state, mouse::Mouse_Button::Right);
        if mleft || mright {
            let mpos = render_window::mouse_pos_in_world(
                args.window,
                mouse_state,
                &args
                    .gameplay_system
                    .levels
                    .first_active_level()
                    .unwrap()
                    .get_camera()
                    .transform,
            );

            let world = &args
                .gameplay_system
                .levels
                .first_active_level()
                .unwrap() // assume we have an active level
                .world;
            let mut entity_stream = new_entity_stream(world)
                .require::<C_Texture_Collider>()
                .require::<C_Spatial2D>()
                .build();
            let tex_cld_entity = entity_stream.next(world);
            if tex_cld_entity.is_none() {
                return;
            }
            let tex_cld_entity = tex_cld_entity.unwrap();
            let texture = world
                .get_component::<C_Texture_Collider>(tex_cld_entity)
                .unwrap()
                .texture;
            let tex_transform = &world
                .get_component::<C_Spatial2D>(tex_cld_entity)
                .unwrap()
                .transform;

            let (tex_w, tex_h) =
                render::get_texture_size(args.game_resources.gfx.get_texture(texture));
            let pixel_collision_system = &mut args.gameplay_system.pixel_collision_system;

            const SIZE: u32 = 50;
            let mpos = Vec2i::from(mpos);
            // @Incomplete: not handling rotation/scale
            let tex_cld_pos = tex_transform.position();

            if Rect::new(
                tex_cld_pos.x as i32 - (tex_w as i32) / 2,
                tex_cld_pos.y as i32 - (tex_h as i32) / 2,
                2 * tex_w as i32,
                2 * tex_h as i32,
            )
            .contains(mpos)
            {
                pixel_collision_system.change_pixels_circle(
                    texture,
                    inle_math::shapes::Circle {
                        center: v2!(
                            (mpos.x + tex_w as i32 / 2) as f32,
                            (mpos.y + tex_h as i32 / 2) as f32
                        ) - tex_cld_pos,
                        radius: (SIZE / 2) as f32,
                    },
                    if mleft {
                        colors::TRANSPARENT
                    } else {
                        colors::rgb(102, 57, 49)
                    },
                    &mut args.game_resources.gfx,
                );
            }
        }
    }

    // DEBUG
    fn blink_lights(&mut self, args: &mut Game_State_Args) {
        if self.t > Duration::from_secs(1) {
            self.t = Duration::default();

            let rng = &mut args.engine_state.rng;
            args.gameplay_system.levels.foreach_active_level(|lv| {
                let mut pls = vec![];
                for pl in lv.lights.point_lights() {
                    pls.push(inle_gfx::light::Point_Light {
                        color: inle_common::colors::color_from_hex(inle_core::rand::rand_range(
                            rng,
                            0.0,
                            0xffffffffu32 as f32,
                        )
                            as u32),
                        ..*pl
                    });
                }

                let mut rls = vec![];
                for rl in lv.lights.rect_lights() {
                    rls.push(inle_gfx::light::Rect_Light {
                        color: inle_common::colors::color_from_hex(inle_core::rand::rand_range(
                            rng,
                            0.0,
                            0xffffffffu32 as f32,
                        )
                            as u32),
                        ..*rl
                    });
                }

                for (i, pl) in pls.iter().enumerate() {
                    lv.lights
                        .queue_command(inle_gfx::light::Light_Command::Change_Point_Light(i, *pl));
                }
                for (i, rl) in rls.iter().enumerate() {
                    lv.lights
                        .queue_command(inle_gfx::light::Light_Command::Change_Rect_Light(i, *rl));
                }
            });
        }
    }

}
*/
