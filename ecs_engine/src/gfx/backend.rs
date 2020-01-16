#[cfg(feature = "use-sfml")]
mod sfml;

use super::render::Paint_Properties;
use crate::core::common::colors::Color;
use crate::core::common::rect::Rect;
use crate::core::common::shapes::Circle;
use crate::core::common::transform::Transform2D;
use crate::core::common::vector::Vec2f;

// -----------------------------------------------------------------------------
// ----------------------------- Backend: SFML ---------------------------------
// -----------------------------------------------------------------------------
#[cfg(feature = "use-sfml")]
pub type Create_Render_Window_Args = sfml::window::Create_Render_Window_Args;
#[cfg(feature = "use-sfml")]
pub type Window_Handle = sfml::window::Window_Handle;
#[cfg(feature = "use-sfml")]
pub type Blend_Mode = sfml::render::Blend_Mode;
#[cfg(feature = "use-sfml")]
pub type Texture<'a> = sfml::render::Texture<'a>;
#[cfg(feature = "use-sfml")]
pub type Sprite<'a> = sfml::render::Sprite<'a>;
#[cfg(feature = "use-sfml")]
pub type Text<'a> = sfml::render::Text<'a>;
#[cfg(feature = "use-sfml")]
pub type Font<'a> = sfml::render::Font<'a>;
#[cfg(feature = "use-sfml")]
pub type Vertex_Buffer = sfml::render::Vertex_Buffer;
#[cfg(feature = "use-sfml")]
pub type Vertex = sfml::render::Vertex;

#[cfg(feature = "use-sfml")]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_render_window(
    create_args: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    sfml::window::create_render_window(create_args, target_size, title)
}

#[cfg(feature = "use-sfml")]
pub fn destroy_render_window(window: &mut Window_Handle) {
    sfml::window::destroy_render_window(window);
}

#[cfg(feature = "use-sfml")]
pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    sfml::window::set_clear_color(window, color);
}

#[cfg(feature = "use-sfml")]
pub fn get_framerate_limit(window: &Window_Handle) -> u32 {
    sfml::window::get_framerate_limit(window)
}

#[cfg(feature = "use-sfml")]
pub fn set_framerate_limit(window: &mut Window_Handle, limit: u32) {
    sfml::window::set_framerate_limit(window, limit);
}

#[cfg(feature = "use-sfml")]
pub fn has_vsync(window: &Window_Handle) -> bool {
    sfml::window::has_vsync(window)
}

#[cfg(feature = "use-sfml")]
pub fn set_vsync(window: &mut Window_Handle, vsync: bool) {
    sfml::window::set_vsync(window, vsync);
}

#[cfg(feature = "use-sfml")]
pub fn clear(window: &mut Window_Handle) {
    sfml::window::clear(window);
}

#[cfg(feature = "use-sfml")]
pub fn display(window: &mut Window_Handle) {
    sfml::window::display(window);
}
#[cfg(feature = "use-sfml")]
pub fn resize_keep_ratio(window: &mut Window_Handle, new_width: u32, new_height: u32) {
    sfml::window::resize_keep_ratio(window, new_width, new_height);
}

#[cfg(feature = "use-sfml")]
pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    sfml::window::get_window_target_size(window)
}

#[cfg(feature = "use-sfml")]
pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect<i32>) -> Sprite<'a> {
    sfml::render::create_sprite(texture, rect)
}

#[cfg(feature = "use-sfml")]
pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &mut Sprite,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    sfml::render::render_sprite(window, sprite, transform, camera);
}

#[cfg(feature = "use-sfml")]
pub fn render_texture(window: &mut Window_Handle, texture: &Texture, rect: Rect<i32>) {
    sfml::render::render_texture(window, texture, rect);
}

#[cfg(feature = "use-sfml")]
pub fn render_text(window: &mut Window_Handle, text: &mut Text, screen_pos: Vec2f) {
    sfml::render::render_text(window, text, screen_pos);
}

#[cfg(feature = "use-sfml")]
pub fn render_text_ws(
    window: &mut Window_Handle,
    text: &Text,
    world_transform: &Transform2D,
    camera: &Transform2D,
) {
    sfml::render::render_text_ws(window, text, world_transform, camera);
}

#[cfg(feature = "use-sfml")]
pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    sfml::render::get_blend_mode(window)
}

#[cfg(feature = "use-sfml")]
pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    sfml::render::set_blend_mode(window, blend_mode);
}

#[cfg(feature = "use-sfml")]
pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    sfml::render::get_texture_size(texture)
}

#[cfg(feature = "use-sfml")]
pub fn fill_color_rect<T>(window: &mut Window_Handle, paint_props: &Paint_Properties, rect: T)
where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    sfml::render::fill_color_rect(window, paint_props, rect);
}

#[cfg(feature = "use-sfml")]
pub fn fill_color_rect_ws<T>(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    sfml::render::fill_color_rect_ws(window, paint_props, rect, transform, camera);
}

#[cfg(feature = "use-sfml")]
pub fn fill_color_circle_ws(
    window: &mut Window_Handle,
    paint_props: &Paint_Properties,
    circle: Circle,
    camera: &Transform2D,
) {
    sfml::render::fill_color_circle_ws(window, paint_props, circle, camera);
}

#[cfg(feature = "use-sfml")]
pub fn start_draw_quads(n_quads: usize) -> Vertex_Buffer {
    sfml::render::start_draw_quads(n_quads)
}

#[cfg(feature = "use-sfml")]
pub fn add_quad(vbuf: &mut Vertex_Buffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, v4: &Vertex) {
    sfml::render::add_quad(vbuf, v1, v2, v3, v4);
}

#[cfg(feature = "use-sfml")]
pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    sfml::render::new_vertex(pos, col, tex_coords)
}

#[cfg(feature = "use-sfml")]
pub fn render_vbuf_ws(
    window: &mut Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    sfml::render::render_vbuf_ws(window, vbuf, transform, camera);
}
