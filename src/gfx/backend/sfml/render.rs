use crate::core::common::rect::Rect;
use crate::core::common::vector::Vec2f;
use crate::ecs::components::transform::C_Transform2D;
use crate::gfx::window::Window_Handle;
use cgmath::Deg;
use sfml::graphics::{RectangleShape, RenderStates, RenderTarget, Transformable};
use sfml::system::Vector2f;

pub type Blend_Mode = sfml::graphics::blend_mode::BlendMode;

pub struct Texture<'a> {
    texture: sfml::graphics::Texture,
    marker: &'a std::marker::PhantomData<()>,
}

impl Texture<'_> {
    pub fn from_file(fname: &str) -> Option<Self> {
        Some(Texture {
            texture: sfml::graphics::Texture::from_file(fname)?,
            marker: &std::marker::PhantomData,
        })
    }
}

impl std::ops::Deref for Texture<'_> {
    type Target = sfml::graphics::Texture;

    fn deref(&self) -> &sfml::graphics::Texture {
        &self.texture
    }
}

impl std::ops::DerefMut for Texture<'_> {
    fn deref_mut(&mut self) -> &mut sfml::graphics::Texture {
        &mut self.texture
    }
}

pub type Text<'a> = sfml::graphics::Text<'a>;

pub struct Font<'a> {
    font: sfml::graphics::Font,
    marker: &'a std::marker::PhantomData<()>,
}

impl Font<'_> {
    pub fn from_file(fname: &str) -> Option<Self> {
        Some(Font {
            font: sfml::graphics::Font::from_file(fname)?,
            marker: &std::marker::PhantomData,
        })
    }
}

impl std::ops::Deref for Font<'_> {
    type Target = sfml::graphics::Font;

    fn deref(&self) -> &sfml::graphics::Font {
        &self.font
    }
}

impl std::ops::DerefMut for Font<'_> {
    fn deref_mut(&mut self) -> &mut sfml::graphics::Font {
        &mut self.font
    }
}

pub type Sprite<'a> = sfml::graphics::Sprite<'a>;

pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect) -> Sprite<'a> {
    let mut sprite = Sprite::with_texture(texture);
    sprite.set_texture_rect(&rect);
    sprite
}

pub fn render_sprite(
    window: &mut Window_Handle,
    sprite: &Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    let render_transform = {
        let mut t = sfml::graphics::Transform::IDENTITY;
        let pos = transform.position();
        t.translate(pos.x, pos.y);
        let cpos = camera.position();
        t.translate(-cpos.x, -cpos.y);
        let Deg(angle) = transform.rotation().into();
        let Deg(cam_angle) = camera.rotation().into();
        t.rotate(angle);
        t.rotate(-cam_angle);
        let scale = transform.scale();
        t.scale(scale.x, scale.y);
        t
    };
    let render_states = RenderStates {
        transform: render_transform,
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };
    window.handle.draw_with_renderstates(sprite, render_states);
}

pub fn render_texture(window: &mut Window_Handle, texture: &Texture<'_>, rect: Rect) {
    let render_states = RenderStates {
        blend_mode: get_blend_mode(window),
        ..Default::default()
    };
    let mut rectangle_shape = RectangleShape::with_texture(texture);
    rectangle_shape.set_position(Vector2f::new(rect.x() as f32, rect.y() as f32));
    rectangle_shape.set_size(Vector2f::new(rect.width() as f32, rect.height() as f32));
    window
        .handle
        .draw_rectangle_shape(&rectangle_shape, render_states);
}

pub fn render_text(window: &mut Window_Handle, text: &Text<'_>) {
    window.handle.draw(text);
}

pub fn get_blend_mode(window: &Window_Handle) -> Blend_Mode {
    window.blend_mode
}

pub fn set_blend_mode(window: &mut Window_Handle, blend_mode: Blend_Mode) {
    window.blend_mode = blend_mode;
}

pub fn get_texture_size(texture: &sfml::graphics::Texture) -> (u32, u32) {
    let s = texture.size();
    (s.x, s.y)
}
