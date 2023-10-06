use crate::transform::Transform2D;
use crate::vector::{Vec2f, Vector2};
use std::cmp::{Eq, Ordering, PartialEq};
use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

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
    #[track_caller]
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

    // @WaitForStable: mark this const
    pub fn from_topleft_size(topleft: Vector2<T>, size: Vector2<T>) -> Rect<T> {
        Rect {
            x: topleft.x,
            y: topleft.y,
            width: size.x,
            height: size.y,
        }
    }
}

impl Rect<f32> {
    pub fn from_center_size(center: Vector2<f32>, size: Vector2<f32>) -> Rect<f32> {
        Rect {
            x: center.x - size.x * 0.5,
            y: center.y - size.y * 0.5,
            width: size.x,
            height: size.y,
        }
    }
}

impl<T: Copy> Rect<T> {
    // @WaitForStable: mark this const
    pub fn pos(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

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

    fn mul(self, x: f32) -> Self::Output {
        Rect::new(self.x * x, self.y * x, self.width * x, self.height * x)
    }
}

impl<T> Add<Vector2<T>> for Rect<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, offset: Vector2<T>) -> Self::Output {
        Rect::new(
            self.x + offset.x,
            self.y + offset.y,
            self.width,
            self.height,
        )
    }
}

impl<T> Sub<Vector2<T>> for Rect<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, offset: Vector2<T>) -> Self::Output {
        Rect::new(
            self.x - offset.x,
            self.y - offset.y,
            self.width,
            self.height,
        )
    }
}

impl<T> Mul<Vector2<T>> for Rect<T>
where
    T: Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, scale: Vector2<T>) -> Self::Output {
        Rect::new(
            self.x,
            self.y,
            self.width * scale.x,
            self.height * scale.y
        )
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

#[inline]
pub fn aabb_of_points<'a, T: IntoIterator<Item = &'a Vec2f>>(points: T) -> Rectf {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for &pt in points {
        min_x = pt.x.min(min_x);
        min_y = pt.y.min(min_y);
        max_x = pt.x.max(max_x);
        max_y = pt.y.max(max_y);
    }

    Rect::from_topleft_botright(v2!(min_x, min_y), v2!(max_x, max_y))
}

#[inline]
#[allow(clippy::collapsible_else_if)]
pub fn aabb_of_transformed_rect(rect: &Rectf, transform: &Transform2D) -> Rectf {
    let (s, c) = transform.rotation().as_rad().sin_cos();
    let Vec2f { x: tx, y: ty } = transform.position();
    let (min, max) = (rect.pos_min(), rect.pos_max());
    if c >= 0.0 {
        if s >= 0.0 {
            Rect::from_topleft_botright(
                v2!(min.x * c - max.y * s + tx, min.x * s + min.y * c + ty),
                v2!(max.x * c - min.y * s + tx, max.x * s + max.y * c + ty),
            )
        } else {
            Rect::from_topleft_botright(
                v2!(min.x * c - min.y * s + tx, max.x * s + min.y * c + ty),
                v2!(max.x * c - max.y * s + tx, min.x * s + max.y * c + ty),
            )
        }
    } else {
        if s >= 0.0 {
            Rect::from_topleft_botright(
                v2!(max.x * c - max.y * s + tx, min.x * s + max.y * c + ty),
                v2!(min.x * c - min.y * s + tx, max.x * s + min.y * c + ty),
            )
        } else {
            Rect::from_topleft_botright(
                v2!(max.x * c - min.y * s + tx, max.x * s + max.y * c + ty),
                v2!(min.x * c - max.y * s + tx, min.x * s + min.y * c + ty),
            )
        }
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

    #[inline]
    pub fn pos_min(&self) -> Vector2<T> {
        v2!(self.x, self.y)
    }

    #[inline]
    pub fn pos_max(&self) -> Vector2<T> {
        v2!(self.x + self.width, self.y + self.height)
    }
}

impl Rect<f32> {
    #[inline]
    pub fn pos_center(&self) -> Vector2<f32> {
        v2!(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let rect = Rect::new(0., 0., 10., 10.);
        assert!(rect.contains(v2!(5., 5.)));
        assert!(rect.contains(v2!(0., 0.)));
        assert!(!rect.contains(v2!(15., 5.)));
        assert!(!rect.contains(v2!(5., -5.)));

        let rect = Rect::new(-2, -4, 10, 20);
        assert!(rect.contains(v2!(-1, -4)));
        assert!(rect.contains(v2!(0, 0)));
        assert!(rect.contains(v2!(1, 3)));
        assert!(rect.contains(v2!(8, 16)));
        assert!(!rect.contains(v2!(8, 17)));
        assert!(!rect.contains(v2!(-4, -5)));
    }

    #[test]
    fn rect_add_vec() {
        let rect = Rect::new(1, 2, 3, 4);
        assert_eq!(rect + v2!(2, 3), Rect::new(3, 5, 3, 4));
        assert_eq!(rect + v2!(0, 0), rect);
    }

    #[test]
    fn rect_sub_vec() {
        let rect = Rect::new(1, 2, 3, 4);
        assert_eq!(rect - v2!(2, 3), Rect::new(-1, -1, 3, 4));
        assert_eq!(rect - v2!(0, 0), rect);
    }

    #[test]
    fn rect_intersect() {
        let a = Rect::new(3, 4, 10, 20);
        let b = Rect::new(4, 1, 300, 5);
        let inter = rects_intersection(&a, &b);
        assert_eq!(inter, Some(Rect::new(4, 4, 9, 2)));

        let c = Rect::new(-30, -40, 8, 19);
        assert_eq!(rects_intersection(&a, &c), None);
    }

    #[test]
    fn aabb_points() {
        let p1 = v2!(0., 0.);
        let p2 = v2!(10., 0.);
        let p3 = v2!(-8., 30.);

        assert_eq!(aabb_of_points(&[p1, p2, p3]), Rect::new(-8., 0., 18., 30.));
    }
}
