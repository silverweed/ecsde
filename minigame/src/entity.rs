use crate::sprites::{self as anim_sprites, Anim_Sprite};
use inle_gfx::render::batcher::Batches;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::transform::Transform2D;
use smallvec::SmallVec;

#[derive(Default, Clone)]
pub struct Entity {
    pub transform: Transform2D,
    pub sprites: SmallVec<[Anim_Sprite; 4]>,
}

impl Entity {
    pub fn new(sprite: Anim_Sprite) -> Self {
        Self {
            transform: Transform2D::default(),
            sprites: smallvec![sprite],
        }
    }

    pub fn draw(&self, window: &mut Render_Window_Handle, batches: &mut Batches) {
        let mut sprites = self.sprites.clone();
        for sprite in &mut sprites {
            sprite.transform = self.transform.combine(&sprite.transform);
            anim_sprites::render_anim_sprite(window, batches, sprite);
        }
    }
}
