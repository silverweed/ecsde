use crate::states::state::{Game_State_Args, Persistent_Game_State};
use crate::systems::pixel_collision_system::C_Texture_Collider;
use inle_cfg::{self, Cfg_Var};
use inle_common::colors;
use inle_common::stringid::String_Id;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::entity_stream::new_entity_stream;
use inle_gfx::{render, render_window};
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_input::mouse;
use inle_math::math;
use inle_math::rect::Rect;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window;
use std::time::Duration;

pub struct Debug_Base_State {
    sid_game_speed_up: String_Id,
    sid_game_speed_down: String_Id,
    sid_pause_toggle: String_Id,
    sid_step_sim: String_Id,
    sid_print_em_debug_info: String_Id,
    sid_toggle_trace_overlay: String_Id,
    sid_move_camera_to_origin: String_Id,
    sid_debug_dig: String_Id,
    // @Cleanup: this cfg_var is already in Game_State!
    // Should we perhaps move it into Engine_State and read it from handle_actions?
    gameplay_update_tick_ms: Cfg_Var<f32>,
    digging: bool,
}

const CHANGE_SPEED_DELTA: f32 = 0.1;

impl Debug_Base_State {
    pub fn new(cfg: &inle_cfg::Config) -> Debug_Base_State {
        Debug_Base_State {
            sid_game_speed_up: sid!("game_speed_up"),
            sid_game_speed_down: sid!("game_speed_down"),
            sid_pause_toggle: sid!("pause_toggle"),
            sid_step_sim: sid!("step_sim"),
            sid_print_em_debug_info: sid!("print_em_debug_info"),
            sid_toggle_trace_overlay: sid!("toggle_trace_overlay"),
            sid_move_camera_to_origin: sid!("move_camera_to_origin"),
            sid_debug_dig: sid!("debug_dig"),
            gameplay_update_tick_ms: Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg),
            digging: false,
        }
    }
}

macro_rules! add_msg {
    ($engine_state: expr, $msg: expr) => {
        $engine_state
            .debug_systems
            .debug_ui
            .get_fadeout_overlay(sid!("msg"))
            .add_line($msg)
    };
}

impl Persistent_Game_State for Debug_Base_State {
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
                    if *name == self.sid_game_speed_up || *name == self.sid_game_speed_down =>
                {
                    let mut ts = engine_state.time.time_scale;
                    if action.0 == self.sid_game_speed_up {
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
                (name, Action_Kind::Pressed) if *name == self.sid_pause_toggle => {
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
                (name, Action_Kind::Pressed) if *name == self.sid_step_sim => {
                    let step_delta = std::time::Duration::from_millis(
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
                (name, Action_Kind::Pressed) if *name == self.sid_print_em_debug_info => {
                    //gs.print_debug_info();
                }
                (name, Action_Kind::Pressed) if *name == self.sid_toggle_trace_overlay => {
                    let show_trace = &mut engine_state.debug_systems.show_trace_overlay;
                    *show_trace = !*show_trace;
                    engine_state
                        .debug_systems
                        .debug_ui
                        .set_overlay_enabled(sid!("trace"), *show_trace);
                }
                (name, Action_Kind::Pressed) if *name == self.sid_move_camera_to_origin => {
                    gs.levels
                        .foreach_active_level(|level| level.move_camera_to(Vec2f::new(0., 0.)));
                    add_msg!(engine_state, "Moved camera to origin");
                }
                (name, Action_Kind::Pressed) if *name == self.sid_debug_dig => {
                    self.digging = true;
                }
                (name, Action_Kind::Released) if *name == self.sid_debug_dig => {
                    self.digging = false;
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, args: &mut Game_State_Args, _dt: &Duration, _real_dt: &Duration) {
        if !self.digging {
            return;
        }

        let mouse_state = &args.engine_state.input_state.raw.mouse_state;
        let mleft = mouse::is_mouse_btn_pressed(mouse_state, mouse::Mouse_Button::Left);
        let mright = mouse::is_mouse_btn_pressed(mouse_state, mouse::Mouse_Button::Right);
        if mleft || mright {
            let mpos = render_window::mouse_pos_in_world(
                args.window,
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
            // assume we have a texture collider (and it's the one we're interested in)
            let tex_cld_entity = entity_stream.next(world).unwrap();
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
}
