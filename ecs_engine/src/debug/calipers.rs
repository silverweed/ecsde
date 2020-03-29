use crate::common::colors;
use crate::common::shapes::Arrow;
use crate::common::transform::Transform2D;
use crate::common::vector::Vec2f;
use crate::debug::painter::Debug_Painter;
use crate::gfx::window::{self, Window_Handle};

// Measures distances
#[derive(Default)]
pub struct Debug_Calipers {
    start_world_pos: Vec2f,
    dragging: bool,
}

impl Debug_Calipers {
    pub fn start_measuring_dist(&mut self, window: &Window_Handle, camera: &Transform2D) {
        let pos = window::raw_mouse_pos_in_window(window);
        let pos = window::unproject_screen_pos(pos, window, camera);
        self.start_world_pos = pos.into();
        self.dragging = true;
    }

    pub fn end_measuring(&mut self) {
        self.dragging = false;
    }

    pub fn draw(&self, window: &Window_Handle, painter: &mut Debug_Painter, camera: &Transform2D) {
        if !self.dragging {
            return;
        }

        let end_screen_pos = window::raw_mouse_pos_in_window(window);
        let end_world_pos = window::unproject_screen_pos(end_screen_pos, window, camera);
        let delta = end_world_pos - self.start_world_pos;
        let arrow = Arrow {
            center: self.start_world_pos,
            direction: delta,
            thickness: 1.5,
            arrow_size: 20.,
        };
        painter.add_arrow(arrow, colors::FUCHSIA);

        let text_pos = arrow.center + arrow.direction + v2!(15., 15.);
        let font_size = (15. * camera.scale().x).max(12.) as u16;
        let shadow = v2!(1., 1.) * camera.scale();
        painter.add_text(
            &format!("world dist {}", delta.magnitude()),
            text_pos + shadow,
            font_size,
            colors::WHITE,
        );
        painter.add_text(
            &format!("world dist {}", delta.magnitude()),
            text_pos,
            font_size,
            colors::rgb(143, 0, 50),
        );
    }
}
