use ecs_engine::common::vector::Vec2f;

#[derive(Copy, Clone, Default)]
pub struct C_Debug_Data {
    pub prev_positions: [Vec2f; 10],
    pub n_prev_positions_filled: u8,
}
