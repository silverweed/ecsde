use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;

pub struct C_Test_Ai {}

pub fn update(ecs_world: &mut Ecs_World, phys_world: &Physics_World) {
    foreach_entity!(ecs_world,
        read: C_Collider, C_Test_Ai;
        write: C_Spatial2D;
    |_e, (cld, _ai): (&C_Collider, &C_Test_Ai), (spatial,): (&mut C_Spatial2D,)| {
        if let Some(collisions) = phys_world.get_collisions(cld.handle) {
            for collision in collisions {
                let other_cld = phys_world.get_collider(collision.other_collider).unwrap();
                // @Incomplete: solid check
                if collision.info.normal.x > 0.5 {
                    spatial.velocity.x *= -1.0;
                }
            }
        }
    });
}
