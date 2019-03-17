use crate::alloc::generational_allocator::{Generational_Allocator, Generational_Index};
use sfml::graphics::{Sprite, Texture};
use std::vec::Vec;

pub struct Sprite_Storage<'a> {
    allocator: Generational_Allocator,
    sprites: Vec<Sprite<'a>>,
}

pub type Sprite_Handle = Generational_Index;

impl<'a> Sprite_Storage<'a> {
    const INITIAL_SIZE: usize = 128;

    pub fn new() -> Self {
        let mut sprites = vec![];
        sprites.resize(Self::INITIAL_SIZE, Sprite::default());
        Sprite_Storage {
            allocator: Generational_Allocator::new(Self::INITIAL_SIZE),
            sprites,
        }
    }

    pub fn new_sprite(&mut self, texture: &'a Texture) -> Sprite_Handle {
        let s = self.allocator.allocate();
        if s.index >= self.sprites.len() {
            self.sprites
                .resize(self.allocator.size(), Sprite::default());
        }
        self.sprites[s.index] = Sprite::with_texture(texture);
        s
    }

    pub fn destroy_sprite(&mut self, sprite: Sprite_Handle) {
        self.allocator.deallocate(sprite);
    }

    pub fn is_valid_sprite(&self, sprite: &Sprite_Handle) -> bool {
        self.allocator.is_valid(sprite)
    }

    pub fn n_loaded_sprites(&self) -> usize {
        self.allocator.live_size()
    }
}

#[cfg(test)]
mod tests {
    use super::super::resources::Resources;
    use super::*;

    #[test]
    fn test_create_destroy_sprite() {
        let mut res = Resources::new();
        let mut storage = Sprite_Storage::new();

        assert_eq!(storage.n_loaded_sprites(), 0);

        let t = res.load_texture("none"); // get fallback texture
        let s = storage.new_sprite(t);

        assert!(storage.is_valid_sprite(&s));
        assert_eq!(storage.n_loaded_sprites(), 1);

        storage.destroy_sprite(s);
        assert!(!storage.is_valid_sprite(&s));
        assert_eq!(storage.n_loaded_sprites(), 0);
    }
}
