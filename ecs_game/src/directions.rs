#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Square_Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

pub const fn square_directions() -> [Square_Direction; 4] {
    use Square_Direction::*;

    [Up, Right, Down, Left]
}
