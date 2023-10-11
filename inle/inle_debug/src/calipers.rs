use crate::painter::Debug_Painter;
use inle_common::colors;
use inle_gfx::render_window::{self, Render_Window_Handle};
use inle_input::input_state::Input_State;
use inle_input::mouse;
use inle_math::shapes::Arrow;
use inle_math::vector::{Vec2f, Vec2i};
use inle_win::window::Camera;

// Measures distances
#[derive(Default)]
pub struct Debug_Calipers {
    start_world_pos: Vec2f,
    dragging: bool,
}

impl Debug_Calipers {
    pub fn start_measuring_dist(
        &mut self,
        window: &Render_Window_Handle,
        camera: &Camera,
        input_state: &Input_State,
    ) {
        let mpos = mouse::raw_mouse_pos_v2i(&input_state.raw.mouse_state);
        let world_pos = render_window::unproject_screen_pos(mpos, window, camera);
        dbg!((mpos, world_pos));
        self.start_world_pos = world_pos;
        self.dragging = true;
    }

    pub fn end_measuring(&mut self) {
        self.dragging = false;
    }

    pub fn draw(
        &self,
        window: &Render_Window_Handle,
        painter: &mut Debug_Painter,
        camera: &Camera,
        input_state: &Input_State,
    ) {
        if !self.dragging {
            return;
        }

        let end_screen_pos = mouse::raw_mouse_pos_v2i(&input_state.raw.mouse_state);
        let end_world_pos = render_window::unproject_screen_pos(end_screen_pos, window, camera);
        let delta = end_world_pos - self.start_world_pos;
        let scale = camera.transform.scale().x;
        let arrow = Arrow {
            center: self.start_world_pos,
            direction: delta,
            thickness: (1.5 * scale),
            arrow_size: (20. * scale),
        };
        painter.add_arrow(arrow, colors::FUCHSIA);

        let d = (15. * scale).max(5.);
        let text_pos = arrow.center + arrow.direction + v2!(d, d);
        let font_size = (17. * scale).max(8.) as u16;
        let shadow = v2!(1., 1.) * scale;
        painter.add_text(
            &format!("{:.2}", delta.magnitude()),
            text_pos + shadow,
            font_size,
            colors::WHITE,
        );
        painter.add_text(
            &format!("{:.2}", delta.magnitude()),
            text_pos,
            font_size,
            colors::rgb(143, 0, 50),
        );
    }
}
