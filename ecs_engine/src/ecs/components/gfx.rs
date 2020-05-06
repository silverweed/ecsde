use crate::common::colors;
use crate::common::rect::Rect;
use crate::common::transform::Transform2D;
use crate::gfx::render;
use crate::resources::gfx::{Shader_Handle, Texture_Handle};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Material {
    pub texture: Texture_Handle,
    pub normals: Texture_Handle,
    pub shader: Shader_Handle,
    pub specular_color: colors::Color,
    pub shininess: u16, // This gets normalized to obtain a value [0.0, MAX_SHININESS].
}

impl Material {
    pub const MAX_SHININESS: f32 = 1000.0;

    pub fn with_texture(texture: Texture_Handle) -> Self {
        Self {
            texture,
            ..Default::default()
        }
    }

    pub fn encode_shininess(sh: f32) -> u16 {
        if sh > Self::MAX_SHININESS {
            lwarn!("value {} passed to encode_shininess is greater than max {} and will be capped.", sh, Self::MAX_SHININESS);
        }
        (sh.min(Self::MAX_SHININESS) / Self::MAX_SHININESS * std::u16::MAX as f32) as _
    }

    pub fn decode_shininess(sh: u16) -> f32 {
        (sh as f32) / std::u16::MAX as f32 * Self::MAX_SHININESS
    }
}

#[derive(Copy, Clone, Debug)]
pub struct C_Renderable {
    pub material: Material,
    pub rect: Rect<i32>,
    pub modulate: colors::Color,
    pub z_index: render::Z_Index,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            material: Material {
                texture: Texture_Handle::default(),
                normals: Texture_Handle::default(),
                shader: Shader_Handle::default(),
                shininess: 0,
                specular_color: colors::WHITE,
            },
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
