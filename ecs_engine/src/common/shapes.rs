use super::vector::Vec2f;

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub radius: f32,
    pub center: Vec2f,
}

#[derive(Copy, Clone, Debug)]
pub struct Arrow {
    pub center: Vec2f,
    pub direction: Vec2f, // also includes magnitude
    pub thickness: f32,
    pub arrow_size: f32,
}

#[derive(Clone)]
pub struct Line {
    pub from: Vec2f,
    pub to: Vec2f,
    pub thickness: f32,
}

impl Circle {
    pub fn intersects(&self, other: &Circle) -> bool {
        let cdist2 = self.center.distance2(other.center);
        let rdist2 = (self.radius + other.radius).powf(2.);
        cdist2 < rdist2
    }

    /// Returns the amount by which this circle is inside the other.
    /// Returns a negative number (the opposite of the 'surface distance')
    /// if the two circles are not overlapping.
    pub fn penetration_distance(&self, other: &Circle) -> f32 {
        let cdist = self.center.distance(other.center);
        let rdist = self.radius + other.radius;
        rdist - cdist
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

    #[test]
    fn circle_penetration_distance() {
        let a = Circle {
            center: Vec2f::new(0., 0.),
            radius: 1.,
        };
        let b = Circle {
            center: Vec2f::new(0.5, 0.),
            radius: 1.,
        };
        assert_approx_eq!(a.penetration_distance(&b), 1.5);
        assert_approx_eq!(b.penetration_distance(&a), 1.5);

        let b = Circle {
            center: Vec2f::new(2., 2.),
            radius: 1.,
        };
        let expected = 2. - 2. * 2_f32.sqrt();
        assert_approx_eq!(a.penetration_distance(&b), expected);
        assert_approx_eq!(b.penetration_distance(&a), expected);
    }
}
