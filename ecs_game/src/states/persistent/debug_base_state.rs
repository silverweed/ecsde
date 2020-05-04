use crate::states::state::{Game_State_Args, Persistent_Game_State};
use crate::systems::pixel_collision_system::C_Texture_Collider;
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::common::colors;
use ecs_engine::common::rect::Rect;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::vector::{Vec2f, Vec2i};
use ecs_engine::gfx::render;
use ecs_engine::gfx::window;
use ecs_engine::input::bindings::mouse;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};

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
    pub fn new(cfg: &cfg::Config) -> Debug_Base_State {
        Debug_Base_State {
            sid_game_speed_up: String_Id::from("game_speed_up"),
            sid_game_speed_down: String_Id::from("game_speed_down"),
            sid_pause_toggle: String_Id::from("pause_toggle"),
            sid_step_sim: String_Id::from("step_sim"),
            sid_print_em_debug_info: String_Id::from("print_em_debug_info"),
            sid_toggle_trace_overlay: String_Id::from("toggle_trace_overlay"),
            sid_move_camera_to_origin: String_Id::from("move_camera_to_origin"),
            sid_debug_dig: String_Id::from("debug_dig"),
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
            .get_fadeout_overlay(String_Id::from("msg"))
            .add_line($msg)
    };
}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(&mut self, actions: &[Game_Action], args: &mut Game_State_Args) -> bool {
        let Game_State_Args {
            engine_state,
            gameplay_system: gs,
            game_resources,
            window,
            ..
        } = args;
        // @Speed: eventually we want to replace all the *name == sid with a const sid function, to allow doing
        // (sid!("game_speed_up"), Action_Kind::Pressed) => { ... }
        for action in actions.iter() {
            match action {
                (name, Action_Kind::Pressed)
                    if *name == self.sid_game_speed_up || *name == self.sid_game_speed_down =>
                {
                    let ts = engine_state.time.time_scale
                        + CHANGE_SPEED_DELTA
                            * if action.0 == self.sid_game_speed_up {
                                1.0
                            } else {
                                -1.0
                            };
                    if ts > 0.0 {
                        engine_state.time.time_scale = ts;
                    }
                    add_msg!(
                        engine_state,
                        &format!("Time scale: {:.2}", engine_state.time.time_scale)
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
                    gs.step(&step_delta, engine_state, game_resources);
                    gs.levels.foreach_active_level(|level| {
                        use ecs_engine::collisions::physics;
                        let mut _ignored = physics::Collision_System_Debug_Data::default();
                        physics::update_collisions(&mut level.world, &level.chunks, &mut _ignored);
                        let mut moved =
                            ecs_engine::alloc::temp::excl_temp_array(&mut engine_state.frame_alloc);
                        crate::movement_system::update(&step_delta, &mut level.world, &mut moved);
                        let moved = unsafe { moved.into_read_only() };
                        for mov in &moved {
                            level.chunks.update_entity(
                                mov.entity,
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
                        .set_overlay_enabled(String_Id::from("trace"), *show_trace);
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
        false
    }

    fn update(&mut self, args: &mut Game_State_Args) {
        if !self.digging {
            return;
        }

        let mleft = mouse::is_mouse_btn_pressed(mouse::Mouse_Button::Left);
        let mright = mouse::is_mouse_btn_pressed(mouse::Mouse_Button::Right);
        if mleft || mright {
            let mpos = window::mouse_pos_in_world(
                args.window,
                &args
                    .gameplay_system
                    .levels
                    .first_active_level()
                    .unwrap()
                    .get_camera()
                    .transform,
            );

            let texture = args
                .gameplay_system
                .levels
                .first_active_level()
                .unwrap() // assume we have an active level
                .world
                .get_components::<C_Texture_Collider>()
                .next()
                .unwrap() // assume we have a texture collider (and it's the one we're interested in)
                .texture;
            let (tex_w, tex_h) =
                render::get_texture_size(args.game_resources.gfx.get_texture(texture));
            let pixel_collision_system = &mut args.gameplay_system.pixel_collision_system;

            const SIZE: u32 = 50;
            let mpos = Vec2i::from(mpos);

            if Rect::new(
                -(tex_w as i32) / 2,
                -(tex_h as i32) / 2,
                2 * tex_w as i32,
                2 * tex_h as i32,
            )
            .contains(mpos)
            {
                pixel_collision_system.change_pixels_circle(
                    texture,
                    ecs_engine::common::shapes::Circle {
                        center: v2!(
                            (mpos.x + tex_w as i32 / 2) as f32,
                            (mpos.y + tex_h as i32 / 2) as f32
                        ),
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
