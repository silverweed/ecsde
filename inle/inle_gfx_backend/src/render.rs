#[cfg(feature = "gfx-sfml")]
pub mod sfml;

#[cfg(feature = "gfx-null")]
pub mod null;

#[cfg(feature = "gfx-sfml")]
pub use self::sfml as backend;

#[cfg(feature = "gfx-null")]
pub use self::null as backend;

pub type Text<'a> = backend::Text<'a>;
pub type Font<'a> = backend::Font<'a>;
pub type Texture<'a> = backend::Texture<'a>;
pub type Shader<'a> = backend::Shader<'a>;
pub type Image = backend::Image;

pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;

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
    Quads,
}

pub use backend::geom_shaders_are_available;
pub use backend::shaders_are_available;
pub use backend::set_image_pixel;
pub use backend::new_image;
pub use backend::new_texture_from_image;
pub use backend::set_texture_repeated;
