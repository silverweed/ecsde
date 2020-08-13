use super::{Vec2f, Vec2i};

impl From<Vec2f> for sfml::system::Vector2f {
    fn from(v: Vec2f) -> sfml::system::Vector2f {
        sfml::system::Vector2f::new(v.x, v.y)
    }
}

impl From<sfml::system::Vector2f> for Vec2f {
    fn from(v: sfml::system::Vector2f) -> Vec2f {
        Vec2f::new(v.x, v.y)
    }
}

impl From<Vec2i> for sfml::system::Vector2i {
    fn from(v: Vec2i) -> sfml::system::Vector2i {
        sfml::system::Vector2i::new(v.x, v.y)
    }
}

impl From<sfml::system::Vector2i> for Vec2i {
    fn from(v: sfml::system::Vector2i) -> Vec2i {
        Vec2i::new(v.x, v.y)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Vec2f;

    #[test]
    fn to_from_framework() {
        let a = Vec2f::new(3., 2.);
        let b: ::sfml::system::Vector2f = a.into();
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);

        let c: Vec2f = b.into();
        assert_eq!(c.x, b.x);
        assert_eq!(c.y, b.y);
    }
}
