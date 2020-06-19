use ecs_engine::common::transform::Transform2D;
use ecs_engine::ecs::components::base::C_Spatial2D;
use ecs_engine::ecs::ecs_world::Ecs_World;
use ecs_engine::gfx::render_window::{mouse_pos_in_world, Render_Window_Handle};

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Entity_Preview;

pub fn update(world: &mut Ecs_World, window: &Render_Window_Handle, camera: &Transform2D) {
    foreach_entity!(world, +C_Spatial2D, +C_Entity_Preview, |entity| {
        let spatial = world.get_component_mut::<C_Spatial2D>(entity).unwrap();
        let mpos = mouse_pos_in_world(window, camera);
        spatial.transform.set_position_v(mpos);
    });
}
