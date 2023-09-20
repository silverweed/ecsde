#[macro_use]
extern crate inle_diagnostics;

use std::ffi::c_char;

use inle_app::app_config;

pub struct Game_State {
}
pub struct Game_Resources {}

#[repr(C)]
pub struct Game_Bundle {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources,
}

#[no_mangle]
pub unsafe extern "C" fn game_init(_args: *const *const c_char, _args_count: usize) -> Game_Bundle {
    let game_state = internal_game_init();
    let game_res = Box::new(Game_Resources{});
    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

#[no_mangle]
pub unsafe extern "C" fn game_update(game_state: *mut Game_State, _game_res: *mut Game_Resources) -> bool {
    let game_state = &*game_state;

    false 
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

fn internal_game_init() -> Box<Game_State> {
    use inle_core::env::Env_Info;
    use inle_cfg::Cfg_Var;

    let mut loggers = unsafe { inle_diagnostics::log::create_loggers() };
    inle_diagnostics::log::add_default_logger(&mut loggers);

    linfo!("Hello!");

    let env = Env_Info::gather().unwrap();
    let config = inle_cfg::Config::new_from_dir(&env.cfg_root);

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

    Box::new(Game_State {})
}
