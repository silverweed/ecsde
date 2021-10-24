use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;

#[derive(Default)]
pub struct C_Test_Ai {
    going_left: bool,
    frames_since_latest_impact: u32,
}

const INITIAL_VELOCITY: f32 = 100.0;

pub fn update(ecs_world: &mut Ecs_World, phys_world: &Physics_World) {
    foreach_entity!(ecs_world,
        read: C_Collider;
        write: C_Spatial2D, C_Test_Ai;
    |_e, (cld,): (&C_Collider,), (spatial,ai): (&mut C_Spatial2D, &mut C_Test_Ai)| {
        ai.frames_since_latest_impact += 1;
        if ai.frames_since_latest_impact > 1 {
            if let Some(collisions) = phys_world.get_collisions(cld.handle) {
                for collision in collisions {
                    let other_cld = phys_world.get_collider(collision.other_collider).unwrap();
                    // @Incomplete: solid check
                    if collision.info.normal.x.abs() > 0.8 {
                        ai.going_left = !ai.going_left;
                        ai.frames_since_latest_impact = 0;
                        break;
                    }
                }
            }
        }
        spatial.velocity.x = (if ai.going_left { -1.0 } else { 1.0 }) * INITIAL_VELOCITY;
    });
}
