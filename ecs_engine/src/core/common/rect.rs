#[cfg(feature = "use-sfml")]
mod sfml;

#[cfg(feature = "use-sfml")]
use self::sfml as backend;

#[cfg(feature = "use-sfml")]
pub type Rect<T> = backend::Rect<T>;

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
