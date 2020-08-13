use inle_common::fixed_string::Fixed_String_64;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Default)]
pub struct C_Debug_Data {
    pub entity_name: Fixed_String_64,
    pub prev_positions: [Vec2f; 10],
    pub n_prev_positions_filled: u8,
}
