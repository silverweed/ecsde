use libloading as ll;
use std::os::raw::c_char;

// Note: this is an opaque type
#[repr(C)]
pub struct Game_State {
    _private: [u8; 0],
}

// Note: this is an opaque type
#[repr(C)]
pub struct Game_Resources {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Game_Bundle {
    pub game_state: *mut Game_State,
    pub game_resources: *mut Game_Resources,
}

pub struct Game_Api<'lib> {
    pub init: ll::Symbol<
        'lib,
        unsafe extern "C" fn(args: *const *const c_char, args_count: usize) -> Game_Bundle,
    >,
    pub update:
        ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State, *mut Game_Resources) -> bool>,
    pub shutdown: ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State, *mut Game_Resources)>,
    pub unload: Option<ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State, *mut Game_Resources)>>,
    pub reload: Option<ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State, *mut Game_Resources)>>,
}
