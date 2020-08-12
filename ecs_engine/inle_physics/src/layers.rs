pub const MAX_LAYERS: u8 = 64;

// Note: collision layers are game-specific, so they're not defined in this crate.
pub type Collision_Layer = u8;

// A symmetrix 64x64 boolean matrix telling us if layer A collides with layer B.
pub struct Collision_Matrix {
    rows: [u64; MAX_LAYERS as usize],
}

impl Default for Collision_Matrix {
    fn default() -> Self {
        Self {
            rows: [0; MAX_LAYERS as usize],
        }
    }
}

impl Collision_Matrix {
    #[allow(unused_parens)]
    pub fn layers_collide(&self, a: Collision_Layer, b: Collision_Layer) -> bool {
        debug_assert!(a < MAX_LAYERS);
        debug_assert!(b < MAX_LAYERS);

        (self.rows[a as usize] & (1 << b as usize)) == 1
    }

    #[allow(unused_parens)]
    pub fn set_layers_collide(&mut self, a: Collision_Layer, b: Collision_Layer) {
        debug_assert!(a < MAX_LAYERS);
        debug_assert!(b < MAX_LAYERS);

        self.rows[a as usize] |= (1 << b as usize);
        self.rows[b as usize] |= (1 << a as usize);
    }
}
