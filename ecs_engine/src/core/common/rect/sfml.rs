use super::Rect;
use std::ops::{Add, Sub};

impl<T> std::convert::From<Rect<T>> for sfml::graphics::Rect<T>
where
    T: Copy,
{
    fn from(r: Rect<T>) -> Self {
        Self::new(r.x, r.y, r.width, r.height)
    }
}

impl<T> std::convert::AsRef<Rect<T>> for sfml::graphics::Rect<T>
where
    T: Copy,
{
    fn as_ref(&self) -> &Rect<T> {
        unsafe { &*(&self as *const _ as *const Rect<T>) }
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

impl<T> std::convert::AsRef<sfml::graphics::Rect<T>> for Rect<T>
where
    T: Copy,
{
    fn as_ref(&self) -> &sfml::graphics::Rect<T> {
        unsafe { &*(&self as *const _ as *const sfml::graphics::Rect<T>) }
    }
}


pub fn rects_intersection<T>(a: &Rect<T>, b: &Rect<T>) -> Option<Rect<T>>
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    a.as_ref().intersection(b.as_ref()).map(Into::into)
}
