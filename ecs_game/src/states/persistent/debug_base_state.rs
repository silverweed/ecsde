use crate::states::state::{Game_State_Args, Persistent_Game_State};
use ecs_engine::cfg::{self, Cfg_Var};
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::input::input_system::{Action_Kind, Game_Action};

pub struct Debug_Base_State {
    sid_game_speed_up: String_Id,
    sid_game_speed_down: String_Id,
    sid_pause_toggle: String_Id,
    sid_step_sim: String_Id,
    sid_print_em_debug_info: String_Id,
    sid_toggle_trace_overlay: String_Id,
    sid_move_camera_to_origin: String_Id,
    // @Cleanup: this cfg_var is already in Game_State!
    // Should we perhaps move it into Engine_State and read it from handle_actions?
    gameplay_update_tick_ms: Cfg_Var<f32>,
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
            gameplay_update_tick_ms: Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg),
        }
    }
}

impl Persistent_Game_State for Debug_Base_State {
    fn handle_actions(&mut self, actions: &[Game_Action], args: &mut Game_State_Args) -> bool {
        let Game_State_Args {
            engine_state,
            gameplay_system: gs,
            ..
        } = args;
        let debug_ui = &mut engine_state.debug_systems.debug_ui;

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
                    let msg_overlay = debug_ui.get_fadeout_overlay(String_Id::from("msg"));
                    msg_overlay
                        .add_line(&format!("Time scale: {:.2}", engine_state.time.time_scale));
                }
                (name, Action_Kind::Pressed) if *name == self.sid_pause_toggle => {
                    let msg_overlay = debug_ui.get_fadeout_overlay(String_Id::from("msg"));
                    engine_state.time.pause_toggle();
                    msg_overlay.add_line(if engine_state.time.paused {
                        "Paused"
                    } else {
                        "Resumed"
                    });
                }
                (name, Action_Kind::Pressed) if *name == self.sid_step_sim => {
                    let msg_overlay = debug_ui.get_fadeout_overlay(String_Id::from("msg"));
                    let step_delta = std::time::Duration::from_millis(
                        self.gameplay_update_tick_ms.read(&engine_state.config) as u64,
                    );
                    msg_overlay.add_line(&format!(
                        "Stepping of: {:.2} ms",
                        step_delta.as_secs_f32() * 1000.0
                    ));
                    engine_state.time.paused = true;
                    engine_state.time.step(&step_delta);
                    gs.step(&step_delta, &engine_state.config, &mut engine_state.rng);
                    gs.foreach_active_level(|level| {
                        use ecs_engine::collisions::physics;
                        let mut _ignored = physics::Collision_System_Debug_Data::default();
                        physics::update_collisions(&mut level.world, &level.chunks, &mut _ignored);
                        let mut moved = vec![];
                        crate::movement_system::update(&step_delta, &mut level.world, &mut moved);
                        for mov in moved {
                            level
                                .chunks
                                .update_entity(mov.entity, mov.prev_pos, mov.new_pos);
                        }
                    });
                }
                (name, Action_Kind::Pressed) if *name == self.sid_print_em_debug_info => {
                    //gs.print_debug_info();
                }
                (name, Action_Kind::Pressed) if *name == self.sid_toggle_trace_overlay => {
                    let show_trace = &mut engine_state.debug_systems.show_trace_overlay;
                    *show_trace = !*show_trace;
                    debug_ui.set_overlay_enabled(String_Id::from("trace"), *show_trace);
                }
                (name, Action_Kind::Pressed) if *name == self.sid_move_camera_to_origin => {
                    gs.foreach_active_level(|level| level.move_camera_to(Vec2f::new(0., 0.)));
                    let msg_overlay = debug_ui.get_fadeout_overlay(String_Id::from("msg"));
                    msg_overlay.add_line("Moved camera to origin");
                }
                _ => {}
            }
        }
        false
    }
}
