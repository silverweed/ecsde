use std::ffi::c_char;

pub struct Game_State {}
pub struct Game_Resources {}

#[repr(C)]
pub struct Game_Bundle {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources,
}

#[no_mangle]
pub unsafe extern "C" fn game_init(_args: *const *const c_char, _args_count: usize) -> Game_Bundle {
    println!("HELLO!");
    let game_state = Box::new(Game_State{});
    let game_res = Box::new(Game_Resources{});
    Game_Bundle {
        game_state: Box::into_raw(game_state),
        game_resources: Box::into_raw(game_res),
    }
}

#[no_mangle]
pub unsafe extern "C" fn game_update(_game_state: *mut Game_State, _game_res: *mut Game_Resources) -> bool { false }

#[no_mangle]
pub unsafe extern "C" fn game_shutdown(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_unload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn game_reload(_game_state: *mut Game_State, _game_res: *mut Game_Resources) {}
