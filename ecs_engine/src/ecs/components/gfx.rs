use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::gfx::render;
use crate::resources;

#[derive(Copy, Clone, Debug)]
pub struct C_Renderable {
    pub texture: resources::gfx::Texture_Handle,
    pub rect: Rect<i32>,
    pub modulate: colors::Color,
    pub z_index: render::Z_Index,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            texture: resources::gfx::Texture_Handle::default(),
            rect: Rect::new(0, 0, 0, 0),
            modulate: colors::WHITE,
            z_index: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Animated_Sprite {
    pub n_frames: u32,
    pub frame_time: f32,
    pub frame_time_elapsed: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Camera2D {
    pub transform: Transform2D,
}
