use crate::common::colors::Color;
use crate::common::rect::{Rect, Rectf};
use crate::common::shapes;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::gfx::paint_props::Paint_Properties;
use crate::gfx::window::{get_blend_mode, Window_Handle};
use sfml::graphics::{
    CircleShape, PrimitiveType, RectangleShape, RenderStates, RenderTarget, Shape, Transformable,
};
use sfml::system::Vector2f;

// @Speed: probably replace this with a VertexBuffer
pub type Vertex_Buffer = sfml::graphics::VertexArray;
pub type Vertex = sfml::graphics::Vertex;

sf_wrap!(Texture, sfml::graphics::Texture);
sf_wrap!(Font, sfml::graphics::Font);

pub type Text<'a> = sfml::graphics::Text<'a>;

fn set_text_paint_props(text: &mut Text, paint_props: &Paint_Properties) {
    text.set_fill_color(paint_props.color.into());
    text.set_outline_color(paint_props.border_color.into());
    text.set_outline_thickness(paint_props.border_thick);
}

pub fn render_text(
    window: &mut Window_Handle,
    text: &mut Text,
    paint_props: &Paint_Properties,
    screen_pos: Vec2f,
) {
    text.set_position(Vector2f::from(screen_pos));
    set_text_paint_props(text, paint_props);

    window.raw_handle_mut().draw(text);
}

pub fn render_text_ws(
    window: &mut Window_Handle,
    text: &mut Text,
    paint_props: &Paint_Properties,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mut render_transform = camera.get_matrix_sfml().inverse();
    render_transform.combine(&transform.get_matrix_sfml());

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };

    set_text_paint_props(text, paint_props);

    window
        .raw_handle_mut()
        .draw_with_renderstates(text, render_states);
}

pub fn fill_color_rect<T>(window: &mut Window_Handle, paint_props: &Paint_Properties, rect: T)
where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    fill_color_rect_internal(
        window,
        paint_props,
        rect,
        RenderStates {
            blend_mode: get_blend_mode(window),
            ..Default::default()
        },
    );
}

pub fn fill_color_rect_ws<T>(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let mut render_transform = camera.get_matrix_sfml().inverse();
    render_transform.combine(&transform.get_matrix_sfml());

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };

    fill_color_rect_internal(window, paint_props, rect, render_states);
}

// @Refactoring: we should migrate shape code outside sfml and leverage
// render_texture_ws for everything.
fn fill_color_rect_internal<T>(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    render_states: RenderStates,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let mut rectangle_shape = RectangleShape::new();
    let rect = rect.into();
    rectangle_shape.set_position(Vector2f::new(rect.x, rect.y));
    rectangle_shape.set_size(Vector2f::new(rect.width, rect.height));
    rectangle_shape.set_fill_color(paint_props.color.into());
    rectangle_shape.set_outline_thickness(paint_props.border_thick);
    rectangle_shape.set_outline_color(paint_props.border_color.into());
    window
        .raw_handle_mut()
        .draw_rectangle_shape(&rectangle_shape, render_states);
}

pub fn fill_color_circle_ws(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    let render_transform = camera.get_matrix_sfml().inverse();

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };

    fill_color_circle_internal(window, paint_props, circle, render_states);
}

fn fill_color_circle_internal(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    render_states: RenderStates,
) {
    let mut circle_shape = CircleShape::new(circle.radius, paint_props.point_count);
    circle_shape.set_position(Vector2f::new(circle.center.x, circle.center.y));
    circle_shape.set_fill_color(paint_props.color.into());
    circle_shape.set_outline_thickness(paint_props.border_thick);
    circle_shape.set_outline_color(paint_props.border_color.into());
    window
        .raw_handle_mut()
        .draw_circle_shape(&circle_shape, render_states);
}

#[inline(always)]
pub fn get_texture_size(texture: &sfml::graphics::Texture) -> (u32, u32) {
    let s = texture.size();
    (s.x, s.y)
}

#[inline(always)]
pub fn get_text_size(text: &sfml::graphics::Text<'_>) -> Vec2f {
    let Rect { width, height, .. } = text.local_bounds().into();
    v2!(width, height)
}

// @Robustness @Cleanup: we may want to refactor this code
pub fn start_draw_quads(_n_quads: usize) -> Vertex_Buffer {
    sfml::graphics::VertexArray::new(PrimitiveType::Quads, 0)
}

pub fn start_draw_triangles(_n_tris: usize) -> Vertex_Buffer {
    sfml::graphics::VertexArray::new(PrimitiveType::Triangles, 0)
}

pub fn start_draw_linestrip(_n_vertices: usize) -> Vertex_Buffer {
    sfml::graphics::VertexArray::new(PrimitiveType::LineStrip, 0)
}

pub fn start_draw_lines(_n_lines: usize) -> Vertex_Buffer {
    sfml::graphics::VertexArray::new(PrimitiveType::Lines, 0)
}

pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    vbuf.append(v1);
    vbuf.append(v2);
    vbuf.append(v3);
    vbuf.append(v4);
}

pub fn add_triangle(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    vbuf.append(v1);
    vbuf.append(v2);
    vbuf.append(v3);
}

pub fn add_vertex(vbuf: &mut Vertex_Buffer, v: &Vertex) {
    vbuf.append(v);
}

pub fn add_line(vbuf: &mut Vertex_Buffer, from: &Vertex, to: &Vertex) {
    vbuf.append(from);
    vbuf.append(to);
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex::new(Vector2f::from(pos), col.into(), Vector2f::from(tex_coords))
}

pub fn render_vbuf(window: &mut Window_Handle, vbuf: &Vertex_Buffer, transform: &Transform2D) {
    let render_states = RenderStates {
        transform: transform.get_matrix_sfml(),
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

pub fn render_vbuf_ws(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mut render_transform = camera.get_matrix_sfml().inverse();
    render_transform.combine(&transform.get_matrix_sfml());

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

pub fn render_vbuf_texture(window: &mut Window_Handle, vbuf: &Vertex_Buffer, texture: &Texture) {
    let render_states = RenderStates {
        blend_mode: get_blend_mode(window),
        texture: Some(texture),
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

fn render_vbuf_internal(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    render_states: RenderStates,
) {
    window.draw_vertex_array(vbuf, render_states);
}

pub fn create_text<'a>(string: &str, font: &'a Font, size: u16) -> Text<'a> {
    Text::new(string, font, size as u32)
}

pub fn get_text_local_bounds(text: &Text) -> Rectf {
    text.local_bounds().into()
}

pub fn render_line(window: &mut Window_Handle, start: &Vertex, end: &Vertex) {
    let vertices: [sfml::graphics::Vertex; 2] = [*start, *end];
    window.draw_primitives(&vertices, PrimitiveType::Lines, RenderStates::default());
}
