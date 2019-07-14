use crate::core::common::rect::Rect;
use crate::core::common::vector::Vec2f;
use crate::ecs::components::transform::C_Transform2D;
use crate::gfx::window::Window_Handle;
use cgmath::Rad;
use sfml::graphics::{RectangleShape, RenderStates, RenderTarget, Transform, Transformable};
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
    //let origin = vector::from_framework_vec(sprite.origin());
    let mut render_transform = camera.get_matrix_sfml().inverse();
    render_transform.combine(&mut transform.get_matrix_sfml());

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

fn calc_render_transform(
    transform: &C_Transform2D,
    camera: &C_Transform2D,
    rot_origin: Vec2f,
    scale_origin: Vec2f,
) -> Transform {
    let epsilon = 0.0001;

    let spos = transform.position();
    let cpos = camera.position();
    let pos = spos - cpos;

    // Apply rotation
    let Rad(srot) = transform.rotation().into();
    let Rad(crot) = camera.rotation().into();
    let rot = srot - crot;
    let rel_rot_origin = rot_origin;
    println!(
        "rot_origin = {:?}, spos = {:?}, rel_rot_origin = {:?}",
        rot_origin, spos, rel_rot_origin
    );
    //if rot > epsilon {
    let cos = rot.cos();
    let sin = rot.sin();
    let mut translation = Transform::new(
        1.0,
        0.0,
        pos.x + rot_origin.x,
        0.0,
        1.0,
        pos.y + rot_origin.y,
        0.0,
        0.0,
        1.0,
    );
    println!("translation = {:?}", translation);
    let mut rotation = Transform::new(cos, sin, 0.0, -sin, cos, 0.0, 0.0, 0.0, 1.0);
    rotation.combine(&mut translation.inverse());
    println!("rotation = {:?}", rotation);
    translation.combine(&mut rotation);
    let mut t = translation;
    //}
    println!("t = {:?}", t);

    let sscale = transform.scale();
    let cscale = camera.scale();
    let scale = Vec2f::new(sscale.x / cscale.x, sscale.y / cscale.y);
    if (scale.x - 1.0).abs() > epsilon || (scale.y - 1.0).abs() > epsilon {
        let mut scale_translation = Transform::new(
            1.0,
            0.0,
            pos.x + scale_origin.x,
            0.0,
            1.0,
            pos.y + scale_origin.y,
            0.0,
            0.0,
            1.0,
        );
        let mut scale_mat = Transform::new(scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 1.0);
        scale_mat.combine(&mut scale_translation.inverse());
        scale_translation.combine(&mut scale_mat);
        t.combine(&mut scale_translation);
    }

    t
}
