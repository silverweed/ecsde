use super::transform::C_Transform2D;
use crate::core::common::rect::Rect;
use crate::resources;
use typename::TypeName;

#[derive(Copy, Clone, Debug, TypeName)]
pub struct C_Renderable {
    pub texture: resources::gfx::Texture_Handle,
    pub rect: Rect<i32>,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            texture: resources::gfx::Texture_Handle::default(),
            rect: Rect::new(0, 0, 0, 0),
        }
    }
}

#[derive(Copy, Clone, Debug, TypeName, Default)]
pub struct C_Animated_Sprite {
    pub n_frames: u32,
    pub frame_time: f32,
    pub frame_time_elapsed: f32,
}

#[derive(Copy, Clone, Debug, TypeName, Default)]
pub struct C_Camera2D {
    pub transform: C_Transform2D,
}
