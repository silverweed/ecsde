use crate::render_window::Render_Window_Handle;
use inle_math::matrix::Matrix3;
use inle_math::rect::Recti;
use inle_math::transform::Transform2D;
use inle_win::window::Camera;

#[cfg(feature = "gfx-null")]
pub mod null;

#[cfg(feature = "gfx-gl")]
pub mod gl;

#[cfg(feature = "gfx-null")]
pub use self::null as backend;

#[cfg(feature = "gfx-gl")]
pub use self::gl as backend;

pub type Text = backend::Text;
pub type Font = backend::Font;
pub type Texture = backend::Texture;
pub type Shader = backend::Shader;
pub type Image = backend::Image;

pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;
pub enum Render_Settings<'a> {
    Basic,
    With_Texture(&'a Texture),
    With_Shader(&'a Shader),
}

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
pub use backend::new_font;
pub use backend::new_image;
pub use backend::new_image_with_data;
pub use backend::new_shader;
pub use backend::new_texture_from_image;
pub use backend::set_image_pixel;
pub use backend::set_texture_repeated;
pub use backend::shaders_are_available;

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Color_Type {
    Grayscale,
    RGB,
    Indexed,
    Grayscale_Alpha,
    RGBA,
}

pub struct Font_Metadata {
    // @Temporary: we want to support more than ASCII
    glyph_data: [Glyph_Data; 256],
    pub atlas_size: (u32, u32),
    pub max_glyph_height: f32,
}

impl Font_Metadata {
    pub fn with_atlas_size(width: u32, height: u32) -> Self {
        Self {
            atlas_size: (width, height),
            glyph_data: [Glyph_Data::default(); 256],
            max_glyph_height: 0.,
        }
    }

    pub fn add_glyph_data(&mut self, glyph_id: char, data: Glyph_Data) {
        if (glyph_id as usize) < 256 {
            self.glyph_data[glyph_id as usize] = data;
            if data.plane_bounds.height() > self.max_glyph_height {
                self.max_glyph_height = data.plane_bounds.height();
            }
        } else {
            lwarn!("We currently don't support non-ASCII characters: discarding glyph data for {} (0x{:X})"
                , glyph_id, glyph_id as usize);
        }
    }

    fn get_glyph_data(&self, glyph: char) -> Option<&Glyph_Data> {
        // @Temporary
        if (glyph as usize) < self.glyph_data.len() {
            Some(&self.glyph_data[glyph as usize])
        } else {
            None
        }
    }

    /// plane_bounds * scale_factor = size_of_glyph_in_pixel
    fn scale_factor(&self, font_size: f32) -> f32 {
        let base_line_height = self.max_glyph_height;
        debug_assert!(base_line_height > 0.);

        // NOTE: this scale factor is chosen so the maximum possible text height is equal to font_size px.
        // We may want to change this and use font_size as the "main corpus" size,
        // but for now it seems like a reasonable choice.
        font_size / base_line_height
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Glyph_Data {
    pub advance: f32,

    /// Bounding box relative to the baseline
    pub plane_bounds: Glyph_Bounds,

    /// Normalized coordinates (uv) inside atlas
    pub normalized_atlas_bounds: Glyph_Bounds,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Glyph_Bounds {
    pub left: f32,
    pub bot: f32,
    pub right: f32,
    pub top: f32,
}

impl Glyph_Bounds {
    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.top - self.bot
    }
}

//pub(crate) use backend::new_shader_internal;

pub fn set_uniform<T: Uniform_Value>(shader: &mut Shader, name: &std::ffi::CStr, val: T) {
    val.apply_to(shader, name);
}

#[inline]
pub fn get_mvp_matrix(transform: &Transform2D, camera: &Camera) -> Matrix3<f32> {
    get_vp_matrix(camera) * transform.get_matrix()
}

#[inline]
pub fn get_vp_matrix(camera: &Camera) -> Matrix3<f32> {
    let view_rect = inle_win::window::get_camera_viewport(camera);
    let view = get_view_matrix(&camera.transform);
    let projection = Matrix3::new(
        2. / view_rect.width,
        0.,
        0.,
        0.,
        -2. / view_rect.height,
        0.,
        0.,
        0.,
        1.,
    );
    projection * view
}

/// Note: we use the camera scale as the zoom factor.
/// Since the view matrix doesn't want to be scaled, this function just transforms a camera transform
/// into a view matrix by setting its scale to 1, 1
#[inline]
pub fn get_view_matrix(camera: &Transform2D) -> Matrix3<f32> {
    let mut view = *camera;
    view.set_scale(1., 1.);
    view.inverse().get_matrix()
}

#[inline]
pub fn get_inverse_view_matrix(camera: &Transform2D) -> Matrix3<f32> {
    let mut view = *camera;
    view.set_scale(1., 1.);
    view.get_matrix()
}
