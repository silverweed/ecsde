pub type Color = sdl2::pixels::Color;

pub fn color_to_hex(c: Color) -> u32 {
    let mut h = 0u32;
    h |= u32::from(c.a);
    h |= u32::from(c.b) << 8;
    h |= u32::from(c.g) << 16;
    h |= u32::from(c.r) << 24;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hex() {
        let c = Color::RGBA(0, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0);

        let c = Color::RGBA(255, 255, 255, 255);
        assert_eq!(color_to_hex(c), 0xFFFFFFFF);

        let c = Color::RGBA(255, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0xFF000000);

        let c = Color::RGBA(0, 255, 0, 0);
        assert_eq!(color_to_hex(c), 0x00FF0000);

        let c = Color::RGBA(0, 0, 255, 0);
        assert_eq!(color_to_hex(c), 0x0000FF00);

        let c = Color::RGBA(0, 0, 0, 255);
        assert_eq!(color_to_hex(c), 0x000000FF);

        let c = Color::RGB(171, 205, 239);
        assert_eq!(color_to_hex(c), 0xABCDEFFF);
    }
}
