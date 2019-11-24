// @Refactoring: probably this should be hidden in the gfx backend

#[cfg(feature = "use-sfml")]
pub type Color = sfml::graphics::Color;

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
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::rgba(r, g, b, a)
}

#[cfg(feature = "use-sfml")]
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::rgb(r, g, b)
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
}
