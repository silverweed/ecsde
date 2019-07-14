use cgmath::Vector2;

pub type Vec2u = Vector2<u32>;
pub type Vec2f = Vector2<f32>;
pub type Vec2i = Vector2<i32>;

// @Convenience: this is ugly to use! Think of a better solution
#[cfg(feature = "use-sfml")]
pub fn to_framework_vec(v: Vec2f) -> sfml::system::Vector2f {
    sfml::system::Vector2f::new(v.x, v.y)
}

#[cfg(feature = "use-sfml")]
pub fn from_framework_vec(v: sfml::system::Vector2f) -> Vec2f {
    Vec2f::new(v.x, v.y)
}
