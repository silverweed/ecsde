use crate::painter::Debug_Painter;
use inle_common::colors;
use inle_gfx::render_window::{self, Render_Window_Handle};
use inle_math::shapes::Arrow;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use inle_win::window;

// Measures distances
#[derive(Default)]
pub struct Debug_Calipers {
    start_world_pos: Vec2f,
    dragging: bool,
}

impl Debug_Calipers {
    pub fn start_measuring_dist(&mut self, window: &Render_Window_Handle, camera: &Transform2D) {
        let pos = window::raw_mouse_pos_in_window(window);
        let pos = render_window::unproject_screen_pos(pos, window, camera);
        self.start_world_pos = pos;
        self.dragging = true;
    }

    pub fn end_measuring(&mut self) {
        self.dragging = false;
    }

    pub fn draw(
        &self,
        window: &Render_Window_Handle,
        painter: &mut Debug_Painter,
        camera: &Transform2D,
    ) {
        if !self.dragging {
            return;
        }

        let end_screen_pos = window::raw_mouse_pos_in_window(window);
        let end_world_pos = render_window::unproject_screen_pos(end_screen_pos, window, camera);
        let delta = end_world_pos - self.start_world_pos;
        let scale = camera.scale().x;
        let arrow = Arrow {
            center: self.start_world_pos,
            direction: delta,
            thickness: (1.5 * scale),
            arrow_size: (20. * scale),
        };
        painter.add_arrow(arrow, colors::FUCHSIA);

        let d = (15. * scale).max(5.);
        let text_pos = arrow.center + arrow.direction + v2!(d, d);
        let font_size = (15. * scale).max(8.) as u16;
        let shadow = v2!(1., 1.) * scale;
        painter.add_text(
            &format!("{}", delta.magnitude()),
            text_pos + shadow,
            font_size,
            colors::WHITE,
        );
        painter.add_text(
            &format!("{}", delta.magnitude()),
            text_pos,
            font_size,
            colors::rgb(143, 0, 50),
        );
    }
}
