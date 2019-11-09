extern crate ecs_engine;

// An opaque type
#[repr(C)]
pub struct Game_State {
    _private: [u8; 0],
}

extern "C" {
    pub fn game_init() -> *mut Game_State;
    pub fn game_update(state: *mut Game_State) -> bool;
    pub fn game_shutdown(state: *mut Game_State);
}

fn main() -> core::common::Maybe_Error {}
