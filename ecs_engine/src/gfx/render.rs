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
    m_bounds: std::cell::Cell<Option<Rectf>>,
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

/// Draws a color-filled rectangle in screen space
pub fn fill_color_rect<R, P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    paint_props: P,
    rect: R,
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
pub fn fill_color_rect_ws<R, P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    paint_props: P,
    rect: R,
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
pub fn fill_color_circle_ws<P>(
    window: &mut Window_Handle,
    batches: &mut batcher::Batches,
    paint_props: P,
    circle: Circle,
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

pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    backend::get_texture_size(texture)
}

pub fn create_text(string: &str, font: Font_Handle, size: u16) -> Text_Props {
    trace!("create_text");
    Text_Props {
        m_string: String::from(string),
        m_font: font,
        m_font_size: size,
        // We don't calculate this until we're asked to
        m_bounds: std::cell::Cell::new(None),
    }
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

pub fn get_text_local_bounds(text: &Text_Props, gres: &Gfx_Resources) -> Rectf {
    trace!("get_text_local_bounds");
    if let Some(bounds) = text.m_bounds.get() {
        bounds
    } else {
        let font = gres.get_font(text.font());
        let txt = Text::new(text.string(), font, text.font_size() as _);
        let bounds = backend::get_text_local_bounds(&txt);
        text.m_bounds.set(Some(bounds));
        bounds
    }
}

pub fn start_draw_quads(n_quads: usize) -> Vertex_Buffer {
    trace!("start_draw_quads");
    backend::start_draw_quads(n_quads)
}

pub fn start_draw_triangles(n_triangles: usize) -> Vertex_Buffer {
    trace!("start_draw_triangles");
    backend::start_draw_triangles(n_triangles)
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

// @Refactoring: make this pass through the batcher
pub fn render_vbuf(window: &mut Window_Handle, vbuf: &Vertex_Buffer, transform: &Transform2D) {
    trace!("render_vbuf");
    backend::render_vbuf(window, vbuf, transform);
}

// @Refactoring: make this pass through the batcher
pub fn render_vbuf_ws(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    trace!("render_vbuf_ws");
    backend::render_vbuf_ws(window, vbuf, transform, camera);
}

pub fn start_draw_linestrip(n_vertices: usize) -> Vertex_Buffer {
    trace!("start_draw_linestrip");
    backend::start_draw_linestrip(n_vertices)
}

pub fn start_draw_lines(n_vertices: usize) -> Vertex_Buffer {
    trace!("start_draw_lines");
    backend::start_draw_lines(n_vertices)
}

// Note: this always renders a line with thickness = 1px
pub fn render_line(batcher: &mut batcher::Batches, start: &Vertex, end: &Vertex) {
    trace!("render_line");
    batcher::add_line(batcher, *start, *end);
}
