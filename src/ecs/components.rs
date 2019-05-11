use crate::resources;
use sdl2::rect::Rect;
use std::fmt::Debug;

use typename::TypeName;

pub trait Component: Clone + Default + Debug + TypeName {}
impl<T> Component for T where T: Clone + Default + Debug + TypeName {}

#[derive(Copy, Clone, Debug, TypeName, PartialEq)]
pub struct C_Position2D {
    pub x: f32,
    pub y: f32,
}

impl Eq for C_Position2D {}

#[derive(Copy, Clone, Debug, TypeName)]
pub struct C_Renderable {
    pub texture: resources::Texture_Handle,
    pub rect: Rect,

    // @Cleanup: really we want these to be in a separate struct (like C_AnimatedSprite or whatever).
    // However, currently the entity_manager does not support simultaneous mutable components borrowing,
    // so let's keep this here for now for convenience.
    pub n_frames: u32,
    pub frame_time: f32,
    pub frame_time_elapsed: f32,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            texture: resources::Texture_Handle::default(),
            rect: Rect::new(0, 0, 0, 0),
            n_frames: 0,
            frame_time: 0.0,
            frame_time_elapsed: 0.0,
        }
    }
}
