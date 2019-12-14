// @Refactoring: probably this should be hidden in the gfx backend

#[cfg(feature = "use-sfml")]
pub type Color = sfml::graphics::Color;

pub const RED: Color = rgb(255, 0, 0);
pub const GREEN: Color = rgb(0, 255, 0);
pub const BLUE: Color = rgb(0, 0, 255);
pub const WHITE: Color = rgb(255, 255, 255);
pub const BLACK: Color = rgb(0, 0, 0);

pub fn color_to_hex(c: Color) -> u32 {
    let mut h = 0u32;
    h |= u32::from(c.a);
    h |= u32::from(c.b) << 8;
    h |= u32::from(c.g) << 16;
    h |= u32::from(c.r) << 24;
    h
}

pub fn color_from_hex(hex: u32) -> Color {
    let a = (hex & 0x00_00_00_FF) as u8;
    let b = ((hex & 0x00_00_FF_00) >> 8) as u8;
    let g = ((hex & 0x00_FF_00_00) >> 16) as u8;
    let r = ((hex & 0xFF_00_00_00) >> 24) as u8;
    Color::rgba(r, g, b, a)
}

#[cfg(feature = "use-sfml")]
#[inline]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

#[cfg(feature = "use-sfml")]
#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

#[inline]
pub fn lerp_col(a: Color, b: Color, t: f32) -> Color {
    let omt = 1. - t;
    rgba(
        (a.r as f32 * omt + b.r as f32 * t) as u8,
        (a.g as f32 * omt + b.g as f32 * t) as u8,
        (a.b as f32 * omt + b.b as f32 * t) as u8,
        (a.a as f32 * omt + b.a as f32 * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hex() {
        let c = rgba(0, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0);

        let c = rgba(255, 255, 255, 255);
        assert_eq!(color_to_hex(c), 0xFFFFFFFF);

        let c = rgba(255, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0xFF000000);

        let c = rgba(0, 255, 0, 0);
        assert_eq!(color_to_hex(c), 0x00FF0000);

        let c = rgba(0, 0, 255, 0);
        assert_eq!(color_to_hex(c), 0x0000FF00);

        let c = rgba(0, 0, 0, 255);
        assert_eq!(color_to_hex(c), 0x000000FF);

        let c = rgb(171, 205, 239);
        assert_eq!(color_to_hex(c), 0xABCDEFFF);
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(color_from_hex(0x0), rgba(0, 0, 0, 0));
        assert_eq!(color_from_hex(0xFFFFFFFF), rgba(255, 255, 255, 255));
        assert_eq!(color_from_hex(0xFF000000), rgba(255, 0, 0, 0));
        assert_eq!(color_from_hex(0x00FF0000), rgba(0, 255, 0, 0));
        assert_eq!(color_from_hex(0x0000FF00), rgba(0, 0, 255, 0));
        assert_eq!(color_from_hex(0x000000FF), rgba(0, 0, 0, 255));
        assert_eq!(color_from_hex(0xABCDEFFF), rgb(171, 205, 239));
    }

    #[test]
    fn test_lerp_colors() {
        let a = rgba(0, 0, 0, 0);
        let b = rgba(255, 255, 255, 255);
        assert_eq!(lerp_col(a, b, 0.), a);
        assert_eq!(lerp_col(a, b, 1.), b);
        assert_eq!(lerp_col(a, b, 0.5), rgba(127, 127, 127, 127));

        let c = rgb(10, 100, 150);
        let d = rgb(20, 200, 250);
        assert_eq!(lerp_col(c, d, 0.75), rgb(17, 175, 225));
    }
}
