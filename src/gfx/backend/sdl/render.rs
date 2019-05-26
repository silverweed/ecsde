use crate::core::common::vector::Vec2f;
use crate::ecs::components::transform::C_Transform2D;
use cgmath::Deg;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

pub type Blend_Mode = sdl2::render::BlendMode;
pub type Texture<'a> = sdl2::render::Texture<'a>;

pub struct Sprite<'a> {
    pub texture: &'a Texture<'a>,
    pub rect: Rect,
}

// @Incomplete
pub struct Text {};

pub fn create_sprite<'a>(texture: &'a Texture<'a>, rect: Rect) -> Sprite<'a> {
    Sprite {
        texture: texture,
        rect: rect,
    }
}

pub fn render_sprite(
    window: &mut WindowCanvas,
    sprite: &mut Sprite<'_>,
    transform: &C_Transform2D,
    camera: &C_Transform2D,
) {
    let src_rect = sprite.rect;

    let pos = transform.position();
    let Deg(angle) = transform.rotation().into();
    let scale = transform.scale();

    let Vec2f { x: cam_x, y: cam_y } = camera.position();

    let Vec2f {
        x: cam_sx,
        y: cam_sy,
    } = camera.scale();
    window.set_scale(cam_sx, cam_sy).unwrap();

    let dst_rect = Rect::new(
        (pos.x - cam_x) as i32,
        (pos.y - cam_y) as i32,
        (scale.x * (src_rect.width() as f32)) as u32,
        (scale.y * (src_rect.height() as f32)) as u32,
    );

    if let Err(msg) = window.copy_ex(
        sprite.texture,
        Some(src_rect),
        dst_rect,
        f64::from(angle), // degrees!
        None,
        false,
        false,
    ) {
        eprintln!("Error copying texture to window: {}", msg);
    }
}

pub fn render_texture(window: &mut WindowCanvas, texture: &Texture<'_>, rect: Rect) {
    if let Err(msg) = window.copy(&texture, None, rect) {
        eprintln!("Error copying texture to window: {}", msg);
    }
}

pub fn render_text(window: &mut Window_Handle, text: &Text) {
    eprintln!("[ WARNING ] render_text is not implemented yet on SDL backend!");
}

pub fn get_blend_mode(window: &WindowCanvas) -> Blend_Mode {
    window.blend_mode()
}

pub fn set_blend_mode(window: &mut WindowCanvas, blend_mode: Blend_Mode) {
    window.set_blend_mode(blend_mode);
}

pub fn get_texture_size(texture: &sdl2::render::Texture<'_>) -> (u32, u32) {
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    (width, height)
}
