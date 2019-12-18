#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate anymap;
extern crate cgmath;
#[macro_use]
extern crate ecs_engine;
#[cfg(test)]
extern crate float_cmp;

mod controllable_system;
mod ecs;
mod game_loop;
mod gameplay_system;
mod gfx;
mod scene_tree;
mod states;

use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::common::rand;
use ecs_engine::core::{app, app_config};
use ecs_engine::gfx::{self as ngfx, window};
use ecs_engine::input;
use ecs_engine::resources;
use std::env;
use std::time::Duration;

#[cfg(debug_assertions)]
use ecs_engine::core::common::colors;
#[cfg(debug_assertions)]
use ecs_engine::core::common::stringid::String_Id;
#[cfg(debug_assertions)]
use ecs_engine::debug;

#[repr(C)]
pub struct Game_State<'a> {
    pub window: window::Window_Handle,
    pub engine_state: app::Engine_State<'a>,

    pub gameplay_system: gameplay_system::Gameplay_System,

    pub state_mgr: states::state_manager::State_Manager,
    #[cfg(debug_assertions)]
    pub fps_debug: debug::fps::Fps_Console_Printer,

    pub execution_time: Duration,
    pub input_provider: Box<dyn input::provider::Input_Provider>,
    pub is_replaying: bool,

    //// Cfg vars
    pub gameplay_update_tick_ms: Cfg_Var<i32>,
    pub smooth_by_extrapolating_velocity: Cfg_Var<bool>,
    pub clear_color: Cfg_Var<u32>,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg: Cfg_Var<bool>,
    #[cfg(debug_assertions)]
    pub draw_sprites_bg_color: Cfg_Var<u32>,
    #[cfg(debug_assertions)]
    pub extra_frame_sleep_ms: Cfg_Var<i32>,
    #[cfg(debug_assertions)]
    pub record_replay: Cfg_Var<bool>, // /engine/debug/replay/record
    #[cfg(debug_assertions)]
    pub trace_overlay_refresh_rate: Cfg_Var<f32>,

    pub rng: rand::Default_Rng,
}

#[repr(C)]
pub struct Game_Resources<'a> {
    pub gfx: resources::gfx::Gfx_Resources<'a>,
    pub audio: resources::audio::Audio_Resources<'a>,
}

#[repr(C)]
pub struct Game_Bundle<'a> {
    pub game_state: *mut Game_State<'a>,
    pub game_resources: *mut Game_Resources<'a>,
}

/////////////////////////////////////////////////////////////////////////////
//                        FOREIGN FUNCTION API                             //
/////////////////////////////////////////////////////////////////////////////

// Note: the lifetime is actually ignored. The Game_State/Resources's lifetime management is manual
// and it's performed by the game runner (the Game_State/Resources stay alive from game_init()
// to game_shutdown()).
#[no_mangle]
pub extern "C" fn game_init<'a>() -> Game_Bundle<'a> {
    eprintln!("[ INFO ] Initializing game...");
    if let Ok((game_state, game_resources)) = internal_game_init() {
        Game_Bundle {
            game_state: Box::into_raw(game_state),
            game_resources: Box::into_raw(game_resources),
        }
    } else {
        Game_Bundle {
            game_state: std::ptr::null_mut(),
            game_resources: std::ptr::null_mut(),
        }
    }
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_update<'a>(
    game_state: *mut Game_State<'a>,
    game_resources: *mut Game_Resources<'a>,
) -> bool {
    if game_state.is_null() || game_resources.is_null() {
        panic!("[ FATAL ] game_update: game state and/or resources are null!");
    }

    {
        let game_state = &mut *game_state;
        if game_state.engine_state.should_close {
            return false;
        }

        #[cfg(debug_assertions)]
        {
            game_state.engine_state.tracer.borrow_mut().start_frame();
        }

        {
            let game_resources = &mut *game_resources;
            if let Ok(true) = game_loop::tick_game(game_state, game_resources) {
                // All green
            } else {
                return false;
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        let game_state = &mut *game_state;
        app::maybe_update_trace_overlay(
            &mut game_state.engine_state,
            game_state.trace_overlay_refresh_rate,
        );
    }

    true
}

/// # Safety
/// Neither pointer is allowed to be null.
/// After calling this function, both pointers become invalid and must not be used anymore.
#[no_mangle]
pub unsafe extern "C" fn game_shutdown(
    game_state: *mut Game_State,
    game_resources: *mut Game_Resources,
) {
    use std::alloc::{dealloc, Layout};

    if game_state.is_null() || game_resources.is_null() {
        panic!("[ FATAL ] game_shutdown: game state and/or resources are null!");
    }

    std::ptr::drop_in_place(game_state);
    dealloc(game_state as *mut u8, Layout::new::<Game_State>());

    std::ptr::drop_in_place(game_resources);
    dealloc(game_resources as *mut u8, Layout::new::<Game_Resources>());

    eprintln!("[ OK ] Game was shut down.");
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {
}

/// # Safety
/// Neither pointer is allowed to be null.
#[no_mangle]
pub unsafe extern "C" fn game_reload(game_state: *mut Game_State, _game_res: *mut Game_Resources) {
    #[cfg(debug_assertions)]
    {
        if game_state.is_null() {
            panic!("[ FATAL ] game_reload: game state is null!");
        }

        let game_state = &mut *game_state;
        game_state
            .engine_state
            .debug_systems
            .debug_ui_system
            .get_fadeout_overlay(String_Id::from("msg"))
            .add_line_color("+++ GAME RELOADED +++", colors::rgb(255, 128, 0));
    }
}

/////////////////////////////////////////////////////////////////////////////
//                      END FOREIGN FUNCTION API                           //
/////////////////////////////////////////////////////////////////////////////

fn internal_game_init<'a>(
) -> Result<(Box<Game_State<'a>>, Box<Game_Resources<'a>>), Box<dyn std::error::Error>> {
    let mut game_resources = create_game_resources()?;
    let mut game_state = create_game_state(&mut game_resources)?;

    {
        let env = &game_state.engine_state.env;
        let gres = &mut game_resources.gfx;
        let cfg = &game_state.engine_state.config;

        game_state
            .gameplay_system
            .init(gres, env, &mut game_state.rng, cfg)?;
        init_states(
            &mut game_state.state_mgr,
            &mut game_state.engine_state,
            &mut game_state.gameplay_system,
        );

        #[cfg(debug_assertions)]
        {
            init_game_debug(&mut game_state, &mut game_resources);
        }
    }

    Ok((game_state, game_resources))
}

fn create_game_state<'a>(
    game_resources: &mut Game_Resources<'_>,
) -> Result<Box<Game_State<'a>>, Box<dyn std::error::Error>> {
    let cfg = app_config::App_Config::new(env::args());
    let window = ngfx::window::create_render_window(&(), cfg.target_win_size, &cfg.title);
    let mut engine_state = app::create_engine_state(cfg);

    println!(
        "Working dir = {:?}\nExe = {:?}",
        engine_state.env.get_cwd(),
        engine_state.env.get_exe()
    );

    app::init_engine_systems(&mut engine_state)?;
    app::start_config_watch(&engine_state.env, &mut engine_state.config)?;

    #[cfg(debug_assertions)]
    {
        app::init_engine_debug(&mut engine_state, &mut game_resources.gfx)?;
        app::start_recording(&mut engine_state)?;
    }

    let cfg = &engine_state.config;
    let input_provider = app::create_input_provider(&mut engine_state.replay_data, cfg);
    let is_replaying = !input_provider.is_realtime_player_input();
    let gameplay_update_tick_ms = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg);
    let smooth_by_extrapolating_velocity =
        Cfg_Var::new("engine/rendering/smooth_by_extrapolating_velocity", cfg);
    let clear_color = Cfg_Var::new("engine/rendering/clear_color", cfg);
    #[cfg(debug_assertions)]
    let draw_sprites_bg = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg", cfg);
    #[cfg(debug_assertions)]
    let draw_sprites_bg_color = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg_color", cfg);
    #[cfg(debug_assertions)]
    let extra_frame_sleep_ms = Cfg_Var::new("engine/debug/extra_frame_sleep_ms", cfg);
    #[cfg(debug_assertions)]
    let record_replay = Cfg_Var::new("engine/debug/replay/record", cfg);
    #[cfg(debug_assertions)]
    let trace_overlay_refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", cfg);

    Ok(Box::new(Game_State {
        window,
        engine_state,

        #[cfg(debug_assertions)]
        fps_debug: debug::fps::Fps_Console_Printer::new(&Duration::from_secs(2), "game"),

        execution_time: Duration::default(),
        input_provider,
        is_replaying,
        gameplay_system: gameplay_system::Gameplay_System::new(),
        state_mgr: states::state_manager::State_Manager::new(),
        rng: rand::new_rng()?,

        // Cfg_Vars
        gameplay_update_tick_ms,
        smooth_by_extrapolating_velocity,
        clear_color,
        #[cfg(debug_assertions)]
        draw_sprites_bg,
        #[cfg(debug_assertions)]
        draw_sprites_bg_color,
        #[cfg(debug_assertions)]
        extra_frame_sleep_ms,
        #[cfg(debug_assertions)]
        record_replay,
        #[cfg(debug_assertions)]
        trace_overlay_refresh_rate,
    }))
}

fn create_game_resources<'a>() -> Result<Box<Game_Resources<'a>>, Box<dyn std::error::Error>> {
    let gfx_resources = resources::gfx::Gfx_Resources::new();
    let audio_resources = resources::audio::Audio_Resources::new();
    Ok(Box::new(Game_Resources {
        gfx: gfx_resources,
        audio: audio_resources,
    }))
}

fn init_states(
    state_mgr: &mut states::state_manager::State_Manager,
    engine_state: &mut app::Engine_State,
    gs: &mut gameplay_system::Gameplay_System,
) {
    let base_state = Box::new(states::persistent::game_base_state::Game_Base_State {});
    state_mgr.add_persistent_state(engine_state, gs, base_state);
    #[cfg(debug_assertions)]
    {
        let debug_base_state = Box::new(
            states::persistent::debug_base_state::Debug_Base_State::new(&engine_state.config),
        );
        state_mgr.add_persistent_state(engine_state, gs, debug_base_state);
    }
}

#[cfg(debug_assertions)]
fn init_game_debug(game_state: &mut Game_State, game_resources: &mut Game_Resources) {
    use ecs_engine::core::common::vector::Vec2f;
    use ecs_engine::debug::overlay::Debug_Overlay_Config;
    use ecs_engine::gfx::align::Align;

    const FONT: &str = "Hack-Regular.ttf";

    let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui_system;
    let font = game_resources.gfx.load_font(&resources::gfx::font_path(
        &game_state.engine_state.env,
        FONT,
    ));

    // Entities overlay
    let overlay_cfg = Debug_Overlay_Config {
        row_spacing: 20.0,
        font_size: 16,
        pad_x: 5.0,
        pad_y: 5.0,
        background: colors::rgba(25, 25, 25, 210),
    };
    let mut overlay = debug_ui.create_overlay(String_Id::from("entities"), overlay_cfg, font);
    overlay.vert_align = Align::End;
    overlay.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(
        80.0,
        game_state.engine_state.app_config.target_win_size.1 as f32,
    );
}
