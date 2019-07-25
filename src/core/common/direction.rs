#[derive(PartialEq, Hash, Copy, Clone)]
pub enum Direction {
    None,
    Up,
    Right,
    Down,
    Left,
}

bitflags! {
    pub struct Direction_Flags: u8 {
        const UP    = 1 << 0;
        const RIGHT = 1 << 1;
        const DOWN  = 1 << 2;
        const LEFT  = 1 << 3;
    }
}

impl std::default::Default for Direction_Flags {
    fn default() -> Direction_Flags {
        Direction_Flags::empty()
    }
}
