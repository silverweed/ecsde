use crate::cmdline;
use crate::gameplay_system;
use crate::states;
use ecs_engine::cfg;
use ecs_engine::cfg::Cfg_Var;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::core::env::Env_Info;
use ecs_engine::core::{app, app_config};
use ecs_engine::gfx::{self as ngfx, window};
use ecs_engine::input;
use ecs_engine::resources;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(debug_assertions)]
#[rustfmt::skip]
use ecs_engine::{
    common::colors,
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
    pub draw_collision_applied_impulses: Cfg_Var<bool>,

    pub draw_entities: Cfg_Var<bool>,
    pub draw_velocities: Cfg_Var<bool>,
    pub draw_entity_prev_frame_ghost: Cfg_Var<bool>,

    pub draw_debug_grid: Cfg_Var<bool>,
    pub debug_grid_square_size: Cfg_Var<f32>,
    pub debug_grid_opacity: Cfg_Var<i32>,

    pub draw_fps_graph: Cfg_Var<bool>,
    pub draw_prev_frame_t_graph: Cfg_Var<bool>,

    pub draw_mouse_rulers: Cfg_Var<bool>,

    pub draw_comp_alloc_colliders: Cfg_Var<bool>,

    pub draw_world_chunks: Cfg_Var<bool>,
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

pub(super) fn internal_game_init<'a>(
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
            &mut game_state.engine_state,
            gameplay_system::Gameplay_System_Config {
                n_entities_to_spawn: parsed_cmdline_args.n_entities_to_spawn.unwrap_or(1),
            },
        )?;

        // @Temporary
        game_state.gameplay_system.load_test_level(
            &mut game_state.engine_state,
            &mut *game_resources,
            &mut game_state.level_batches,
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

    lok!(
        "Initialized sleep with granularity {:?}",
        game_state.sleep_granularity
    );

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

    let mut engine_state = app::create_engine_state(env, config, app_config)?;

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
        let font = Cfg_Var::<String>::new("engine/debug/ui/font", &engine_state.config);
        let cfg = ngdebug::debug_ui::Debug_Ui_System_Config {
            ui_scale,
            target_win_size: engine_state.app_config.target_win_size,
            font: font.read(&engine_state.config).to_string(),
        };
        app::init_engine_debug(&mut engine_state, &mut game_resources.gfx, cfg)?;
        if ecs_engine::debug::console::load_console_hist(
            &mut engine_state.debug_systems.console,
            &engine_state.env,
        )
        .is_ok()
        {
            lok!("Loaded console history");
        }

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
    let draw_entity_prev_frame_ghost =
        Cfg_Var::new("engine/debug/entities/draw_prev_frame_ghost", cfg);
    let draw_colliders = Cfg_Var::new("engine/debug/collisions/draw_colliders", cfg);
    let draw_collision_quadtree = Cfg_Var::new("engine/debug/collisions/draw_quadtree", cfg);
    let draw_collision_applied_impulses =
        Cfg_Var::new("engine/debug/collisions/draw_applied_impulses", cfg);
    let draw_debug_grid = Cfg_Var::new("engine/debug/rendering/grid/draw_grid", cfg);
    let debug_grid_square_size = Cfg_Var::new("engine/debug/rendering/grid/square_size", cfg);
    let debug_grid_opacity = Cfg_Var::new("engine/debug/rendering/grid/opacity", cfg);
    let draw_fps_graph = Cfg_Var::new("engine/debug/graphs/fps", cfg);
    let draw_prev_frame_t_graph = Cfg_Var::new("engine/debug/graphs/prev_frame_t", cfg);
    let draw_mouse_rulers = Cfg_Var::new("engine/debug/window/draw_mouse_rulers", cfg);
    let draw_comp_alloc_colliders = Cfg_Var::new("engine/debug/ecs/comp_alloc/colliders", cfg);
    let draw_world_chunks = Cfg_Var::new("engine/debug/world/draw_chunks", cfg);

    Debug_CVars {
        draw_sprites_bg,
        draw_sprites_bg_color,
        record_replay,
        trace_overlay_refresh_rate,
        draw_entities,
        draw_velocities,
        draw_colliders,
        draw_collision_quadtree,
        draw_collision_applied_impulses,
        draw_debug_grid,
        debug_grid_square_size,
        debug_grid_opacity,
        draw_fps_graph,
        draw_prev_frame_t_graph,
        draw_mouse_rulers,
        draw_comp_alloc_colliders,
        draw_world_chunks,
        draw_entity_prev_frame_ghost,
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

    let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui;
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

    // Physics overlay
    let overlay = debug_ui
        .create_overlay(String_Id::from("physics"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 40. * ui_scale);

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
