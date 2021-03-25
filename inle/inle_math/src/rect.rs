use crate::vector::Vector2;
use std::cmp::{Eq, Ordering, PartialEq};
use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

#[cfg(feature = "gfx-sfml")]
mod sfml;

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

impl<T: Default> Default for Rect<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            width: T::default(),
            height: T::default(),
        }
    }
}

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

impl<T> Rect<T>
where
    T: Sub<Output = T> + PartialOrd + Copy,
{
    // @WaitForStable: mark this const
    pub fn from_topleft_botright(topleft: Vector2<T>, botright: Vector2<T>) -> Rect<T> {
        debug_assert!(topleft.x <= botright.x);
        debug_assert!(topleft.y <= botright.y);
        Rect {
            x: topleft.x,
            y: topleft.y,
            width: botright.x - topleft.x,
            height: botright.y - topleft.y,
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

impl From<Rect<f32>> for Rect<i32> {
    fn from(r: Rect<f32>) -> Self {
        Rect::new(r.x as i32, r.y as i32, r.width as i32, r.height as i32)
    }
}

impl From<Rect<u32>> for Rect<f32> {
    fn from(r: Rect<u32>) -> Self {
        Rect::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
    }
}

impl Mul<f32> for Rect<f32> {
    type Output = Self;

    fn mul(self, x: f32) -> Self {
        Rect::new(self.x * x, self.y * x, self.width * x, self.height * x)
    }
}

fn min<T: PartialOrd + Copy>(a: T, b: T) -> T {
    match a.partial_cmp(&b) {
        Some(Ordering::Greater) => b,
        _ => a,
    }
}

fn max<T: PartialOrd + Copy>(a: T, b: T) -> T {
    match a.partial_cmp(&b) {
        Some(Ordering::Less) => b,
        _ => a,
    }
}

// Translated from SFML/Graphics/Rect.inl
pub fn rects_intersection<T>(a: &Rect<T>, b: &Rect<T>) -> Option<Rect<T>>
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    // Rectangles with negative dimensions are allowed, so we must handle them correctly

    // Compute the min and max of the first rectangle on both axes
    let r1_minx = min(a.x, a.x + a.width);
    let r1_maxx = max(a.x, a.x + a.width);
    let r1_miny = min(a.y, a.y + a.height);
    let r1_maxy = max(a.y, a.y + a.height);

    // Compute the min and max of the second rectangle on both axes
    let r2_minx = min(b.x, b.x + b.width);
    let r2_maxx = max(b.x, b.x + b.width);
    let r2_miny = min(b.y, b.y + b.height);
    let r2_maxy = max(b.y, b.y + b.height);

    // Compute the intersection boundaries
    let ileft = max(r1_minx, r2_minx);
    let itop = max(r1_miny, r2_miny);
    let iright = min(r1_maxx, r2_maxx);
    let ibot = min(r1_maxy, r2_maxy);

    // If the intersection is valid (positive non zero area), then there is an intersection
    if (ileft < iright) && (itop < ibot) {
        Some(Rect::new(ileft, itop, iright - ileft, ibot - itop))
    } else {
        None
    }
}

impl<T> Rect<T>
where
    T: Add<Output = T> + Copy,
{
    // @WaitForStable: make this const
    pub fn contains<V>(&self, pos: V) -> bool
    where
        T: PartialOrd,
        V: Into<Vector2<T>>,
    {
        let pos: Vector2<T> = pos.into();
        pos.x >= self.x
            && pos.x <= self.x + self.width
            && pos.y >= self.y
            && pos.y <= self.y + self.height
    }

    pub fn pos_min(&self) -> Vector2<T> {
        v2!(self.x, self.y)
    }

    pub fn pos_max(&self) -> Vector2<T> {
        v2!(self.x + self.width, self.y + self.height)
    }
}
