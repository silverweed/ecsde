use super::ui::UI_Context;
use crate::common::colors;
use crate::common::rect::Rectf;
use crate::gfx::render;
use crate::resources::gfx::Gfx_Resources;
use crate::gfx::window::Window_Handle;

pub fn draw_button(window: &mut Window_Handle, gres: &Gfx_Resources, ui: &UI_Context, text: &str, rect: Rectf, active: bool, hot: bool) {
    let col = if active {
        colors::YELLOW
    } else if hot {
        colors::rgb(200, 200, 200)
    } else {
        colors::WHITE
    };
    render::render_rect(window, rect, col);
    let mut text = render::create_text(text, gres.get_font(ui.font), 12);
    render::render_text(window, &mut text, colors::BLACK, v2!(rect.x, rect.y) + v2!(1., 1.));
}
