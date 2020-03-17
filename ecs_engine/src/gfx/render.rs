use super::paint_props::Paint_Properties;
use crate::common::colors::Color;
use crate::common::rect::{Rect, Rectf};
use crate::common::shapes::Circle;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::{Font_Handle, Gfx_Resources, Texture_Handle};
use std::convert::Into;

pub mod batcher;

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

pub type Text<'a> = backend::Text<'a>;
pub type Font<'a> = backend::Font<'a>;
pub type Texture<'a> = backend::Texture<'a>;

pub type Vertex_Buffer = backend::Vertex_Buffer;
pub type Vertex = backend::Vertex;

// Note: this struct cannot be mutated after creation
#[derive(Clone, Default)]
pub struct Text_Props {
    m_string: String,
    m_font: Font_Handle,
    m_font_size: u16,
    m_size: std::cell::Cell<Option<Vec2f>>,
}

impl Text_Props {
    pub fn string(&self) -> &str {
        &self.m_string
    }
    pub fn owned_string(self) -> String {
        self.m_string
    }
    pub fn font(&self) -> Font_Handle {
        self.m_font
    }
    pub fn font_size(&self) -> u16 {
        self.m_font_size
    }
}

//////////////////////////// DRAWING //////////////////////////////////

/// Draws a color-filled rectangle in screen space
pub fn render_rect<R, P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    rect: R,
    paint_props: P,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    trace!("fill_color_rect");
    let paint_props = paint_props.into();
    // @Temporary measure until the batcher supports outlines et al.
    if paint_props.border_thick == 0. {
        batcher::add_rect(batches, &rect.into(), &paint_props);
    } else {
        backend::fill_color_rect(window, &paint_props, rect);
    }
}

/// Draws a color-filled rectangle in world space
pub fn render_rect_ws<R, P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    rect: R,
    paint_props: P,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
    P: Into<Paint_Properties>,
{
    trace!("fill_color_rect_ws");
    let paint_props = paint_props.into();
    // @Temporary measure until the batcher supports outlines et al.
    if paint_props.border_thick == 0. {
        batcher::add_rect_ws(batches, &rect.into(), &paint_props, transform);
    } else {
        backend::fill_color_rect_ws(window, &paint_props, rect, transform, camera);
    }
}

/// Draws a color-filled circle in world space
pub fn render_circle_ws<P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    circle: Circle,
    paint_props: P,
    camera: &Transform2D,
) where
    P: Into<Paint_Properties>,
{
    trace!("fill_color_circle_ws");
    let paint_props = paint_props.into();
    // @Temporary measure until the batcher supports outlines et al.
    if paint_props.border_thick == 0. {
        batcher::add_circle_ws(batches, &circle, &paint_props);
    } else {
        backend::fill_color_circle_ws(window, &paint_props, circle, camera);
    }
}

pub fn render_texture_ws(
    batches: &mut batcher::Batches,
    texture: Texture_Handle,
    tex_rect: &Rect<i32>,
    color: Color,
    transform: &Transform2D,
) {
    trace!("render_texture_ws");
    batcher::add_texture_ws(batches, texture, tex_rect, color, transform);
}

pub fn render_text<P>(
    batches: &mut batcher::Batches,
    text: Text_Props,
    paint_props: P,
    screen_pos: Vec2f,
) where
    P: Into<Paint_Properties>,
{
    trace!("render_text");
    batcher::add_text(batches, text, &paint_props.into(), screen_pos);
}

pub fn render_text_ws<P>(
    batches: &mut batcher::Batches,
    text: Text_Props,
    paint_props: P,
    world_transform: &Transform2D,
) where
    P: Into<Paint_Properties>,
{
    trace!("render_text_ws");
    batcher::add_text_ws(batches, text, &paint_props.into(), world_transform);
}

pub fn render_vbuf(batches: &mut batcher::Batches, vbuf: Vertex_Buffer, transform: &Transform2D) {
    trace!("render_vbuf");
    batcher::add_vbuf(batches, vbuf, *transform);
}

pub fn render_vbuf_ws(
    batches: &mut batcher::Batches,
    vbuf: Vertex_Buffer,
    transform: &Transform2D,
) {
    trace!("render_vbuf_ws");
    batcher::add_vbuf_ws(batches, vbuf, *transform);
}

// Note: this always renders a line with thickness = 1px
pub fn render_line(batches: &mut batcher::Batches, start: &Vertex, end: &Vertex) {
    trace!("render_line");
    batcher::add_line(batches, *start, *end);
}

///////////////////////////////// QUERYING ///////////////////////////////////
pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    backend::get_texture_size(texture)
}

pub fn get_text_size(text: &Text_Props, gres: &Gfx_Resources) -> Vec2f {
    trace!("get_text_size");
    if let Some(size) = text.m_size.get() {
        size
    } else {
        let font = gres.get_font(text.font());
        let txt = Text::new(text.string(), font, text.font_size() as _);
        let bounds = backend::get_text_local_bounds(&txt);
        let size = Vec2f::new(bounds.width, bounds.height);
        text.m_size.set(Some(size));
        size
    }
}

///////////////////////////////// CREATING ///////////////////////////////////

pub fn create_text(string: &str, font: Font_Handle, font_size: u16) -> Text_Props {
    trace!("create_text");
    Text_Props {
        m_string: String::from(string),
        m_font: font,
        m_font_size: font_size,
        // We don't calculate this until we're asked to
        m_size: std::cell::Cell::new(None),
    }
}

// @Refactoring: simplify these and make it more robust via types
pub fn start_draw_quads(n_quads: usize) -> Vertex_Buffer {
    trace!("start_draw_quads");
    backend::start_draw_quads(n_quads)
}

pub fn start_draw_triangles(n_triangles: usize) -> Vertex_Buffer {
    trace!("start_draw_triangles");
    backend::start_draw_triangles(n_triangles)
}

pub fn start_draw_linestrip(n_vertices: usize) -> Vertex_Buffer {
    trace!("start_draw_linestrip");
    backend::start_draw_linestrip(n_vertices)
}

pub fn start_draw_lines(n_vertices: usize) -> Vertex_Buffer {
    trace!("start_draw_lines");
    backend::start_draw_lines(n_vertices)
}

pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    backend::add_quad(vbuf, v1, v2, v3, v4);
}

pub fn add_triangle(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    backend::add_triangle(vbuf, v1, v2, v3);
}

pub fn add_line(vbuf: &mut Vertex_Buffer, from: &Vertex, to: &Vertex) {
    backend::add_line(vbuf, from, to);
}

pub fn add_vertex(vbuf: &mut Vertex_Buffer, v: &Vertex) {
    backend::add_vertex(vbuf, v);
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    backend::new_vertex(pos, col, tex_coords)
}
