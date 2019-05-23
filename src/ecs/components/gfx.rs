use crate::resources;
use sdl2::rect::Rect;
use typename::TypeName;

#[derive(Copy, Clone, Debug, TypeName)]
pub struct C_Renderable {
    pub texture: resources::Texture_Handle,
    pub rect: Rect,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            texture: resources::Texture_Handle::default(),
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
