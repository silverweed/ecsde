use crate::collisions::Game_Collision_Layer;
use inle_ecs::ecs_world::Ecs_World;
use inle_physics::collider::C_Collider;
use inle_physics::phys_world::Physics_World;

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Ground_Detection {
    pub touching_ground: bool,
    pub just_touched_ground: bool,
    pub just_left_ground: bool,
}

const GROUND_Y_COMP_THRESHOLD: f32 = -0.9;

pub fn update(world: &mut Ecs_World, phys_world: &Physics_World) {
    trace!("ground_detection_system::update");

    foreach_entity!(world, +C_Collider, +C_Ground_Detection, |entity| {
        let cld = world.get_component::<C_Collider>(entity).unwrap();
        let cld = phys_world.get_collider(cld.handle).unwrap();
        let touching_ground = cld.colliding_with.iter().any(|cls_data| {
            debug_assert!(cls_data.info.normal.is_normalized());
            if cls_data.info.normal.y > GROUND_Y_COMP_THRESHOLD {
                return false;
            }

            let other_cld = world.get_component::<C_Collider>(cls_data.entity).unwrap();
            if let Some(other_cld) = phys_world.get_collider(other_cld.handle) {
                other_cld.layer == Game_Collision_Layer::Ground as _
            } else {
                false
            }
        });

        let ground_detect = world.get_component_mut::<C_Ground_Detection>(entity).unwrap();
        ground_detect.just_touched_ground = touching_ground &&  ground_detect.touching_ground != touching_ground;
        ground_detect.just_left_ground = !touching_ground &&  ground_detect.touching_ground != touching_ground;
        ground_detect.touching_ground = touching_ground;
    });
}
