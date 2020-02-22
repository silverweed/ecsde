use crate::common::vector::Vector2;
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
use std::ops::{Add, Sub};

#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

#[repr(C)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T: Copy> Copy for Rect<T> {}
impl<T: Clone> Clone for Rect<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
        }
    }
}
impl<T: Debug> Debug for Rect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Rect {{ x: {:?}, y: {:?}, width: {:?}, height: {:?} }}",
            self.x, self.y, self.width, self.height
        )
    }
}
impl<T: PartialEq> PartialEq for Rect<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.width == other.width
            && self.height == other.height
    }
}
impl<T: Eq> Eq for Rect<T> {}

pub type Rectf = Rect<f32>;
pub type Recti = Rect<i32>;
pub type Rectu = Rect<u32>;

impl<T> Rect<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl<T: Copy> Rect<T> {
    #[inline]
    // @WaitForStable: mark this const
    pub fn size(&self) -> Vector2<T> {
        Vector2::new(self.width, self.height)
    }
}

impl From<Rect<i32>> for Rect<f32> {
    fn from(r: Rect<i32>) -> Self {
        Rect::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
    }
}

impl From<Rect<u32>> for Rect<f32> {
    fn from(r: Rect<u32>) -> Self {
        Rect::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
    }
}

pub fn rects_intersection<T>(a: &Rect<T>, b: &Rect<T>) -> Option<Rect<T>>
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    backend::rects_intersection(a, b)
}
