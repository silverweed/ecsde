use crate::render_window::Render_Window_Handle;
use inle_math::matrix::Matrix3;
use inle_math::transform::Transform2D;

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
pub type Font_Metadata = backend::Font_Metadata;
pub type Texture<'a> = backend::Texture<'a>;
pub type Shader<'a> = backend::Shader<'a>;
pub type Image = backend::Image;

pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;
pub type Color_Type = backend::Color_Type;

pub enum Render_Settings<'t> {
    Basic,
    With_Texture(&'t Texture<'t>),
    With_Shader(&'t Shader<'t>),
}

// @Refactoring @Cleanup: we're currently exposing data that's supposed to be backend-specific.
// Rethink this later!
pub type Glyph_Data = backend::Glyph_Data;
pub type Glyph_Bounds = backend::Glyph_Bounds;

impl Default for Render_Settings<'_> {
    fn default() -> Self {
        Self::Basic
    }
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
pub use backend::get_texture_size;
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

#[inline]
pub fn get_mvp_matrix(
    window: &Render_Window_Handle,
    transform: &Transform2D,
    camera: &Transform2D,
) -> Matrix3<f32> {
    get_vp_matrix(window, camera) * transform.get_matrix()
}

#[inline]
pub fn get_vp_matrix(window: &Render_Window_Handle, camera: &Transform2D) -> Matrix3<f32> {
    let (width, height) = inle_win::window::get_window_target_size(window);
    let view = crate::render_window::get_view_matrix(camera);
    let projection = Matrix3::new(
        2. / (width as f32 * camera.scale().x),
        0.,
        0.,
        0.,
        -2. / (height as f32 * camera.scale().y),
        0.,
        0.,
        0.,
        1.,
    );
    projection * view
}
