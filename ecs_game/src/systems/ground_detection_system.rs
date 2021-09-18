use inle_ecs::ecs_world::Ecs_World;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;
use inle_physics::physics::Physics_Settings;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Ground_Detection {
    pub touching_ground: bool,
    pub just_touched_ground: bool,
    pub just_left_ground: bool,
}

const GROUND_Y_COMP_THRESHOLD: f32 = -0.9;

pub fn update(
    world: &mut Ecs_World,
    phys_world: &Physics_World,
    physics_settings: &Physics_Settings,
) {
    trace!("ground_detection_system::update");

    foreach_entity_new!(world,
        read: C_Collider;
        write: C_Ground_Detection;
        |entity, (collider,): (&C_Collider,), (ground_detect,): (&mut C_Ground_Detection,)| {
        let cld_handle = collider.handle;
        let touching_ground = if let Some(collisions) = phys_world.get_collisions(cld_handle) {
            let cld = phys_world.get_collider(cld_handle).unwrap();
             collisions.iter().any(|cls_data| {
                debug_assert!(cls_data.info.normal.is_normalized(),
                    "normal is not normalized: magnitude is {}",
                    cls_data.info.normal.magnitude());
                if cls_data.info.normal.y > GROUND_Y_COMP_THRESHOLD {
                    return false;
                }

                if let Some(other_cld) = phys_world.get_collider(cls_data.other_collider) {
                    physics_settings.collision_matrix.layers_collide(cld.layer, other_cld.layer)
                } else {
                    false
                }
            })
        } else {
            false
        };

        ground_detect.just_touched_ground = touching_ground &&  ground_detect.touching_ground != touching_ground;
        ground_detect.just_left_ground = !touching_ground &&  ground_detect.touching_ground != touching_ground;
        ground_detect.touching_ground = touching_ground;
    });
}
