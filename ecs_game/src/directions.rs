use std::ops::*;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Square_Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Square_Directions(u8);

impl Square_Directions {
    pub const fn all() -> Self {
        Self(0b00001111)
    }

    pub const fn has(self, dir: Square_Direction) -> bool {
        (self.0 & (1 << (dir as u8))) != 0
    }

    pub const fn without(self, dir: Square_Direction) -> Self {
        Self(self.0 & !(1 << (dir as u8)))
    }
}

impl std::fmt::Debug for Square_Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for dir in square_directions() {
            if self.has(dir) {
                write!(f, "{:?},", dir)?;
            }
        }
        write!(f, "]")
    }
}

impl BitOrAssign<Square_Direction> for Square_Directions {
    fn bitor_assign(&mut self, dir: Square_Direction) {
        self.0 |= 1 << (dir as u8);
    }
}

impl BitOr for Square_Directions {
    type Output = Self;

    fn bitor(self, dirs: Self) -> Self::Output {
        Self(self.0 | dirs.0)
    }
}

impl BitAndAssign<Square_Direction> for Square_Directions {
    fn bitand_assign(&mut self, dir: Square_Direction) {
        self.0 &= 1 << (dir as u8);
    }
}

impl BitAnd for Square_Directions {
    type Output = Self;

    fn bitand(self, dirs: Self) -> Self::Output {
        Self(self.0 & dirs.0)
    }
}

pub const fn square_directions() -> [Square_Direction; 4] {
    use Square_Direction::*;

    [Up, Right, Down, Left]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmask_manual() {
        let mut bitmask = Square_Directions::default();
        bitmask |= Square_Direction::Down;
        bitmask |= Square_Direction::Left;
        assert!(bitmask.has(Square_Direction::Left));
        assert!(bitmask.has(Square_Direction::Down));
        assert!(!bitmask.has(Square_Direction::Up));
        bitmask = bitmask.without(Square_Direction::Down);
        assert!(!bitmask.has(Square_Direction::Down));
    }

    #[test]
    fn bitmask_for() {
        let mut bitmask = Square_Directions::default();
        for dir in square_directions() {
            bitmask |= dir;
        }
        assert_eq!(bitmask, Square_Directions::all());
    }
}
