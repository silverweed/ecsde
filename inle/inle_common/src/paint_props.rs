use crate::colors::{self, Color};

#[derive(Copy, Clone)]
pub struct Paint_Properties {
    pub color: Color,
    pub border_thick: f32,
    pub border_color: Color,
    pub point_count: u32, // used for drawing circles
}

impl Default for Paint_Properties {
    fn default() -> Self {
        Paint_Properties {
            color: colors::WHITE,
            border_thick: 0.,
            border_color: colors::BLACK,
            point_count: 20,
        }
    }
}

impl From<Color> for Paint_Properties {
    fn from(color: Color) -> Self {
        Paint_Properties {
            color,
            ..Default::default()
        }
    }
}
