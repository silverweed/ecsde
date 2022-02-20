use super::interface::{Game_System, Update_Args};
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_physics::collider::C_Collider;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Ground_Detection {
    pub touching_ground: bool,
    pub just_touched_ground: bool,
    pub just_left_ground: bool,
}

const GROUND_Y_COMP_THRESHOLD: f32 = -0.9;

pub struct Ground_Detection_System {
    query: Ecs_Query,
}

impl Ground_Detection_System {
    pub fn new() -> Self {
        Self {
            query: Ecs_Query::new()
                .read::<C_Collider>()
                .write::<C_Ground_Detection>(),
        }
    }
}

impl Game_System for Ground_Detection_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn update(&self, args: &mut Update_Args) {
        trace!("ground_detection_system::update");

        let Update_Args {
            ecs_world: world,
            phys_world,
            phys_settings,
            ..
        } = args;

        foreach_entity!(self.query, world,
            read: C_Collider;
            write: C_Ground_Detection;
            |_e, (collider,): (&C_Collider,), (ground_detect,): (&mut C_Ground_Detection,)| {
            let touching_ground = {
                let phys_body = phys_world.get_physics_body(collider.phys_body_handle).unwrap();
                let rb_handle = phys_body.rigidbody_colliders[0].0;
                let collisions = phys_world.get_collisions(rb_handle);
                let cld = phys_world.get_collider(rb_handle).unwrap();
                 collisions.iter().any(|cls_data| {
                    debug_assert!(cls_data.info.normal.is_normalized(),
                        "normal is not normalized: magnitude is {}",
                        cls_data.info.normal.magnitude());
                    if cls_data.info.normal.y > GROUND_Y_COMP_THRESHOLD {
                        return false;
                    }

                    if let Some(other_cld) = phys_world.get_collider(cls_data.other_collider) {
                        phys_settings.collision_matrix.layers_collide(cld.layer, other_cld.layer)
                    } else {
                        false
                    }
                })
            };

            ground_detect.just_touched_ground = touching_ground &&  ground_detect.touching_ground != touching_ground;
            ground_detect.just_left_ground = !touching_ground &&  ground_detect.touching_ground != touching_ground;
            ground_detect.touching_ground = touching_ground;
        });
    }
}
