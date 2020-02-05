use super::Rect;
use std::ops::{Add, Sub};

impl<T> std::convert::From<&Rect<T>> for sfml::graphics::Rect<T>
where
    T: Copy,
{
    fn from(r: &Rect<T>) -> Self {
        Self::new(r.x, r.y, r.width, r.height)
    }
}

impl<T> std::convert::From<sfml::graphics::Rect<T>> for Rect<T>
where
    T: Copy,
{
    fn from(r: sfml::graphics::Rect<T>) -> Self {
        Self::new(r.left, r.top, r.width, r.height)
    }
}

pub fn rects_intersect<T>(a: &Rect<T>, b: &Rect<T>) -> bool
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    let a: sfml::graphics::Rect<T> = a.into();
    a.intersection(&b.into()) != None
}
