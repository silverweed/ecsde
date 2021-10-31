use inle_cfg::Cfg_Var;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::Ecs_World;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::{Collider_Handle, Physics_World};

pub struct C_Test_Ai {
    speed: Cfg_Var<f32>,
    left_gnd_check: Collider_Handle,
    right_gnd_check: Collider_Handle,
    going_left: bool,
    frames_since_latest_impact: u32,
}

impl C_Test_Ai {
    pub fn new(
        left_gnd_check: Collider_Handle,
        right_gnd_check: Collider_Handle,
        speed: Cfg_Var<f32>,
    ) -> Self {
        Self {
            speed,
            left_gnd_check,
            right_gnd_check,
            going_left: false,
            frames_since_latest_impact: 0,
        }
    }
}

pub fn update(ecs_world: &mut Ecs_World, phys_world: &Physics_World, config: &inle_cfg::Config) {
    foreach_entity!(ecs_world,
        read: C_Collider;
        write: C_Spatial2D, C_Test_Ai;
    |entity, (cld,): (&C_Collider,), (spatial, ai): (&mut C_Spatial2D, &mut C_Test_Ai)| {
        // Check wall impact
        ai.frames_since_latest_impact += 1;
        if ai.frames_since_latest_impact > 1 {
            let collisions = phys_world.get_collisions(cld.handle);
            for collision in collisions {
                let other_cld = phys_world.get_collider(collision.other_collider).unwrap();
                // @Incomplete: solid check
                if collision.info.normal.x.abs() > 0.8 {
                    ai.going_left = !ai.going_left;
                    ldebug!("{:?} inverting due to impact vs {:?}", entity, other_cld.entity);
                    ai.frames_since_latest_impact = 0;
                    break;
                }
            }
        }

        // Check edge of platform
        let cld_to_check = if ai.going_left {
            ai.left_gnd_check
        } else {
            ai.right_gnd_check
        };

        let collisions = phys_world.get_collisions(cld_to_check);
        if collisions.is_empty() {
            ai.going_left = !ai.going_left;
        }

        // Proceed
        let speed = ai.speed.read(config);
        spatial.velocity.x = (if ai.going_left { -1.0 } else { 1.0 }) * speed;
    });
}
