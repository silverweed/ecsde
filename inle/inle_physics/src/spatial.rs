use super::phys_world::Physics_World;
use inle_math::vector::Vec2f;

pub trait Spatial_Accelerator<T> {
    fn get_neighbours<R>(
        &self,
        pos: Vec2f,
        extent: Vec2f,
        phys_world: &Physics_World,
        result: &mut R,
    ) where
        R: Extend<T>;
}
