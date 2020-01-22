use super::vector::Vec2f;

#[derive(Copy, Clone)]
pub struct Circle {
    pub radius: f32,
    pub center: Vec2f,
}

#[derive(Clone)]
pub struct Arrow {
    pub center: Vec2f,
    pub direction: Vec2f, // also includes magnitude
    pub thickness: f32,
    pub arrow_size: f32,
}

impl Circle {
    pub fn intersects(&self, other: &Circle) -> bool {
        let cdist2 = self.center.distance2(&other.center);
        let rdist2 = (self.radius + other.radius).powf(2.);
        cdist2 < rdist2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_intersect() {
        let a = Circle {
            center: Vec2f::new(0., 0.),
            radius: 1.,
        };
        assert!(a.intersects(&a));

        let b = Circle {
            center: Vec2f::new(0.5, 0.),
            radius: 1.,
        };
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));

        let b = Circle {
            center: Vec2f::new(2., 2.),
            radius: 1.,
        };
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }
}
