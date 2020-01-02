use std::fmt::{Debug, Formatter, Result};
use std::ops::{Add, Sub};

#[derive(Copy, Clone)]
pub struct Rect<T: Copy + Clone>(sfml::graphics::Rect<T>);

// The most boring facade ever written.
impl<T> Rect<T>
where
    T: Copy + Clone + Debug,
{
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        Rect(sfml::graphics::Rect::new(x, y, w, h))
    }

    #[inline]
    pub fn x(&self) -> T {
        self.0.left
    }

    #[inline]
    pub fn y(&self) -> T {
        self.0.top
    }

    #[inline]
    pub fn set_x(&mut self, x: T) {
        self.0.left = x;
    }

    #[inline]
    pub fn set_y(&mut self, y: T) {
        self.0.top = y;
    }

    #[inline]
    pub fn width(&self) -> T {
        self.0.width
    }

    #[inline]
    pub fn height(&self) -> T {
        self.0.height
    }

    #[inline]
    pub fn set_width(&mut self, w: T) {
        self.0.width = w;
    }

    #[inline]
    pub fn set_height(&mut self, h: T) {
        self.0.height = h;
    }
}

pub fn rects_intersect<T>(a: &Rect<T>, b: &Rect<T>) -> bool
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy,
{
    a.0.intersection(&b.0).is_some()
}

impl<T> std::ops::Deref for Rect<T>
where
    T: Copy + Clone + Debug,
{
    type Target = sfml::graphics::Rect<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Rect<T>
where
    T: Copy + Clone + Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Debug for Rect<T>
where
    T: Copy + Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{{ x: {:?}, y: {:?}, w: {:?}, h: {:?} }}",
            self.0.left, self.0.top, self.0.width, self.0.height
        )
    }
}

impl<T> std::convert::From<sfml::graphics::Rect<T>> for Rect<T>
where
    T: Copy + Clone + Debug,
{
    fn from(r: sfml::graphics::Rect<T>) -> Self {
        Rect::new(r.left, r.top, r.width, r.height)
    }
}
