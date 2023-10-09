use super::Phase_Args;
use inle_app::debug::systems::Overlay_Shown;
use inle_app::phases::{Persistent_Game_Phase, Phase_Id};
use inle_input::input_state::{Action_Kind, Game_Action};
use inle_win::window::Camera;
use std::ops::DerefMut;

#[derive(Default)]
pub struct Debug;

impl Debug {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("debug");
}

macro_rules! add_msg {
    ($game_state: expr, $msg: expr) => {
        $game_state
            .debug_systems
            .debug_ui
            .get_overlay(sid!("msg"))
            .add_line($msg)
    };
}

impl Persistent_Game_Phase for Debug {
    type Args = Phase_Args;

    fn update(&mut self, args: &mut Phase_Args) {
        let mut game_state = args.game_state_mut();
        let gs = game_state.deref_mut();

        for action in &gs.input.processed.game_actions {
            match action {
                (name, Action_Kind::Pressed) if *name == sid!("calipers") => {
                    // @Incomplete
                    let camera = Camera::default();
                    gs.debug_systems
                        .calipers
                        .start_measuring_dist(&gs.window, &camera, &gs.input);
                }
                (name, Action_Kind::Released) if *name == sid!("calipers") => {
                    gs.debug_systems.calipers.end_measuring();
                }
                (name, Action_Kind::Pressed)
                    if *name == sid!("game_speed_up") || *name == sid!("game_speed_down") =>
                {
                    let mut ts = gs.time.time_scale;
                    if action.0 == sid!("game_speed_up") {
                        ts *= 2.0;
                    } else {
                        ts *= 0.5;
                    }
                    ts = inle_math::math::clamp(ts, 0.001, 32.0);
                    if ts > 0.0 {
                        gs.time.time_scale = ts;
                    }
                    add_msg!(gs, &format!("Time scale: {:.3}", gs.time.time_scale));
                }
                (name, Action_Kind::Pressed) if *name == sid!("pause_toggle") => {
                    gs.time.pause_toggle();
                    inle_win::window::set_key_repeat_enabled(&mut gs.window, gs.time.paused);
                    add_msg!(gs, if gs.time.paused { "Paused" } else { "Resumed" });
                }
                (name, Action_Kind::Pressed) if *name == sid!("step_sim") => {}
                (name, Action_Kind::Pressed) if *name == sid!("toggle_trace_overlay") => {
                    let show_trace = gs.debug_systems.show_overlay;
                    let show_trace = show_trace != Overlay_Shown::Trace;
                    if show_trace {
                        gs.debug_systems.show_overlay = Overlay_Shown::Trace;
                    } else {
                        gs.debug_systems.show_overlay = Overlay_Shown::None;
                    }
                    gs.debug_systems
                        .debug_ui
                        .set_overlay_enabled(sid!("trace"), show_trace);
                }
                (name, Action_Kind::Pressed) if *name == sid!("toggle_threads_overlay") => {
                    let show_threads = gs.debug_systems.show_overlay;
                    let show_threads = show_threads != Overlay_Shown::Threads;
                    if show_threads {
                        gs.debug_systems
                            .debug_ui
                            .set_overlay_enabled(sid!("trace"), false);
                        gs.debug_systems.show_overlay = Overlay_Shown::Threads;
                    } else {
                        gs.debug_systems.show_overlay = Overlay_Shown::None;
                    }
                }
                (name, Action_Kind::Pressed) if *name == sid!("move_camera_to_origin") => {
                    add_msg!(gs, "Moved camera to origin");
                }
                (name, Action_Kind::Released) if *name == sid!("toggle_camera_on_player") => {
                    gs.free_camera = !gs.free_camera;
                    if !gs.free_camera {
                        gs.camera.transform = inle_math::transform::Transform2D::default();
                    }
                }
                (name, Action_Kind::Pressed) if *name == sid!("toggle_overlays") => {
                    gs.config
                        .toggle_cfg(sid!("engine/debug/overlay/display"))
                        .unwrap_or_else(|err| lerr!("{}", err));
                }
                _ => {}
            }
        }
    }
}
