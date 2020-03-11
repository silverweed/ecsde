#![allow(clippy::new_without_default)]
#![allow(non_camel_case_types)]
#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
extern crate ecs_engine;
#[cfg(test)]
extern crate float_cmp;

mod cmdline;
mod controllable_system;
mod game_loop;
mod gameplay_system;
mod input_utils;
mod load;
mod movement_system;
mod states;

#[cfg(debug_assertions)]
mod debug;

use ecs_engine::cfg;
use ecs_engine::cfg::Cfg_Var;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::rand;
use ecs_engine::core::{app, app_config};
use ecs_engine::gfx::{self as ngfx, window};
use ecs_engine::input;
use ecs_engine::resources;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::Duration;

#[cfg(debug_assertions)]
#[rustfmt::skip]
use ecs_engine::{
    common::colors,
    common::stringid::String_Id,
    debug as ngdebug
};

#[repr(C)]
pub struct Game_State<'a> {
    pub window: window::Window_Handle,
    pub engine_state: app::Engine_State<'a>,

    pub gameplay_system: gameplay_system::Gameplay_System,

    pub state_mgr: states::state_manager::State_Manager,
    #[cfg(debug_assertions)]
    pub fps_debug: ngdebug::fps::Fps_Console_Printer,

    pub execution_time: Duration,
    pub input_provider: Box<dyn input::provider::Input_Provider>,
    pub is_replaying: bool,

    pub sleep_granularity: Option<Duration>,

    pub level_batches: HashMap<String_Id, ngfx::render::batcher::Batches>,

    //// Cfg vars
    pub cvars: CVars,

    #[cfg(debug_assertions)]
    pub debug_cvars: Debug_CVars,

    pub rng: rand::Default_Rng,
}

pub struct CVars {
    pub gameplay_update_tick_ms: Cfg_Var<f32>,
    pub vsync: Cfg_Var<bool>,
    pub clear_color: Cfg_Var<u32>,
}

#[cfg(debug_assertions)]
pub struct Debug_CVars {
    pub draw_sprites_bg: Cfg_Var<bool>,
    pub draw_sprites_bg_color: Cfg_Var<u32>,

    pub record_replay: Cfg_Var<bool>,

    pub trace_overlay_refresh_rate: Cfg_Var<f32>,

    pub draw_colliders: Cfg_Var<bool>,
    pub draw_collision_quadtree: Cfg_Var<bool>,

    pub draw_entities: Cfg_Var<bool>,
    pub draw_velocities: Cfg_Var<bool>,

    pub draw_debug_grid: Cfg_Var<bool>,
    pub debug_grid_square_size: Cfg_Var<f32>,
    pub debug_grid_opacity: Cfg_Var<i32>,

    pub draw_fps_graph: Cfg_Var<bool>,

    pub draw_mouse_rulers: Cfg_Var<bool>,
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

/// Given a c_char pointer, returns a String allocated from the raw string it points to,
/// or an empty string if the conversion fails.
fn new_string_from_c_char_ptr(c_char_ptr: *const c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(c_char_ptr) };
    let str_slice = cstr.to_str().unwrap_or_else(|_| {
        lerr!("Failed to convert argument {:?} to a valid String.", cstr);
        ""
    });
    String::from(str_slice)
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
    raw_args: *const *const c_char,
    args_count: usize,
) -> Game_Bundle<'a> {
    linfo!("Initializing game...");

    // Copy all arguments into rust strings
    let mut args: Vec<String> = Vec::with_capacity(args_count);
    for i in 0..args_count {
        let arg = raw_args.add(i);
        assert!(!(*arg).is_null(), "{}-th cmdline argument is null!", i);
        args.push(new_string_from_c_char_ptr(*arg));
    }

    if let Ok((game_state, game_resources)) = internal_game_init(&args) {
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
        fatal!("game_update: game state and/or resources are null!");
    }

    {
        let game_state = &mut *game_state;
        if game_state.engine_state.should_close {
            return false;
        }

        #[cfg(debug_assertions)]
        {
            ecs_engine::prelude::DEBUG_TRACER
                .lock()
                .unwrap()
                .start_frame();

            let log = &mut game_state.engine_state.debug_systems.log;

            if !game_state.engine_state.time.paused {
                if game_state.engine_state.time.was_paused {
                    // Just resumed
                    log.reset_from_frame(game_state.engine_state.cur_frame);
                }
                log.start_frame();
            }
        }

        let game_resources = &mut *game_resources;
        if game_loop::tick_game(game_state, game_resources).is_err() {
            return false;
        }
    }

    #[cfg(debug_assertions)]
    {
        let game_state = &mut *game_state;
        app::update_traces(
            &mut game_state.engine_state,
            game_state.debug_cvars.trace_overlay_refresh_rate,
        );
    }

    !(*game_state).engine_state.should_close
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
        fatal!("game_shutdown: game state and/or resources are null!");
    }

    std::ptr::drop_in_place(game_state);
    dealloc(game_state as *mut u8, Layout::new::<Game_State>());

    std::ptr::drop_in_place(game_resources);
    dealloc(game_resources as *mut u8, Layout::new::<Game_Resources>());

    lok!("Game was shut down.");
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
            fatal!("game_reload: game state is null!");
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
    args: &[String],
) -> Result<(Box<Game_State<'a>>, Box<Game_Resources<'a>>), Box<dyn std::error::Error>> {
    let mut game_resources = create_game_resources()?;
    let (mut game_state, parsed_cmdline_args) = create_game_state(&mut game_resources, args)?;

    {
        let env = &game_state.engine_state.env;
        let gres = &mut game_resources.gfx;
        let cfg = &game_state.engine_state.config;

        game_state.gameplay_system.init(
            gres,
            env,
            &mut game_state.rng,
            cfg,
            gameplay_system::Gameplay_System_Config {
                n_entities_to_spawn: parsed_cmdline_args.n_entities_to_spawn.unwrap_or(2),
            },
        )?;

        // @Temporary
        game_state.gameplay_system.load_test_level(
            &mut game_state.engine_state,
            &mut *game_resources,
            &mut game_state.level_batches,
            &mut game_state.rng,
        );

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

    game_state.sleep_granularity = ecs_engine::core::sleep::init_sleep()
        .ok()
        .map(|g| g.max(Duration::from_micros(1)));

    // This happens after all the initialization
    game_state.engine_state.time.start();

    Ok((game_state, game_resources))
}

fn create_game_state<'a>(
    game_resources: &mut Game_Resources<'_>,
    cmdline_args: &[String],
) -> Result<(Box<Game_State<'a>>, cmdline::Cmdline_Args), Box<dyn std::error::Error>> {
    // Load Config first, as it's needed to setup everything that follows.
    let env = Env_Info::gather().unwrap();
    let config = cfg::Config::new_from_dir(&env.cfg_root);

    // Load initial App_Config (some values may be overwritten by cmdline args)
    let mut_in_debug!(app_config) = {
        let cfg = &config;
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", cfg);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", cfg);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", cfg);
        app_config::App_Config {
            title: win_title.read(cfg).clone(),
            target_win_size: (win_width.read(cfg) as u32, win_height.read(cfg) as u32),
            #[cfg(debug_assertions)]
            in_replay_file: None,
        }
    };

    let mut_in_debug!(parsed_cmdline_args) = cmdline::parse_cmdline_args(cmdline_args.iter());
    #[cfg(debug_assertions)]
    {
        if let Some(in_replay_file) = parsed_cmdline_args.in_replay_file.take() {
            app_config.in_replay_file = Some(in_replay_file);
        }
    }

    let mut engine_state = app::create_engine_state(env, config, app_config);

    let appcfg = &engine_state.app_config;
    let cfg = &engine_state.config;
    let cvars = create_cvars(cfg);

    let window_create_args = ngfx::window::Create_Render_Window_Args {
        vsync: cvars.vsync.read(cfg),
    };
    let window = ngfx::window::create_render_window(
        &window_create_args,
        appcfg.target_win_size,
        &appcfg.title,
    );

    #[cfg(debug_assertions)]
    {
        if let Some(in_replay_file) = &appcfg.in_replay_file {
            linfo!("Loading replay file {:?}", in_replay_file);
            engine_state.replay_data = app::try_create_replay_data(in_replay_file);
        }
    }

    linfo!("Working dir = {:?}", engine_state.env.working_dir);
    linfo!("Exe = {:?}", engine_state.env.full_exe_path);

    app::init_engine_systems(&mut engine_state)?;

    #[cfg(debug_assertions)]
    {
        app::start_config_watch(&engine_state.env, &mut engine_state.config)?;

        let ui_scale = Cfg_Var::<f32>::new("engine/debug/ui/ui_scale", &engine_state.config)
            .read(&engine_state.config);
        let cfg = ngdebug::debug_ui_system::Debug_Ui_System_Config {
            ui_scale,
            target_win_size: engine_state.app_config.target_win_size,
        };
        app::init_engine_debug(&mut engine_state, &mut game_resources.gfx, cfg)?;

        app::start_recording(&mut engine_state)?;
    }

    #[cfg(debug_assertions)]
    let cfg = &engine_state.config;

    #[cfg(debug_assertions)]
    let input_provider = app::create_input_provider(&mut engine_state.replay_data, cfg);
    #[cfg(not(debug_assertions))]
    let input_provider = app::create_input_provider();

    let is_replaying = !input_provider.is_realtime_player_input();

    #[cfg(debug_assertions)]
    let debug_cvars = create_debug_cvars(cfg);

    Ok((
        Box::new(Game_State {
            window,
            engine_state,

            #[cfg(debug_assertions)]
            fps_debug: ngdebug::fps::Fps_Console_Printer::new(&Duration::from_secs(2), "game"),

            sleep_granularity: None,

            level_batches: HashMap::new(),

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
            cvars,
            #[cfg(debug_assertions)]
            debug_cvars,
        }),
        parsed_cmdline_args,
    ))
}

fn create_cvars(cfg: &ecs_engine::cfg::Config) -> CVars {
    let gameplay_update_tick_ms = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg);
    let clear_color = Cfg_Var::new("engine/rendering/clear_color", cfg);
    let vsync = Cfg_Var::new("engine/window/vsync", cfg);

    CVars {
        gameplay_update_tick_ms,
        clear_color,
        vsync,
    }
}

#[cfg(debug_assertions)]
fn create_debug_cvars(cfg: &ecs_engine::cfg::Config) -> Debug_CVars {
    let draw_sprites_bg = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg", cfg);
    let draw_sprites_bg_color = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg_color", cfg);
    let record_replay = Cfg_Var::new("engine/debug/replay/record", cfg);
    let trace_overlay_refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", cfg);
    let draw_entities = Cfg_Var::new("engine/debug/entities/draw_entities", cfg);
    let draw_velocities = Cfg_Var::new("engine/debug/entities/draw_velocities", cfg);
    let draw_colliders = Cfg_Var::new("engine/debug/collisions/draw_colliders", cfg);
    let draw_collision_quadtree =
        Cfg_Var::new("engine/debug/collisions/draw_collision_quadtree", cfg);
    let draw_debug_grid = Cfg_Var::new("engine/debug/rendering/grid/draw_grid", cfg);
    let debug_grid_square_size = Cfg_Var::new("engine/debug/rendering/grid/square_size", cfg);
    let debug_grid_opacity = Cfg_Var::new("engine/debug/rendering/grid/opacity", cfg);
    let draw_fps_graph = Cfg_Var::new("engine/debug/graphs/fps", cfg);
    let draw_mouse_rulers = Cfg_Var::new("engine/debug/window/draw_mouse_rulers", cfg);

    Debug_CVars {
        draw_sprites_bg,
        draw_sprites_bg_color,
        record_replay,
        trace_overlay_refresh_rate,
        draw_entities,
        draw_velocities,
        draw_colliders,
        draw_collision_quadtree,
        draw_debug_grid,
        debug_grid_square_size,
        debug_grid_opacity,
        draw_fps_graph,
        draw_mouse_rulers,
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
    let base_state = Box::new(states::persistent::game_base_state::Game_Base_State::new());
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
    use ecs_engine::common::vector::Vec2f;
    use ecs_engine::debug::overlay::Debug_Overlay_Config;
    use ecs_engine::gfx::align::Align;

    const FONT: &str = "Hack-Regular.ttf";

    let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui_system;
    let font = game_resources.gfx.load_font(&resources::gfx::font_path(
        &game_state.engine_state.env,
        FONT,
    ));
    let ui_scale = debug_ui.cfg.ui_scale;
    let (win_w, win_h) = game_state.engine_state.app_config.target_win_size;

    {
        // Frame scroller
        let scroller = &mut debug_ui.frame_scroller;
        let fps = (1000.
            / game_state
                .cvars
                .gameplay_update_tick_ms
                .read(&game_state.engine_state.config)
            + 0.5) as u64;
        let log_len = game_state.engine_state.debug_systems.log.max_hist_len;
        scroller.n_frames = fps as _;
        scroller.n_seconds = (log_len / fps as u32) as _;
    }

    let overlay_cfg = Debug_Overlay_Config {
        row_spacing: 20.0 * ui_scale,
        font_size: (13.0 * ui_scale) as u16,
        pad_x: 5.0 * ui_scale,
        pad_y: 5.0 * ui_scale,
        background: colors::rgba(25, 25, 25, 210),
        font,
        ..Default::default()
    };
    // Entities overlay
    let overlay = debug_ui
        .create_overlay(String_Id::from("entities"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 22. * ui_scale);
    // Camera overlay
    let overlay = debug_ui
        .create_overlay(String_Id::from("camera"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::End;
    overlay.position = Vec2f::new(win_w as f32, win_h as f32 - 40. * ui_scale);

    // Console hints
    game_state.engine_state.debug_systems.console.add_hints(
        "",
        crate::debug::console_executor::ALL_CMD_STRINGS
            .iter()
            .map(|s| String::from(*s)),
    );
    game_state.engine_state.debug_systems.console.add_hints(
        "var",
        game_state
            .engine_state
            .config
            .get_all_pairs()
            .map(|(k, _)| k),
    );
    game_state.engine_state.debug_systems.console.add_hints(
        "toggle",
        game_state
            .engine_state
            .config
            .get_all_pairs()
            .filter_map(|(k, v)| {
                if let cfg::Cfg_Value::Bool(_) = v {
                    Some(k)
                } else {
                    None
                }
            }),
    );
}
