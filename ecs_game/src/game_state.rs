use crate::cmdline;
use crate::collisions;
use crate::gameplay_system;
use crate::states;
use inle_app::{app, app_config};
use inle_cfg::{self, Cfg_Var};
use inle_common::stringid::String_Id;
use inle_core::env::Env_Info;
use inle_gfx::{self, render_window::Render_Window_Handle};
use std::collections::HashMap;
use std::time::Duration;

#[cfg(debug_assertions)]
#[rustfmt::skip]
use {
    inle_common::colors,
    inle_debug 
};

pub type Level_Batches = HashMap<String_Id, inle_gfx::render::batcher::Batches>;

#[repr(C)]
pub struct Game_State<'a> {
    pub window: Render_Window_Handle,
    pub engine_state: app::Engine_State<'a>,

    pub gameplay_system: gameplay_system::Gameplay_System,
    pub state_mgr: states::state_manager::State_Manager,
    pub execution_time: Duration,

    pub sleep_granularity: Option<Duration>,

    pub level_batches: Level_Batches,

    pub cvars: CVars,

    #[cfg(debug_assertions)]
    pub debug_cvars: Debug_CVars,

    #[cfg(debug_assertions)]
    pub fps_debug: inle_debug::fps::Fps_Counter,
}

pub struct CVars {
    pub gameplay_update_tick_ms: Cfg_Var<f32>,
    pub vsync: Cfg_Var<bool>,
    pub clear_color: Cfg_Var<u32>,
    pub enable_shaders: Cfg_Var<bool>,
    pub enable_shadows: Cfg_Var<bool>,
    pub enable_particles: Cfg_Var<bool>,
}

#[cfg(debug_assertions)]
pub struct Debug_CVars {
    pub draw_sprites_bg: Cfg_Var<bool>,      // @Cleanup: unused
    pub draw_sprites_bg_color: Cfg_Var<u32>, // @Cleanup: unused
    pub draw_lights: Cfg_Var<bool>,

    pub record_replay: Cfg_Var<bool>,

    pub trace_overlay_refresh_rate: Cfg_Var<f32>,

    pub draw_colliders: Cfg_Var<bool>,

    pub draw_entities: Cfg_Var<bool>,
    pub draw_velocities: Cfg_Var<bool>,
    pub draw_entity_prev_frame_ghost: Cfg_Var<bool>,
    pub draw_component_lists: Cfg_Var<bool>,

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
    pub gfx: inle_resources::gfx::Gfx_Resources<'a>,
    pub audio: inle_resources::audio::Audio_Resources<'a>,
}

#[repr(C)]
pub struct Game_Bundle<'a> {
    pub game_state: *mut Game_State<'a>,
    pub game_resources: *mut Game_Resources<'a>,
}

pub(super) fn internal_game_init<'a>(
    args: &[String],
) -> Result<(Box<Game_State<'a>>, Box<Game_Resources<'a>>), Box<dyn std::error::Error>> {
    let mut game_resources = create_game_resources()
        .unwrap_or_else(|err| fatal!("create_game_resources() failed with err {}", err));
    let (mut game_state, parsed_cmdline_args) = create_game_state(&mut game_resources, args)
        .unwrap_or_else(|err| fatal!("create_game_state() failed with err {}", err));

    {
        let gres = &mut game_resources.gfx;
        game_state.gameplay_system.init(
            gres,
            &mut game_state.engine_state,
            gameplay_system::Gameplay_System_Config {
                n_entities_to_spawn: parsed_cmdline_args.n_entities_to_spawn.unwrap_or(1),
            },
        )?;

        collisions::init_collision_layers(
            &mut game_state
                .engine_state
                .systems
                .physics_settings
                .collision_matrix,
        );

        init_states(
            &mut game_state.state_mgr,
            &mut game_state.engine_state,
            &mut game_state.gameplay_system,
            &mut game_state.window,
            &mut *game_resources,
            &mut game_state.level_batches,
            &parsed_cmdline_args,
        );

        #[cfg(debug_assertions)]
        {
            init_game_debug(&mut game_state, &mut game_resources);
        }
    }

    game_state.sleep_granularity = inle_core::sleep::init_sleep()
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
    let config = inle_cfg::Config::new_from_dir(&env.cfg_root);

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

    let window_create_args = inle_win::window::Create_Window_Args {
        vsync: cvars.vsync.read(cfg),
    };
    let window =
        inle_win::window::create_window(&window_create_args, appcfg.target_win_size, &appcfg.title);
    let window = inle_gfx::render_window::create_render_window(window);

    #[cfg(debug_assertions)]
    {
        if let Some(in_replay_file) = &appcfg.in_replay_file {
            linfo!("Loading replay file {:?}", in_replay_file);
            if let Some(replay_data) = app::try_create_replay_data(in_replay_file) {
                app::set_replay_data(&mut engine_state, replay_data);
            }
        }
    }

    linfo!("Working dir = {:?}", engine_state.env.working_dir);
    linfo!("Exe = {:?}", engine_state.env.full_exe_path);

    app::init_engine_systems(&mut engine_state, &mut game_resources.gfx)?;

    #[cfg(debug_assertions)]
    {
        app::start_config_watch(&engine_state.env, &mut engine_state.config)?;

        let ui_scale = Cfg_Var::<f32>::new("engine/debug/ui/ui_scale", &engine_state.config)
            .read(&engine_state.config);
        let font = Cfg_Var::<String>::new("engine/debug/ui/font", &engine_state.config);
        let cfg = inle_debug::debug_ui::Debug_Ui_System_Config {
            ui_scale,
            target_win_size: engine_state.app_config.target_win_size,
            font: font.read(&engine_state.config).to_string(),
        };

        app::init_engine_debug(&mut engine_state, &mut game_resources.gfx, cfg)?;

        if inle_debug::console::load_console_hist(
            &mut engine_state.debug_systems.console,
            &engine_state.env,
        )
        .is_ok()
        {
            lok!("Loaded console history");
        }
    }

    #[cfg(debug_assertions)]
    let cfg = &engine_state.config;

    #[cfg(debug_assertions)]
    let debug_cvars = create_debug_cvars(cfg);

    #[cfg(debug_assertions)]
    {
        let record_replay_data = debug_cvars.record_replay.read(&engine_state.config);
        if record_replay_data && engine_state.app_config.in_replay_file.is_none() {
            app::start_recording(&mut engine_state).unwrap_or_else(|err| {
                lerr!("Failed to start recording input: {}", err);
            });
        }
    }

    Ok((
        Box::new(Game_State {
            window,
            engine_state,
            sleep_granularity: None,
            level_batches: HashMap::new(),
            execution_time: Duration::default(),
            gameplay_system: gameplay_system::Gameplay_System::new(),
            state_mgr: states::state_manager::State_Manager::new(),
            cvars,

            #[cfg(debug_assertions)]
            debug_cvars,
            #[cfg(debug_assertions)]
            fps_debug: inle_debug::fps::Fps_Counter::with_update_rate(&Duration::from_secs(2)),
        }),
        parsed_cmdline_args,
    ))
}

fn create_cvars(cfg: &inle_cfg::Config) -> CVars {
    let gameplay_update_tick_ms = Cfg_Var::new("engine/gameplay/gameplay_update_tick_ms", cfg);
    let clear_color = Cfg_Var::new("engine/rendering/clear_color", cfg);
    let vsync = Cfg_Var::new("engine/window/vsync", cfg);
    let enable_shaders = Cfg_Var::new("engine/rendering/enable_shaders", cfg);
    let enable_shadows = Cfg_Var::new("engine/rendering/enable_shadows", cfg);
    let enable_particles = Cfg_Var::new("engine/rendering/enable_particles", cfg);

    CVars {
        gameplay_update_tick_ms,
        clear_color,
        vsync,
        enable_shaders,
        enable_shadows,
        enable_particles,
    }
}

#[cfg(debug_assertions)]
fn create_debug_cvars(cfg: &inle_cfg::Config) -> Debug_CVars {
    let draw_sprites_bg = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg", cfg);
    let draw_sprites_bg_color = Cfg_Var::new("engine/debug/rendering/draw_sprites_bg_color", cfg);
    let draw_lights = Cfg_Var::new("engine/debug/rendering/draw_lights", cfg);
    let record_replay = Cfg_Var::new("engine/debug/replay/record", cfg);
    let trace_overlay_refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", cfg);
    let draw_entities = Cfg_Var::new("engine/debug/entities/draw_entities", cfg);
    let draw_velocities = Cfg_Var::new("engine/debug/entities/draw_velocities", cfg);
    let draw_component_lists = Cfg_Var::new("engine/debug/entities/draw_component_lists", cfg);
    let draw_entity_prev_frame_ghost =
        Cfg_Var::new("engine/debug/entities/draw_prev_frame_ghost", cfg);
    let draw_colliders = Cfg_Var::new("engine/debug/collisions/draw_colliders", cfg);
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
        draw_lights,
        record_replay,
        trace_overlay_refresh_rate,
        draw_entities,
        draw_velocities,
        draw_colliders,
        draw_component_lists,
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
    let gfx_resources = inle_resources::gfx::Gfx_Resources::new();
    let audio_resources = inle_resources::audio::Audio_Resources::new();
    Ok(Box::new(Game_Resources {
        gfx: gfx_resources,
        audio: audio_resources,
    }))
}

fn init_states(
    state_mgr: &mut states::state_manager::State_Manager,
    engine_state: &mut app::Engine_State,
    gs: &mut gameplay_system::Gameplay_System,
    window: &mut Render_Window_Handle,
    game_resources: &mut Game_Resources,
    level_batches: &mut Level_Batches,
    cmdline_args: &cmdline::Cmdline_Args,
) {
    let mut args = states::state::Game_State_Args {
        engine_state,
        gameplay_system: gs,
        window,
        game_resources,
        level_batches,
    };
    let base_state = Box::new(states::persistent::game_base_state::Game_Base_State::new());
    state_mgr.add_persistent_state(base_state, &mut args);
    #[cfg(debug_assertions)]
    {
        let debug_base_state = Box::new(
            states::persistent::debug_base_state::Debug_Base_State::new(&engine_state.config),
        );
        let mut args = states::state::Game_State_Args {
            engine_state,
            gameplay_system: gs,
            window,
            game_resources,
            level_batches,
        };
        state_mgr.add_persistent_state(debug_base_state, &mut args);
    }

    let mut args = states::state::Game_State_Args {
        engine_state,
        gameplay_system: gs,
        window,
        game_resources,
        level_batches,
    };
    if cmdline_args.start_from_menu {
        let menu_state = Box::new(states::main_menu_state::Main_Menu_State::default());
        state_mgr.push_state(menu_state, &mut args);
    } else {
        let in_game_state = Box::new(states::in_game_state::In_Game_State::default());
        state_mgr.push_state(in_game_state, &mut args);
    }
}

#[cfg(debug_assertions)]
fn init_game_debug(game_state: &mut Game_State, game_resources: &mut Game_Resources) {
    use inle_common::vis_align::Align;
    use inle_debug::overlay::Debug_Overlay_Config;
    use inle_math::vector::Vec2f;

    const FONT: &str = "Hack-Regular.ttf";

    let debug_ui = &mut game_state.engine_state.debug_systems.debug_ui;
    let font = game_resources
        .gfx
        .load_font(&inle_resources::gfx::font_path(
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
        .create_overlay(sid!("entities"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 22. * ui_scale);

    // Camera overlay
    let overlay = debug_ui
        .create_overlay(sid!("camera"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::End;
    overlay.position = Vec2f::new(win_w as f32, win_h as f32 - 40. * ui_scale);

    // Physics overlay
    let overlay = debug_ui
        .create_overlay(sid!("physics"), overlay_cfg)
        .unwrap();
    overlay.config.vert_align = Align::End;
    overlay.config.horiz_align = Align::Begin;
    overlay.position = Vec2f::new(0.0, win_h as f32 - 40. * ui_scale);

    // Console hints
    let console = &mut game_state.engine_state.debug_systems.console;
    console.add_hints(
        "",
        crate::debug::console_executor::ALL_CMD_STRINGS
            .iter()
            .map(|s| String::from(*s)),
    );
    console.add_hints(
        "var",
        game_state
            .engine_state
            .config
            .get_all_pairs()
            .map(|(k, _)| k),
    );
    console.add_hints(
        "toggle",
        game_state
            .engine_state
            .config
            .get_all_pairs()
            .filter_map(|(k, v)| {
                if let inle_cfg::Cfg_Value::Bool(_) = v {
                    Some(k)
                } else {
                    None
                }
            }),
    );
}
