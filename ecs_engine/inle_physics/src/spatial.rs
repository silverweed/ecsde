use crate::common::vector::Vec2f;

pub trait Spatial_Accelerator<T> {
    fn get_neighbours<R>(&self, pos: Vec2f, extent: Vec2f, result: &mut R)
    where
        R: Extend<T>;
}
