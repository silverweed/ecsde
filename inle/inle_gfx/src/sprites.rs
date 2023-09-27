use inle_resources::gfx::{Gfx_Resources,Texture_Handle};
use std::path::Path;
use super::material::Material;
use inle_math::rect::{Rect, Recti};
use inle_math::transform::Transform2D;
use super::render;
use inle_common::colors;

pub struct Sprite {
    pub material: Material,
    pub rect: Recti,
    pub transform: Transform2D,
    pub z_index: render::Z_Index,
    pub color: colors::Color,
}

impl Sprite {
    pub fn new(gres: &Gfx_Resources, tex: Texture_Handle) -> Self {
        let texture = gres.get_texture(tex);
        let (sw, sh) = render::get_texture_size(texture);
        let rect = Rect::new(0, 0, sw as i32, sh as i32);
        let material = Material::with_texture(tex);
        Self {
            material,
            rect,
            transform: Transform2D::default(),
            z_index: 0,
            color: colors::WHITE,
        }
    }

    pub fn from_tex_path(gres: &mut Gfx_Resources, tex_path: &Path) -> Self {
        let tex = gres.load_texture(tex_path);
        Self::new(gres, tex)
    }
}

pub fn render_sprite(
    window: &mut super::render_window::Render_Window_Handle,
    batches: &mut render::batcher::Batches,
    sprite: &Sprite,
) {
    render::render_texture_ws(
        window, batches, &sprite.material, &sprite.rect, sprite.color, &sprite.transform, sprite.z_index,
    );
}

