#[cfg(feature = "gfx-sfml")]
pub mod sfml;

#[cfg(feature = "gfx-null")]
pub mod null;

#[cfg(feature = "gfx-gl")]
pub mod gl;

#[cfg(feature = "gfx-sfml")]
pub use self::sfml as backend;

#[cfg(feature = "gfx-null")]
pub use self::null as backend;

#[cfg(feature = "gfx-gl")]
pub use self::gl as backend;

pub type Text<'a> = backend::Text<'a>;
pub type Font<'a> = backend::Font<'a>;
pub type Texture<'a> = backend::Texture<'a>;
pub type Shader<'a> = backend::Shader<'a>;
pub type Image = backend::Image;

pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;
pub type Color_Type = backend::Color_Type;

#[derive(Default)]
pub struct Render_Extra_Params<'t, 's> {
    pub texture: Option<&'t Texture<'t>>,
    pub shader: Option<&'s Shader<'s>>,
}

#[derive(Copy, Clone, Debug)]
pub enum Primitive_Type {
    Points,
    Lines,
    Line_Strip,
    Triangles,
    Triangle_Strip,
    Triangle_Fan,
}

pub trait Uniform_Value: Copy {
    fn apply_to(self, shader: &mut Shader, name: &std::ffi::CStr);
}

pub use backend::geom_shaders_are_available;
pub use backend::new_image;
pub use backend::new_image_with_data;
pub use backend::new_shader;
pub use backend::new_texture_from_image;
pub use backend::set_image_pixel;
pub use backend::set_texture_repeated;
pub use backend::shaders_are_available;

pub(crate) use backend::new_shader_internal;

pub fn set_uniform<T: Uniform_Value>(shader: &mut Shader, name: &std::ffi::CStr, val: T) {
    val.apply_to(shader, name);
}
