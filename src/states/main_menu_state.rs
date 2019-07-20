use super::state::Game_State;
use crate::core::world::World;

pub struct Main_Menu_State {}

impl Game_State for Main_Menu_State {
    fn on_start(&mut self, world: &World) {}
}
