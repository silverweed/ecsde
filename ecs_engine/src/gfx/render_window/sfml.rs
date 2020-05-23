use crate::common::colors::{self, Color};
use crate::common::rect::Rectf;
use crate::common::transform::Transform2D;
use crate::common::vector::{Vec2f, Vec2i};
use crate::gfx::window::Window_Handle;
use sfml::graphics::{RenderTarget, RenderWindow};

pub struct Render_Window_Handle {
    render_window: RenderWindow,
    window: Window_Handle,
    clear_color: Color,
}

impl Render_Window_Handle {
    #[inline(always)]
    pub fn raw_handle(&self) -> &RenderWindow {
        &self.render_window
    }

    #[inline(always)]
    pub fn raw_handle_mut(&mut self) -> &mut RenderWindow {
        &mut self.render_window
    }
}

impl AsRef<Window_Handle> for Render_Window_Handle {
    fn as_ref(&self) -> &Window_Handle {
        &self.window
    }
}

impl AsMut<Window_Handle> for Render_Window_Handle {
    fn as_mut(&mut self) -> &mut Window_Handle {
        &mut self.window
    }
}

pub fn create_render_window(window: Window_Handle) -> Render_Window_Handle {
    let render_window =
        unsafe { RenderWindow::from_handle(window.raw_handle().handle(), &Default::default()) };
    Render_Window_Handle {
        render_window,
        window,
        clear_color: colors::BLACK,
    }
}

pub fn set_clear_color(window: &mut Render_Window_Handle, color: Color) {
    window.clear_color = color;
}

pub fn clear(window: &mut Render_Window_Handle) {
    let c = window.clear_color.into();
    window.render_window.clear(c);
}

pub fn display(window: &mut Render_Window_Handle) {
    window.render_window.display();
}

pub fn set_viewport(window: &mut Render_Window_Handle, viewport: &Rectf, view_rect: &Rectf) {
    use sfml::graphics::View;

    let mut view = View::from_rect(view_rect.as_ref());
    view.set_viewport(viewport.as_ref());
    window.render_window.set_view(&view);
}

pub fn raw_unproject_screen_pos(
    screen_pos: Vec2i,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2f {
    let pos_cam_space = window
        .render_window
        .map_pixel_to_coords_current_view(screen_pos.into());
    let world_pos = camera.get_matrix_sfml().transform_point(pos_cam_space);
    world_pos.into()
}

pub fn raw_project_world_pos(
    world_pos: Vec2f,
    window: &Render_Window_Handle,
    camera: &Transform2D,
) -> Vec2i {
    let pos_cam_space = camera
        .get_matrix_sfml()
        .inverse()
        .transform_point(world_pos.into());
    let screen_pos = window
        .render_window
        .map_coords_to_pixel_current_view(pos_cam_space);
    screen_pos.into()
}
