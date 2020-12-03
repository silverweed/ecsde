use super::Render_Extra_Params;
use crate::render_window::Render_Window_Handle;
use inle_common::colors::Color;
use inle_common::paint_props::Paint_Properties;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::{sfml::to_matrix_sfml, Transform2D};
use inle_math::vector::Vec2f;
use sfml::graphics::{
    glsl, BlendMode, CircleShape, PrimitiveType, RectangleShape, RenderStates, RenderTarget, Shape,
    Transformable, VertexBuffer, VertexBufferUsage,
};
use sfml::system::Vector2f;

pub struct Vertex_Buffer {
    buf: sfml::graphics::VertexBuffer,
    cur_vertices: u32,
}

impl std::fmt::Debug for Vertex_Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Vertex_Buffer {{ cur_vertices: {} }}", self.cur_vertices)
    }
}

pub type Vertex = sfml::graphics::Vertex;
pub type Image = sfml::graphics::Image;
pub type Shader<'texture> = sfml::graphics::Shader<'texture>;
pub type Primitive_Type = PrimitiveType;

sf_wrap!(Texture, sfml::graphics::Texture);
sf_wrap!(Font, sfml::graphics::Font);

pub type Text<'a> = sfml::graphics::Text<'a>;

fn set_text_paint_props(text: &mut Text, paint_props: &Paint_Properties) {
    text.set_fill_color(paint_props.color.into());
    text.set_outline_color(paint_props.border_color.into());
    text.set_outline_thickness(paint_props.border_thick);
}

pub fn render_text(
    window: &mut Render_Window_Handle,
    text: &mut Text,
    paint_props: &Paint_Properties,
    screen_pos: Vec2f,
) {
    text.set_position(Vector2f::from(screen_pos));
    set_text_paint_props(text, paint_props);

    window.raw_handle_mut().draw(text);
}

pub fn render_text_ws(
    window: &mut Render_Window_Handle,
    text: &mut Text,
    paint_props: &Paint_Properties,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mut render_transform = to_matrix_sfml(camera).inverse();
    render_transform.combine(&to_matrix_sfml(transform));

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };

    set_text_paint_props(text, paint_props);

    window
        .raw_handle_mut()
        .draw_with_renderstates(text, render_states);
}

pub fn fill_color_rect<T>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    fill_color_rect_internal(
        window,
        paint_props,
        rect,
        RenderStates {
            blend_mode: BlendMode::ALPHA,
            ..Default::default()
        },
    );
}

pub fn fill_color_rect_ws<T>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let mut render_transform = to_matrix_sfml(camera).inverse();
    render_transform.combine(&to_matrix_sfml(transform));

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };

    fill_color_rect_internal(window, paint_props, rect, render_states);
}

// @Refactoring: we should migrate shape code outside sfml and leverage
// render_texture_ws for everything.
fn fill_color_rect_internal<T>(
    window: &mut Render_Window_Handle,
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

pub fn fill_color_circle(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
) {
    let render_states = RenderStates {
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };

    fill_color_circle_internal(window, paint_props, circle, render_states);
}

pub fn fill_color_circle_ws(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    let render_transform = to_matrix_sfml(camera).inverse();

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };

    fill_color_circle_internal(window, paint_props, circle, render_states);
}

fn fill_color_circle_internal(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    render_states: RenderStates,
) {
    let mut circle_shape = CircleShape::new(circle.radius, paint_props.point_count);
    circle_shape.set_origin(Vector2f::new(circle.radius, circle.radius));
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
pub fn get_image_size(image: &sfml::graphics::Image) -> (u32, u32) {
    let s = image.size();
    (s.x, s.y)
}

#[inline(always)]
pub fn get_text_size(text: &sfml::graphics::Text<'_>) -> Vec2f {
    let Rect { width, height, .. } = text.local_bounds().into();
    v2!(width, height)
}

#[inline(always)]
pub fn new_vbuf(primitive: PrimitiveType, n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(primitive, n_vertices, VertexBufferUsage::Stream),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    vbuf.buf.primitive_type()
}

#[inline(always)]
pub fn start_draw_quads(n_quads: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(PrimitiveType::Quads, n_quads * 4, VertexBufferUsage::Stream),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn start_draw_triangles(n_tris: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(
            PrimitiveType::Triangles,
            n_tris * 3,
            VertexBufferUsage::Stream,
        ),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn start_draw_lines(n_lines: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(PrimitiveType::Lines, n_lines * 2, VertexBufferUsage::Stream),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn start_draw_linestrip(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(
            PrimitiveType::LineStrip,
            n_vertices,
            VertexBufferUsage::Stream,
        ),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn start_draw_points(n_vertices: u32) -> Vertex_Buffer {
    Vertex_Buffer {
        buf: VertexBuffer::new(
            PrimitiveType::Points,
            n_vertices,
            VertexBufferUsage::Stream,
        ),
        cur_vertices: 0,
    }
}

#[inline(always)]
pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    debug_assert!(vbuf.cur_vertices + 4 <= vbuf.buf.vertex_count());
    vbuf.buf.update(&[*v1, *v2, *v3, *v4], vbuf.cur_vertices);
    vbuf.cur_vertices += 4;
}

#[inline(always)]
pub fn add_triangle(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
    debug_assert!(vbuf.cur_vertices + 3 <= vbuf.buf.vertex_count());
    vbuf.buf.update(&[*v1, *v2, *v3], vbuf.cur_vertices);
    vbuf.cur_vertices += 3;
}

#[inline(always)]
pub fn add_line(vbuf: &mut Vertex_Buffer, from: &Vertex, to: &Vertex) {
    debug_assert!(vbuf.cur_vertices + 2 <= vbuf.buf.vertex_count());
    vbuf.buf.update(&[*from, *to], vbuf.cur_vertices);
    vbuf.cur_vertices += 2;
}

#[inline(always)]
pub fn add_vertex(vbuf: &mut Vertex_Buffer, v: &Vertex) {
    debug_assert!(vbuf.cur_vertices < vbuf.buf.vertex_count());
    vbuf.buf.update(&[*v], vbuf.cur_vertices);
    vbuf.cur_vertices += 1;
}

#[inline(always)]
pub fn update_vbuf(vbuf: &mut Vertex_Buffer, vertices: &[Vertex], offset: u32) {
    vbuf.buf.update(vertices, offset);
}

#[inline(always)]
pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.cur_vertices
}

#[inline(always)]
pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.buf.vertex_count()
}

#[inline(always)]
pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    vbuf.cur_vertices = cur_vertices;
}

#[inline(always)]
pub fn swap_vbuf(a: &mut Vertex_Buffer, b: &mut Vertex_Buffer) -> bool {
    a.buf.swap(&mut b.buf);
    std::mem::swap(&mut a.cur_vertices, &mut b.cur_vertices);
    true
}

#[inline(always)]
pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex::new(Vector2f::from(pos), col.into(), Vector2f::from(tex_coords))
}

pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    let render_states = RenderStates {
        transform: to_matrix_sfml(transform),
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

pub fn render_vbuf_ws(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mut render_transform = to_matrix_sfml(camera).inverse();
    render_transform.combine(&to_matrix_sfml(transform));

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: BlendMode::ALPHA,
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

pub fn render_vbuf_ws_ex(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
    extra_params: Render_Extra_Params,
) {
    let mut render_transform = to_matrix_sfml(camera).inverse();
    render_transform.combine(&to_matrix_sfml(transform));

    use std::borrow::Borrow;

    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: BlendMode::ALPHA,
        texture: extra_params.texture.map(|t| t.wrapped.borrow()),
        shader: extra_params.shader,
    };
    render_vbuf_internal(window, vbuf, render_states);
}

pub fn render_vbuf_texture(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    texture: &Texture,
) {
    let render_states = RenderStates {
        blend_mode: BlendMode::ALPHA,
        texture: Some(texture),
        ..Default::default()
    };
    render_vbuf_internal(window, vbuf, render_states);
}

fn render_vbuf_internal(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    render_states: RenderStates,
) {
    window
        .raw_handle_mut()
        .draw_vertex_buffer(&vbuf.buf, render_states);
}

#[inline(always)]
pub fn create_text<'a>(string: &str, font: &'a Font, size: u16) -> Text<'a> {
    Text::new(string, font, size as u32)
}

pub fn render_line(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    let vertices: [sfml::graphics::Vertex; 2] = [*start, *end];
    window.raw_handle_mut().draw_primitives(
        &vertices,
        PrimitiveType::Lines,
        RenderStates::default(),
    );
}

#[inline(always)]
pub fn copy_texture_to_image(texture: &Texture) -> Image {
    texture
        .copy_to_image()
        .expect("Failed to copy Texture to image!")
}

#[inline(always)]
pub fn get_image_pixel(image: &Image, x: u32, y: u32) -> Color {
    image.pixel_at(x, y).into()
}

#[inline(always)]
pub fn set_image_pixel(image: &mut Image, x: u32, y: u32, val: Color) {
    image.set_pixel(x, y, val.into());
}

#[inline(always)]
pub fn get_image_pixels(image: &Image) -> &[Color] {
    let pixel_data = image.pixel_data();
    debug_assert_eq!(pixel_data.len() % 4, 0);
    let len = pixel_data.len() / 4;
    // Safe because Color is repr(C) and SFML pixel data are packed as r,g,b,a
    unsafe { std::slice::from_raw_parts(pixel_data.as_ptr() as *const Color, len) }
}

pub fn update_texture_pixels(texture: &mut Texture, rect: &Rect<u32>, pixels: &[Color]) {
    assert_eq!(pixels.len(), rect.width as usize * rect.height as usize);
    unsafe {
        let pixels = std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * 4);
        texture.update_from_pixels(pixels, rect.width, rect.height, rect.x, rect.y);
    }
}

#[inline(always)]
pub fn shaders_are_available() -> bool {
    Shader::is_available()
}

#[inline(always)]
pub fn geom_shaders_are_available() -> bool {
    Shader::is_geometry_available()
}

#[inline(always)]
pub fn set_uniform_float(shader: &mut Shader, name: &str, val: f32) {
    shader.set_uniform_float(name, val);
}

#[inline(always)]
pub fn set_uniform_vec2(shader: &mut Shader, name: &str, val: Vec2f) {
    shader.set_uniform_vec2(name, val.into());
}

#[inline(always)]
pub fn set_uniform_color(shader: &mut Shader, name: &str, val: Color) {
    shader.set_uniform_vec3(name, col2v3(val));
}

#[inline(always)]
pub fn set_uniform_texture(shader: &mut Shader, name: &str, val: &Texture) {
    unsafe {
        set_uniform_texture_workaround(shader, name, val);
    }
}

// !!! @Hack !!! to make set_uniform_texture work until https://github.com/jeremyletang/rust-sfml/issues/213 is solved
#[allow(unused_unsafe)]
unsafe fn set_uniform_texture_workaround(shader: &mut Shader, name: &str, texture: &Texture) {
    let tex = unsafe { std::mem::transmute::<&Texture, *const Texture<'static>>(texture) };
    shader.set_uniform_texture(name, unsafe { &*tex });
}

fn col2v3(color: Color) -> glsl::Vec3 {
    let c = glsl::Vec4::from(sfml::graphics::Color::from(color));
    glsl::Vec3::new(c.x, c.y, c.z)
}

#[inline(always)]
pub fn set_texture_repeated(texture: &mut Texture, repeated: bool) {
    texture.set_repeated(repeated);
}
