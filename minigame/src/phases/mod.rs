use crate::{Game_Resources, Game_State};
use std::cell::{Ref, RefCell, RefMut};

pub mod menu;

pub use menu::*;

pub struct Phase_Args {
    game_state: RefCell<*mut Game_State>,
    game_res: RefCell<*mut Game_Resources>,
}

impl Phase_Args {
    pub fn new(game_state: &mut Game_State, game_res: &mut Game_Resources) -> Self {
        Self {
            game_state: RefCell::new(game_state as *mut _),
            game_res: RefCell::new(game_res as *mut _),
        }
    }

    pub fn game_state(&self) -> Ref<'_, Game_State> {
        Ref::map(self.game_state.borrow(), |ptr| unsafe { &**ptr })
    }

    pub fn game_state_mut(&self) -> RefMut<'_, Game_State> {
        RefMut::map(self.game_state.borrow_mut(), |ptr| unsafe { &mut **ptr })
    }

    pub fn game_res(&self) -> Ref<'_, Game_Resources> {
        Ref::map(self.game_res.borrow(), |ptr| unsafe { &**ptr })
    }

    pub fn game_res_mut(&self) -> RefMut<'_, Game_Resources> {
        RefMut::map(self.game_res.borrow_mut(), |ptr| unsafe { &mut **ptr })
    }
}
