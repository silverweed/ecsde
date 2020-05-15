use super::ui_context::UI_Context;
use crate::common::colors;
//use crate::gfx::align::Align;
use super::widgets::*;
use crate::common::rect::Rectf;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx::Gfx_Resources;

fn select_ac<T>(by_ac: &By_Activeness<T>, active: bool, hot: bool) -> &T {
    if active {
        &by_ac.active
    } else if hot {
        &by_ac.hot
    } else {
        &by_ac.normal
    }
}

#[inline]
fn disabled_col(c: colors::Color) -> colors::Color {
    colors::darken(colors::to_gray_scale(c), 0.5)
}

pub fn draw_button(
    window: &mut Window_Handle,
    gres: &Gfx_Resources,
    ui: &UI_Context,
    text: &str,
    rect: Rectf,
    active: bool,
    hot: bool,
    props: &Button_Props,
) {
    let mut bg_col = *select_ac(&props.bg_color, active, hot);
    let mut text_col = *select_ac(&props.text_color, active, hot);
    if !props.enabled {
        bg_col = disabled_col(bg_col);
        text_col = disabled_col(text_col);
    }

    render::render_rect(window, rect, bg_col);
    let mut text = render::create_text(text, gres.get_font(ui.font), props.font_size);
    let text_size = render::get_text_size(&text);
    // @Incomplete: consider Align
    let pos = v2!(rect.x, rect.y) + v2!(rect.width * 0.5, rect.height * 0.5) - text_size * 0.5;
    render::render_text(window, &mut text, text_col, pos);
}
