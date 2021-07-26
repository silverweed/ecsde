use inle_math::matrix::Matrix3;
use inle_math::transform::Transform2D;

#[cfg(feature = "gfx-sfml")]
pub mod sfml;

#[cfg(feature = "gfx-null")]
pub mod null;

#[cfg(feature = "gfx-gl")]
pub mod gl;

#[cfg(feature = "gfx-sfml")]
pub use self::sfml as backend;

#[cfg(feature = "gfx-null")]
pub use self::null as backend;

#[cfg(feature = "gfx-gl")]
pub use self::gl as backend;

pub type Render_Window_Handle = backend::Render_Window_Handle;

/// Note: we use the camera scale as the zoom factor.
/// Since the view matrix doesn't want to be scaled, this function just transforms a camera transform
/// into a view matrix by setting its scale to 1, 1
#[inline]
pub fn get_view_matrix(camera: &Transform2D) -> Matrix3<f32> {
    let mut view = *camera;
    view.set_scale(1., 1.);
    view.inverse().get_matrix()
}

#[inline]
pub fn get_inverse_view_matrix(camera: &Transform2D) -> Matrix3<f32> {
    let mut view = *camera;
    view.set_scale(1., 1.);
    view.get_matrix()
}
