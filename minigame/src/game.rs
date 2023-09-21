use super::debug;
use super::{Game_Resources, Game_State};
use std::time::Duration;
use inle_cfg::Cfg_Var;
use inle_input::core_actions::Core_Action;

pub struct CVars {
    pub gameplay_update_tick_ms: Cfg_Var<f32>,
    pub gameplay_max_time_budget_ms: Cfg_Var<f32>,
    pub vsync: Cfg_Var<bool>,
    pub clear_color: Cfg_Var<u32>,
    pub enable_shaders: Cfg_Var<bool>,
    pub enable_shadows: Cfg_Var<bool>,
    pub enable_particles: Cfg_Var<bool>,
    pub ambient_intensity: Cfg_Var<f32>,
    pub ambient_color: Cfg_Var<u32>,
}

#[cfg(debug_assertions)]
pub struct Debug_CVars {
    pub render_debug_visualization: Cfg_Var<String>,
    pub draw_lights: Cfg_Var<bool>,
    pub draw_particle_emitters: Cfg_Var<bool>,
    pub trace_overlay_refresh_rate: Cfg_Var<f32>,
    pub draw_colliders: Cfg_Var<bool>,
    pub draw_entities: Cfg_Var<bool>,
    pub draw_velocities: Cfg_Var<bool>,
    pub draw_debug_grid: Cfg_Var<bool>,
    pub debug_grid_square_size: Cfg_Var<f32>,
    pub debug_grid_opacity: Cfg_Var<i32>,
    pub debug_grid_font_size: Cfg_Var<u32>,
    pub draw_fps_graph: Cfg_Var<bool>,
    pub draw_prev_frame_t_graph: Cfg_Var<bool>,
    pub draw_mouse_rulers: Cfg_Var<bool>,
    pub draw_buf_alloc: Cfg_Var<bool>,
    pub display_log_window: Cfg_Var<bool>,
    pub display_overlays: Cfg_Var<bool>,
    pub update_physics: Cfg_Var<bool>,
    pub print_draw_stats: Cfg_Var<bool>,
}


//
// Init
//
pub fn internal_game_init() -> Box<Game_State> {
    use inle_core::env::Env_Info;

    let mut loggers = unsafe { inle_diagnostics::log::create_loggers() };
    inle_diagnostics::log::add_default_logger(&mut loggers);

    let env = Env_Info::gather().unwrap();
    let mut config = inle_cfg::Config::new_from_dir(&env.cfg_root);
    #[cfg(debug_assertions)]
    {
        inle_app::app::start_config_watch(&env, &mut config)
            .unwrap_or_else(|err| lerr!("Failed to start config watch: {}", err));
    }

    let app_config = {
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", &config);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", &config);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", &config);
        inle_app::app_config::App_Config {
            title: win_title.read(&config).clone(),
            target_win_size: (
                win_width.read(&config) as u32,
                win_height.read(&config) as u32,
            ),
        }
    };

    let mut window = {
        let window_create_args = inle_win::window::Create_Window_Args { vsync: true };
        let window = inle_win::window::create_window(
            &window_create_args,
            app_config.target_win_size,
            &app_config.title,
        );
        inle_gfx::render_window::create_render_window(window)
    };

    inle_gfx::render_window::set_clear_color(&mut window, inle_common::colors::rgb(30, 30, 30));

    let input = inle_input::input_state::create_input_state(&env);

    let seed = inle_core::rand::new_random_seed().unwrap();
    let rng = inle_core::rand::new_rng_with_seed(seed);

    let mut debug_systems = inle_app::debug_systems::Debug_Systems::new(&config);
    //debug_systems.show_overlay = inle_app::debug_systems::Overlay_Shown::Trace;

    let time = inle_core::time::Time::default();
    let frame_alloc =
        inle_alloc::temp::Temp_Allocator::with_capacity(inle_common::units::gigabytes(1));

    let cvars = create_cvars(&config);
    #[cfg(debug_assertions)]
    let debug_cvars = create_debug_cvars(&config);

    Box::new(Game_State {
        env,
        config,
        app_config,
        window,
        loggers,
        time,
        prev_frame_time: Duration::default(),
        input,
        rng,
        cur_frame: 0,
        frame_alloc,
        should_quit: false,
        default_font: None,
        debug_systems,
        cvars,
        #[cfg(debug_assertions)]
        debug_cvars,
        #[cfg(debug_assertions)]
        fps_counter: inle_debug::fps::Fps_Counter::with_update_rate(&Duration::from_secs(1)),
    })
}

pub fn create_game_resources<'a>() -> Box<Game_Resources<'a>> {
    let gfx = inle_resources::gfx::Gfx_Resources::new();
    let audio = inle_resources::audio::Audio_Resources::new();
    let shader_cache = inle_resources::gfx::Shader_Cache::new();
    Box::new(Game_Resources {
        gfx,
        audio,
        shader_cache,
    })
}

// Used to initialize game state stuff that needs resources
pub fn game_post_init(game_state: &mut Game_State, game_res: &mut Game_Resources<'_>) {
    let font_name = inle_cfg::Cfg_Var::<String>::new("engine/debug/ui/font", &game_state.config);
    game_state.default_font = game_res.gfx.load_font(&inle_resources::gfx::font_path(
        &game_state.env,
        font_name.read(&game_state.config),
    ));

    #[cfg(debug_assertions)]
    {
        debug::init_debug(game_state, game_res);
    }
}

pub fn start_frame(game_state: &mut Game_State) {
    inle_gfx::render_window::start_new_frame(&mut game_state.window);

    #[cfg(debug_assertions)]
    {
        debug::start_debug_frame(
            &mut game_state.debug_systems,
            &game_state.time,
            game_state.cur_frame,
        );
    }

    game_state.time.update();
}

pub fn end_frame(game_state: &mut Game_State) {
    #[cfg(debug_assertions)]
    {
        let refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", &game_state.config);
        debug::update_traces(
            refresh_rate,
            &mut game_state.debug_systems,
            &game_state.config,
            &game_state.time,
            game_state.cur_frame,
            &mut game_state.frame_alloc,
        );
    }

    unsafe {
        game_state.frame_alloc.dealloc_all();
    }
}

//
// Input
//
pub fn process_input(game_state: &mut Game_State) {
    let process_game_actions;
    #[cfg(debug_assertions)]
    {
        process_game_actions = game_state.debug_systems.console.lock().unwrap().status
            != inle_debug::console::Console_Status::Open;
    }
    #[cfg(not(debug_assertions))]
    {
        process_game_actions = true;
    }

    inle_input::input_state::update_raw_input(&mut game_state.window, &mut game_state.input.raw);
    inle_input::input_state::process_raw_input(
        &game_state.input.raw,
        &game_state.input.bindings,
        &mut game_state.input.processed,
        process_game_actions,
    );

    if handle_core_actions(&mut game_state.window, &mut game_state.input) {
        game_state.should_quit = true;
    }
}

fn handle_core_actions(
    window: &mut inle_gfx::render_window::Render_Window_Handle,
    input: &mut inle_input::input_state::Input_State,
) -> bool {
    for action in &input.processed.core_actions {
        match action {
            Core_Action::Quit => return true,
            Core_Action::Resize(new_width, new_height) => {
                inle_gfx::render_window::resize_keep_ratio(window, *new_width, *new_height);
            }
            Core_Action::Focus_Lost => {
                input.raw.kb_state.modifiers_pressed = 0;
                inle_input::mouse::reset_mouse_state(&mut input.raw.mouse_state);
            }
            _ => unimplemented!(),
        }
    }
    false
}

//
// Render
//
pub fn render(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    trace!("render");

    let win = &mut game_state.window;
    inle_gfx::render_window::clear(win);

    // TEMP
    let font = game_res.gfx.get_font(game_state.default_font);
    let txt = inle_gfx::render::create_text(win, "Hello Minigame!", font, 42);
    inle_gfx::render::render_text(win, &txt, inle_common::colors::GREEN, v2!(100., 100.));
    //

    #[cfg(debug_assertions)]
    {
        debug::render_debug(
            &mut game_state.debug_systems,
            win,
            &game_state.input,
            &game_state.config,
            &mut game_state.frame_alloc,
            &mut game_state.time,
            &mut game_res.gfx,
        );
    }

    inle_win::window::display(win);
}

fn create_cvars(cfg: &inle_cfg::Config) -> CVars {
    let gameplay_update_tick_ms = Cfg_Var::new("engine/gameplay/update_tick_ms", cfg);
    let gameplay_max_time_budget_ms = Cfg_Var::new("engine/gameplay/max_time_budget_ms", cfg);
    let clear_color = Cfg_Var::new("engine/rendering/clear_color", cfg);
    let vsync = Cfg_Var::new("engine/window/vsync", cfg);
    let enable_shaders = Cfg_Var::new("engine/rendering/enable_shaders", cfg);
    let enable_shadows = Cfg_Var::new("engine/rendering/enable_shadows", cfg);
    let enable_particles = Cfg_Var::new("engine/rendering/enable_particles", cfg);
    let ambient_intensity = Cfg_Var::new("game/world/lighting/ambient_intensity", cfg);
    let ambient_color = Cfg_Var::new("game/world/lighting/ambient_color", cfg);

    CVars {
        gameplay_update_tick_ms,
        gameplay_max_time_budget_ms,
        vsync,
        clear_color,
        enable_shaders,
        enable_shadows,
        enable_particles,
        ambient_intensity,
        ambient_color,
    }
}

#[cfg(debug_assertions)]
fn create_debug_cvars(cfg: &inle_cfg::Config) -> Debug_CVars {
    let render_debug_visualization = Cfg_Var::new("debug/rendering/debug_visualization", cfg);
    let draw_lights = Cfg_Var::new("debug/rendering/draw_lights", cfg);
    let draw_particle_emitters = Cfg_Var::new("debug/rendering/draw_particle_emitters", cfg);
    let trace_overlay_refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", cfg);
    let draw_entities = Cfg_Var::new("debug/entities/draw_entities", cfg);
    let draw_velocities = Cfg_Var::new("debug/entities/draw_velocities", cfg);
    let draw_colliders = Cfg_Var::new("debug/collisions/draw_colliders", cfg);
    let draw_debug_grid = Cfg_Var::new("debug/rendering/grid/draw_grid", cfg);
    let debug_grid_square_size = Cfg_Var::new("debug/rendering/grid/square_size", cfg);
    let debug_grid_opacity = Cfg_Var::new("debug/rendering/grid/opacity", cfg);
    let debug_grid_font_size = Cfg_Var::new("debug/rendering/grid/font_size", cfg);
    let draw_fps_graph = Cfg_Var::new("debug/graphs/fps", cfg);
    let draw_prev_frame_t_graph = Cfg_Var::new("debug/graphs/prev_frame_t", cfg);
    let draw_mouse_rulers = Cfg_Var::new("debug/window/draw_mouse_rulers", cfg);
    let draw_buf_alloc = Cfg_Var::new("debug/rendering/draw_buf_alloc", cfg);
    let display_log_window = Cfg_Var::new("engine/debug/log_window/display", cfg);
    let display_overlays = Cfg_Var::new("engine/debug/overlay/display", cfg);
    let update_physics = Cfg_Var::new("engine/debug/physics/update", cfg);
    let print_draw_stats = Cfg_Var::new("debug/rendering/print_draw_stats", cfg);

    Debug_CVars {
        render_debug_visualization,
        draw_lights,
        draw_particle_emitters,
        trace_overlay_refresh_rate,
        draw_colliders,
        draw_entities,
        draw_velocities,
        draw_debug_grid,
        debug_grid_square_size,
        debug_grid_opacity,
        debug_grid_font_size,
        draw_fps_graph,
        draw_prev_frame_t_graph,
        draw_mouse_rulers,
        draw_buf_alloc,
        display_log_window,
        display_overlays,
        update_physics,
        print_draw_stats,
    }
}
