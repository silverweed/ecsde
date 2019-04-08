#[derive(Copy, Clone, Default, Debug)]
pub struct Norm_Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Norm_Color {
    const NORM_MUL: f32 = 0.00392156862;

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Norm_Color {
        Norm_Color {
            r: Self::NORM_MUL * r as f32,
            g: Self::NORM_MUL * g as f32,
            b: Self::NORM_MUL * b as f32,
            a: Self::NORM_MUL * a as f32,
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Norm_Color {
        Norm_Color {
            r: Self::NORM_MUL * r as f32,
            g: Self::NORM_MUL * g as f32,
            b: Self::NORM_MUL * b as f32,
            a: 1f32,
        }
    }
}

impl Norm_Color {
    #[inline(always)]
    pub fn white() -> Norm_Color {
        Norm_Color::rgb(255, 255, 255)
    }

    #[inline(always)]
    pub fn black() -> Norm_Color {
        Norm_Color::rgb(0, 0, 0)
    }
}
