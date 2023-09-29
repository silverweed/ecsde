use crate::res::{Shader_Handle, Texture_Handle};
use inle_common::colors;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Material {
    pub texture: Texture_Handle,
    pub normals: Texture_Handle,
    pub shader: Shader_Handle,
    pub specular_color: colors::Color,
    pub shininess: u16, // This gets normalized to obtain a value [0.0, MAX_SHININESS].
    pub cast_shadows: bool,
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
            lwarn!(
                "value {} passed to encode_shininess is greater than max {} and will be capped.",
                sh,
                Self::MAX_SHININESS
            );
        }
        (sh.min(Self::MAX_SHININESS) / Self::MAX_SHININESS * std::u16::MAX as f32) as _
    }

    pub fn decode_shininess(sh: u16) -> f32 {
        (sh as f32) / std::u16::MAX as f32 * Self::MAX_SHININESS
    }
}
