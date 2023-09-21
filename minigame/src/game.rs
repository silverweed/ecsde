use super::debug;
use super::{Game_Resources, Game_State};
use inle_cfg::Cfg_Var;
use inle_input::core_actions::Core_Action;

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

    Box::new(Game_State {
        env,
        config,
        app_config,
        window,
        loggers,
        time,
        input,
        rng,
        cur_frame: 0,
        frame_alloc,
        should_quit: false,
        default_font: None,
        debug_systems,
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
