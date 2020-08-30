use super::Rect;

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
        // Safe because we have the same repr as sfml Rect
        unsafe { &*(self as *const _ as *const Rect<T>) }
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
        // Safe because we have the same repr as sfml Rect
        unsafe { &*(self as *const _ as *const sfml::graphics::Rect<T>) }
    }
}
