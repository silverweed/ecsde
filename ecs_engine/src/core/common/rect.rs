use std::ops::{Add, Sub};

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

#[cfg(feature = "use-sfml")]
pub type Rect<T> = backend::Rect<T>;

pub type Rectf = Rect<f32>;
pub type Recti = Rect<i32>;
pub type Rectu = Rect<u32>;

impl From<Rect<i32>> for Rect<f32> {
    fn from(r: Rect<i32>) -> Self {
        Rect::new(
            r.x() as f32,
            r.y() as f32,
            r.width() as f32,
            r.height() as f32,
        )
    }
}

pub fn rects_intersect<T>(a: &Rect<T>, b: &Rect<T>) -> bool
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    backend::rects_intersect(a, b)
}
