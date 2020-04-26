#[cfg(feature = "use-sfml")]
mod sfml;

use std::f32::consts::{FRAC_PI_3, PI};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const TRANSPARENT: Color = rgba(0, 0, 0, 0);
pub const RED: Color = rgb(255, 0, 0);
pub const GREEN: Color = rgb(0, 255, 0);
pub const BLUE: Color = rgb(0, 0, 255);
pub const YELLOW: Color = rgb(255, 255, 0);
pub const FUCHSIA: Color = rgb(255, 0, 255);
pub const AQUA: Color = rgb(0, 255, 255);
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
    rgba(r, g, b, a)
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

pub fn lerp_col(a: Color, b: Color, t: f32) -> Color {
    let omt = 1. - t;
    rgba(
        (f32::from(a.r) * omt + f32::from(b.r) * t) as u8,
        (f32::from(a.g) * omt + f32::from(b.g) * t) as u8,
        (f32::from(a.b) * omt + f32::from(b.b) * t) as u8,
        (f32::from(a.a) * omt + f32::from(b.a) * t) as u8,
    )
}

pub fn to_hsv(c: Color) -> (f32, f32, f32) {
    let r = c.r as f32 / 255.0;
    let g = c.g as f32 / 255.0;
    let b = c.b as f32 / 255.0;
    let cmax = [r, g, b]
        .iter()
        .copied()
        .fold(-std::f32::INFINITY, f32::max);
    let cmin = [r, g, b].iter().copied().fold(std::f32::INFINITY, f32::min);
    let delta = cmax - cmin;
    let h = if delta == 0. {
        0.
    } else if r == cmax {
        FRAC_PI_3 * (((g - b) / delta) % 6.)
    } else if g == cmax {
        FRAC_PI_3 * ((b - r) / delta + 2.)
    } else {
        FRAC_PI_3 * ((r - g) / delta + 4.)
    };
    let s = if cmax == 0. { 0. } else { delta / cmax };
    let v = cmax;

    (h, s, v)
}

pub fn from_hsv((h, s, v): (f32, f32, f32)) -> Color {
    let c = v * s;
    let x = c * (1. - (h / FRAC_PI_3 % 2. - 1.).abs());
    let m = v - c;
    let (r, g, b) = if h < FRAC_PI_3 {
        (c, x, 0.)
    } else if h < 2. * FRAC_PI_3 {
        (x, c, 0.)
    } else if h < PI {
        (0., c, x)
    } else if h < 4. * FRAC_PI_3 {
        (0., x, c)
    } else if h < 5. * FRAC_PI_3 {
        (x, 0., c)
    } else {
        (c, 0., x)
    };

    Color {
        r: ((r + m) * 255.) as u8,
        g: ((g + m) * 255.) as u8,
        b: ((b + m) * 255.) as u8,
        a: 255,
    }
}

/// Heuristic "darken" function that multiplies the value of `c` by `1 - amount`.
pub fn darken(c: Color, amount: f32) -> Color {
    let (h, s, v) = to_hsv(c);
    from_hsv((h, s, v * (1. - amount)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::*;

    impl Approx_Eq_Testable for (f32, f32, f32) {
        fn cmp_list(&self) -> Vec<f32> {
            vec![self.0, self.1, self.2]
        }
    }

    #[test]
    fn test_color_to_hex() {
        let c = rgba(0, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0);

        let c = rgba(255, 255, 255, 255);
        assert_eq!(color_to_hex(c), 0xFFFF_FFFF);

        let c = rgba(255, 0, 0, 0);
        assert_eq!(color_to_hex(c), 0xFF00_0000);

        let c = rgba(0, 255, 0, 0);
        assert_eq!(color_to_hex(c), 0x00FF_0000);

        let c = rgba(0, 0, 255, 0);
        assert_eq!(color_to_hex(c), 0x0000_FF00);

        let c = rgba(0, 0, 0, 255);
        assert_eq!(color_to_hex(c), 0x0000_00FF);

        let c = rgb(171, 205, 239);
        assert_eq!(color_to_hex(c), 0xABCD_EFFF);
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(color_from_hex(0x0), rgba(0, 0, 0, 0));
        assert_eq!(color_from_hex(0xFFFF_FFFF), rgba(255, 255, 255, 255));
        assert_eq!(color_from_hex(0xFF00_0000), rgba(255, 0, 0, 0));
        assert_eq!(color_from_hex(0x00FF_0000), rgba(0, 255, 0, 0));
        assert_eq!(color_from_hex(0x0000_FF00), rgba(0, 0, 255, 0));
        assert_eq!(color_from_hex(0x0000_00FF), rgba(0, 0, 0, 255));
        assert_eq!(color_from_hex(0xABCD_EFFF), rgb(171, 205, 239));
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

    #[test]
    fn test_to_hsv() {
        assert_approx_eq!(to_hsv(BLACK), (0., 0., 0.));
        assert_approx_eq!(to_hsv(WHITE), (0., 0., 1.));
        assert_approx_eq!(to_hsv(RED), (0., 1., 1.));
        assert_approx_eq!(to_hsv(GREEN), (2. * FRAC_PI_3, 1., 1.));
        assert_approx_eq!(to_hsv(BLUE), (4. * FRAC_PI_3, 1., 1.));
        assert_approx_eq!(to_hsv(YELLOW), (FRAC_PI_3, 1., 1.));
        assert_approx_eq!(to_hsv(rgb(128, 0, 0)), (0., 1., 0.5), eps = 0.1);
        assert_approx_eq!(to_hsv(rgb(191, 191, 191)), (0., 0., 0.75), eps = 0.1);
        assert_approx_eq!(to_hsv(rgb(0, 128, 128)), (PI, 1., 0.5), eps = 0.1);

        assert_eq!(from_hsv(to_hsv(BLACK)), BLACK);
        assert_eq!(from_hsv(to_hsv(WHITE)), WHITE);
        assert_eq!(from_hsv(to_hsv(RED)), RED);
        assert_eq!(from_hsv(to_hsv(GREEN)), GREEN);
        assert_eq!(from_hsv(to_hsv(BLUE)), BLUE);
    }
}
