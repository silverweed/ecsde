#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

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
mod states;

use ecs_engine::cfg;
use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::core::{app, app_config};
use ecs_engine::gfx::{self as ngfx, window};
use ecs_engine::input;
use ecs_engine::resources;
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
    pub debug_cvars: Debug_CVars,

    pub rng: rand::Default_Rng,
}

#[cfg(debug_assertions)]
pub struct Debug_CVars {
    pub draw_sprites_bg: Cfg_Var<bool>,
    pub draw_sprites_bg_color: Cfg_Var<u32>,

    pub extra_frame_sleep_ms: Cfg_Var<i32>,

    pub record_replay: Cfg_Var<bool>,

    pub trace_overlay_refresh_rate: Cfg_Var<f32>,

    pub draw_colliders: Cfg_Var<bool>,
    pub draw_collision_quadtree: Cfg_Var<bool>,

    pub draw_entities: Cfg_Var<bool>,
    pub draw_entities_velocities: Cfg_Var<bool>,

    pub draw_debug_grid: Cfg_Var<bool>,
    pub debug_grid_square_size: Cfg_Var<f32>,
    pub debug_grid_opacity: Cfg_Var<i32>,
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
/// `raw_args` is a pointer to the first command-line argument given to the game runner,
/// `args_count` is the total number of arguments.
/// # Safety
/// If `args_count` > 0, `raw_args` must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn game_init<'a>(
    raw_args: *const String,
    args_count: usize,
) -> Game_Bundle<'a> {
    eprintln!("[ INFO ] Initializing game...");

    let mut args: Vec<&String> = Vec::with_capacity(args_count);
    for i in 0..args_count {
        let arg = raw_args.add(i);
        args.push(&*arg);
    }

    if let Ok((game_state, game_resources)) = internal_game_init(args) {
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
            game_state.debug_cvars.trace_overlay_refresh_rate,
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
    args: Vec<&String>,
) -> Result<(Box<Game_State<'a>>, Box<Game_Resources<'a>>), Box<dyn std::error::Error>> {
    let mut game_resources = create_game_resources()?;
    let mut game_state = create_game_state(&mut game_resources, args)?;

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
    cmdline_args: Vec<&String>,
) -> Result<Box<Game_State<'a>>, Box<dyn std::error::Error>> {
    // Load Config first, as it's needed to setup everything that follows.
    let env = Env_Info::gather().unwrap();
    let config = cfg::Config::new_from_dir(env.get_cfg_root());

    // Load initial App_Config (some values may be overwritten by cmdline args)
    let mut app_config = {
        let cfg = &config;
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", cfg);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", cfg);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", cfg);
        app_config::App_Config {
            title: win_title.read(cfg),
            target_win_size: (win_width.read(cfg) as u32, win_height.read(cfg) as u32),
            #[cfg(debug_assertions)]
            in_replay_file: None,
        }
    };
    app_config::maybe_override_with_cmdline_args(&mut app_config, cmdline_args.into_iter());

    let mut engine_state = app::create_engine_state(env, config, app_config);

    let appcfg = &engine_state.app_config;
    let window = ngfx::window::create_render_window(&(), appcfg.target_win_size, &appcfg.title);

    #[cfg(debug_assertions)]
    {
        if let Some(in_replay_file) = &appcfg.in_replay_file {
            eprintln!("[ INFO ] Loading replay file {:?}", in_replay_file);
            engine_state.replay_data = app::try_create_replay_data(in_replay_file);
        }
    }

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

    #[cfg(debug_assertions)]
    let input_provider = app::create_input_provider(&mut engine_state.replay_data, cfg);
    #[cfg(not(debug_assertions))]
    let input_provider = app::create_input_provider(cfg);

    let is_replaying = !input_provider.is_realtime_player_input();
    let gameplay_update_tick_ms = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg);
    let smooth_by_extrapolating_velocity =
        Cfg_Var::new("engine/rendering/smooth_by_extrapolating_velocity", cfg);
    let clear_color = Cfg_Var::new("engine/rendering/clear_color", cfg);
    #[cfg(debug_assertions)]
    let debug_cvars = create_debug_cvars(cfg);

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
        #[cfg(debug_assertions)]
        rng: rand::new_rng_with_seed([
            0x12, 0x23, 0x33, 0x44, 0x44, 0xab, 0xbc, 0xcc, 0x45, 0x21, 0x72, 0x21, 0xfe, 0x31,
            0xdf, 0x46, 0xfe, 0xb4, 0x2a, 0xa9, 0x47, 0xdd, 0xd1, 0x37, 0x80, 0xfc, 0x22, 0xa1,
            0xa2, 0xb3, 0xc0, 0xfe,
        ])?,
        #[cfg(not(debug_assertions))]
        rng: rand::new_rng_with_random_seed()?,

        // Cfg_Vars
        gameplay_update_tick_ms,
        smooth_by_extrapolating_velocity,
        clear_color,
        #[cfg(debug_assertions)]
        debug_cvars,
    }))
}

#[cfg(debug_assertions)]
fn create_debug_cvars(cfg: &ecs_engine::cfg::Config) -> Debug_CVars {
    let draw_sprites_bg = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg", cfg);
    let draw_sprites_bg_color = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg_color", cfg);
    let extra_frame_sleep_ms = Cfg_Var::new("engine/debug/extra_frame_sleep_ms", cfg);
    let record_replay = Cfg_Var::new("engine/debug/replay/record", cfg);
    let trace_overlay_refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", cfg);
    let draw_entities = Cfg_Var::new("engine/debug/draw_entities", cfg);
    let draw_entities_velocities = Cfg_Var::new("engine/debug/draw_entities_velocities", cfg);
    let draw_colliders = Cfg_Var::new("engine/debug/collisions/draw_colliders", cfg);
    let draw_collision_quadtree =
        Cfg_Var::new("engine/debug/collisions/draw_collision_quadtree", cfg);
    let draw_debug_grid = Cfg_Var::new("engine/debug/rendering/grid/draw_grid", cfg);
    let debug_grid_square_size = Cfg_Var::new("engine/debug/rendering/grid/square_size", cfg);
    let debug_grid_opacity = Cfg_Var::new("engine/debug/rendering/grid/opacity", cfg);

    Debug_CVars {
        draw_sprites_bg,
        draw_sprites_bg_color,
        extra_frame_sleep_ms,
        record_replay,
        trace_overlay_refresh_rate,
        draw_entities,
        draw_entities_velocities,
        draw_colliders,
        draw_collision_quadtree,
        draw_debug_grid,
        debug_grid_square_size,
        debug_grid_opacity,
    }
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

    let overlay_cfg = Debug_Overlay_Config {
        row_spacing: 20.0,
        font_size: 13,
        pad_x: 5.0,
        pad_y: 5.0,
        background: colors::rgba(25, 25, 25, 210),
    };
    // Entities overlay
    let mut overlay = debug_ui.create_overlay(String_Id::from("entities"), overlay_cfg, font);
    overlay.vert_align = Align::End;
    overlay.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(
        0.0,
        game_state.engine_state.app_config.target_win_size.1 as f32 - 20.,
    );
    // Camera overlay
    let mut overlay = debug_ui.create_overlay(String_Id::from("camera"), overlay_cfg, font);
    overlay.vert_align = Align::End;
    overlay.horiz_align = Align::End;
    overlay.position = Vec2f::new(
        game_state.engine_state.app_config.target_win_size.0 as f32,
        game_state.engine_state.app_config.target_win_size.1 as f32 - 20.,
    );
}
