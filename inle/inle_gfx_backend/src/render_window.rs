#[cfg(feature = "gfx-null")]
pub mod null;

#[cfg(feature = "gfx-gl")]
pub mod gl;

#[cfg(feature = "gfx-null")]
pub use self::null as backend;

#[cfg(feature = "gfx-gl")]
pub use self::gl as backend;

pub type Render_Window_Handle = backend::Render_Window_Handle;
