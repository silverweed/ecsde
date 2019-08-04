use crate::cfg;
use crate::core::world::World;
use crate::input::input_system::Game_Action;
use crate::states::state::Persistent_Game_State;

pub struct Engine_Base_State {}

impl Persistent_Game_State for Engine_Base_State {
    fn handle_actions(
        &mut self,
        _actions: &[Game_Action],
        _world: &World,
        _config: &cfg::Config,
    ) -> bool {
        false
    }
}
