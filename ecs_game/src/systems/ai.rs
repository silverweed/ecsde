use inle_ecs::ecs_world::Ecs_World;
use inle_physics::phys_world::Physics_World;
use std::time::Duration;

mod test_ai_system;

#[derive(Default)]
pub struct Ai_System {}

impl Ai_System {
    pub fn update(
        &mut self,
        ecs_world: &mut Ecs_World,
        phys_world: &Physics_World,
        _dt: &Duration,
    ) {
        test_ai_system::update(ecs_world, phys_world);
    }
}
