impl std::convert::From<super::Color> for sfml::graphics::Color {
    fn from(c: super::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl std::convert::From<sfml::graphics::Color> for super::Color {
    fn from(c: sfml::graphics::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}
