use super::cache::Texture_Handle;
use crate::alloc::generational_allocator::{Generational_Allocator, Generational_Index};
use sdl2::rect::Rect;
use std::vec::Vec;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub texture: Texture_Handle,
    pub rect: Rect,
}

impl std::default::Default for Sprite {
    fn default() -> Self {
        Sprite {
            texture: Texture_Handle::default(),
            rect: Rect::new(0, 0, 0, 0),
        }
    }
}

pub struct Sprite_Storage {
    allocator: Generational_Allocator,
    sprites: Vec<Sprite>,
}

pub type Sprite_Handle = Generational_Index;

impl Sprite_Storage {
    const INITIAL_SIZE: usize = 128;

    pub fn new() -> Self {
        let mut sprites = vec![];
        sprites.resize(Self::INITIAL_SIZE, Sprite::default());
        Sprite_Storage {
            allocator: Generational_Allocator::new(Self::INITIAL_SIZE),
            sprites,
        }
    }

    pub fn new_sprite(
        &mut self,
        texture: Texture_Handle,
        width: u32,
        height: u32,
    ) -> Sprite_Handle {
        let s = self.allocator.allocate();
        if s.index >= self.sprites.len() {
            self.sprites
                .resize(self.allocator.size(), Sprite::default());
        }
        self.sprites[s.index] = Sprite {
            texture,
            rect: Rect::new(0, 0, width, height),
        };
        s
    }

    pub fn destroy_sprite(&mut self, sprite: Sprite_Handle) {
        self.allocator.deallocate(sprite);
    }

    pub fn is_valid_sprite(&self, sprite: Sprite_Handle) -> bool {
        self.allocator.is_valid(sprite)
    }

    pub fn n_loaded_sprites(&self) -> usize {
        self.allocator.live_size()
    }

    pub fn get_sprite(&self, handle: Sprite_Handle) -> &Sprite {
        assert!(self.is_valid_sprite(handle));
        &self.sprites[handle.index]
    }
}

#[cfg(test)]
mod tests {
    use super::super::Resources;
    use super::*;
    use crate::gfx;

    #[test]
    fn test_create_destroy_sprite() {
        let mut res = create_resources();
        let mut storage = Sprite_Storage::new();

        assert_eq!(storage.n_loaded_sprites(), 0);

        let t = res.load_texture("none"); // get fallback texture
        let s = storage.new_sprite(t, 1, 1);

        assert!(storage.is_valid_sprite(s));
        assert_eq!(storage.n_loaded_sprites(), 1);

        storage.destroy_sprite(s);
        assert!(!storage.is_valid_sprite(s));
        assert_eq!(storage.n_loaded_sprites(), 0);
    }

    fn create_resources() -> Resources {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let canvas = gfx::window::create_render_canvas(&video_subsystem, (0, 0), "Test");
        let texture_creator = canvas.texture_creator();
        Resources::new(texture_creator)
    }
}
