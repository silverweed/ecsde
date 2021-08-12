use crate::material::Material;
use crate::render;
use inle_common::colors;
use inle_core::env::Env_Info;
use inle_math::rect::Rect;
use inle_math::transform::Transform2D;
use inle_resources::gfx::{shader_path, tex_path, Gfx_Resources, Shader_Cache};

#[derive(Copy, Clone, Debug)]
pub struct C_Renderable {
    pub material: Material,
    pub rect: Rect<i32>,
    pub modulate: colors::Color, // @Redundant: we're already passing color to render_texture_ws! Do we care about this?
    pub z_index: render::Z_Index,
}

impl Default for C_Renderable {
    fn default() -> Self {
        C_Renderable {
            material: Material {
                specular_color: colors::WHITE,
                ..Default::default()
            },
            rect: Rect::new(0, 0, 0, 0),
            modulate: colors::WHITE,
            z_index: 0,
        }
    }
}

impl C_Renderable {
    pub fn new_with_diffuse(gres: &mut Gfx_Resources, env: &Env_Info, diffuse: &str) -> Self {
        let texture = gres.load_texture(&tex_path(env, diffuse));
        let (sw, sh) = render::get_texture_size(gres.get_texture(texture));
        C_Renderable {
            material: Material {
                texture,
                ..Default::default()
            },
            rect: Rect::new(0, 0, sw as i32, sh as i32),
            ..Default::default()
        }
    }

    pub fn with_n_frames<T: Into<i32>>(mut self, n_frames: T) -> Self {
        self.rect.width /= n_frames.into();
        self
    }

    pub fn with_normals(mut self, gres: &mut Gfx_Resources, env: &Env_Info, normals: &str) -> Self {
        let texture = gres.load_texture(&tex_path(env, normals));
        self.material.normals = texture;
        self
    }

    pub fn with_shader(
        mut self,
        shader_cache: &mut Shader_Cache,
        env: &Env_Info,
        shader: &str,
    ) -> Self {
        let shader = shader_cache.load_shader(&shader_path(env, shader));
        self.material.shader = shader;
        self
    }

    pub fn with_z_index(mut self, z_index: render::Z_Index) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_cast_shadows(mut self, cast_shadows: bool) -> Self {
        self.material.cast_shadows = cast_shadows;
        self
    }

    pub fn with_shininess(mut self, shininess: f32) -> Self {
        self.material.shininess = Material::encode_shininess(shininess);
        self
    }

    pub fn with_color(mut self, color: colors::Color) -> Self {
        self.modulate = color;
        self
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Multi_Renderable {
    pub renderables: [C_Renderable; Self::MAX_RENDERABLES],
    /// Local transforms relative to the entity
    pub rend_transforms: [Transform2D; Self::MAX_RENDERABLES],
    pub n_renderables: u8,
}

impl C_Multi_Renderable {
    pub const MAX_RENDERABLES: usize = 32;

    pub fn add(&mut self, renderable: C_Renderable) {
        assert!((self.n_renderables as usize) < self.renderables.len());
        self.renderables[self.n_renderables as usize] = renderable;
        self.n_renderables += 1;
    }

    pub fn add_with_local_transform(&mut self, renderable: C_Renderable, transform: &Transform2D) {
        self.add(renderable);
        self.rend_transforms[self.n_renderables as usize - 1] = *transform;
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
