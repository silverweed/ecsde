#[macro_use]
extern crate inle_diagnostics;

#[macro_use]
extern crate inle_math;

use std::ffi::c_char;

use inle_input::core_actions::Core_Action;
use inle_app::app_config;

pub struct Game_State {
    should_quit: bool,
    env: inle_core::env::Env_Info,
    config: inle_cfg::config::Config,

    window: inle_gfx::render_window::Render_Window_Handle,
    input: inle_input::input_state::Input_State,

    default_font: inle_resources::gfx::Font_Handle,
}
pub struct Game_Resources<'r> {
    pub gfx: inle_resources::gfx::Gfx_Resources<'r>,
    pub audio: inle_resources::audio::Audio_Resources<'r>,
    pub shader_cache: inle_resources::gfx::Shader_Cache<'r>,
}

#[repr(C)]
pub struct Game_Bundle<'r> {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources<'r>,
}

#[no_mangle]
pub unsafe extern "C" fn game_init<'a>(_args: *const *const c_char, _args_count: usize) -> Game_Bundle<'a> {
    let mut game_state = internal_game_init();
    let mut game_res = create_game_resources();

    let font_name = inle_cfg::Cfg_Var::<String>::new("engine/debug/ui/font", &game_state.config);
    game_state.default_font = game_res.gfx
        .load_font(&inle_resources::gfx::font_path(
            &game_state.env,
            font_name.read(&game_state.config),
        ));

    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

#[no_mangle]
pub unsafe extern "C" fn game_update(game_state: *mut Game_State, game_res: *mut Game_Resources<'_>) -> bool {
    let game_state = &mut *game_state;
    let game_res = &mut *game_res;

    //
    // Input
    //
    process_input(game_state);
    if game_state.should_quit {
        return false;
    }

    //
    // Render
    //
    let win = &mut game_state.window;
    inle_gfx::render_window::clear(win);
    let font = game_res.gfx.get_font(game_state.default_font);
    let txt = inle_gfx::render::create_text(win, "Hello Minigame!", font, 42);
    inle_gfx::render::render_text(win, &txt, inle_common::colors::GREEN, v2!(100., 100.));
    inle_win::window::display(win);

    true 
}

#[no_mangle]
pub unsafe extern "C" fn game_shutdown(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

/*
#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_reload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}
*/

//
// Init
//
fn internal_game_init() -> Box<Game_State> {
    use inle_core::env::Env_Info;
    use inle_cfg::Cfg_Var;

    let mut loggers = unsafe { inle_diagnostics::log::create_loggers() };
    inle_diagnostics::log::add_default_logger(&mut loggers);

    linfo!("Hello!");

    let env = Env_Info::gather().unwrap();
    let config = inle_cfg::Config::new_from_dir(&env.cfg_root);

    let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", &config);
    let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", &config);
    let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", &config);
    let target_win_size = (win_width.read(&config) as u32, win_height.read(&config) as u32);
    let window_create_args = inle_win::window::Create_Window_Args { vsync: true };
    let window = inle_win::window::create_window(&window_create_args, target_win_size, win_title.read(&config));
    let window = inle_gfx::render_window::create_render_window(window);

    /*
    let app_cfg = {
        let cfg = &config;
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", cfg);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", cfg);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", cfg);
        app_config::App_Config {
                title: win_title.read(cfg).clone(),
                target_win_size: (win_width.read(cfg) as u32, win_height.read(cfg) as u32),
        }
    };
    */

    let input = inle_input::input_state::create_input_state(&env);

    Box::new(Game_State { 
        env,
        config,
        window, 
        input,
        should_quit: false,
        default_font: None,
    })
}

fn create_game_resources<'a>() -> Box<Game_Resources<'a>> {
    let gfx = inle_resources::gfx::Gfx_Resources::new();
    let audio = inle_resources::audio::Audio_Resources::new();
    let shader_cache = inle_resources::gfx::Shader_Cache::new();
    Box::new(Game_Resources {
        gfx,
        audio,
        shader_cache,
    })
}

//
// Input
//
fn process_input(game_state: &mut Game_State) {
    inle_input::input_state::update_raw_input(
        &mut game_state.window,
        &mut game_state.input.raw,
    );
    let process_game_actions = true;
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


fn handle_core_actions(window: &mut inle_gfx::render_window::Render_Window_Handle, input: &mut inle_input::input_state::Input_State) -> bool {
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

