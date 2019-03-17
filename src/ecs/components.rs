use crate::resources::resources;
use sfml::graphics::Sprite;
use std::fmt::Debug;

use typename::TypeName;

pub trait Component: Clone + Default + Debug + TypeName {}
impl<T> Component for T where T: Clone + Default + Debug + TypeName {}

#[derive(Copy, Clone, Default, Debug, TypeName, PartialEq)] // @Convenience: there's gotta be a better way to say this is a Component
pub struct C_Position2D {
    pub x: f32,
    pub y: f32,
}

impl Eq for C_Position2D {}

#[derive(Clone, Default, Debug, TypeName)]
pub struct C_Renderable {
    pub sprite: resources::Sprite_Handle,
}
