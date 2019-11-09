use libloading as ll;

// Note: this is an opaque type
#[repr(C)]
pub struct Game_State {
    _private: [u8; 0],
}

pub struct Game_Api<'lib> {
    pub init: ll::Symbol<'lib, unsafe extern "C" fn() -> *mut Game_State>,
    pub update: ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State) -> bool>,
    pub shutdown: ll::Symbol<'lib, unsafe extern "C" fn(*mut Game_State)>,
}
