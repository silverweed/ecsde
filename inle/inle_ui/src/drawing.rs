use super::ui_context::UI_Context;
use inle_common::colors;
//use crate::gfx::align::Align;
use super::widgets::*;
use inle_common::paint_props::Paint_Properties;
use inle_gfx::render;
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::rect::Rectf;
use inle_math::vector::Vec2f;
use inle_resources::gfx::Gfx_Resources;

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

pub enum Draw_Command {
    Rect {
        rect: Rectf,
        props: Paint_Properties,
    },
    Text {
        text: String,
        props: Paint_Properties,
        font_size: u16,
        pos: Vec2f,
    },
}

pub fn draw_all_ui(window: &mut Render_Window_Handle, gres: &Gfx_Resources, ui: &mut UI_Context) {
    for cmd in &ui.draw_cmd_queue {
        match cmd {
            Draw_Command::Rect { rect, props } => {
                render::render_rect(window, *rect, *props);
            }
            Draw_Command::Text {
                text,
                pos,
                font_size,
                props,
            } => {
                let mut text = render::create_text(&text, gres.get_font(ui.font), *font_size);
                render::render_text(window, &mut text, *props, *pos);
            }
        }
    }
    ui.draw_cmd_queue.clear();
}

pub fn draw_button(
    gres: &Gfx_Resources,
    ui: &UI_Context,
    text: &str,
    rect: Rectf,
    active: bool,
    hot: bool,
    props: &Button_Props,
) -> Vec<Draw_Command> {
    let mut draw_cmds = Vec::with_capacity(2);
    let mut bg_col = *select_ac(&props.bg_color, active, hot);
    let mut text_col = *select_ac(&props.text_color, active, hot);
    if !props.enabled {
        bg_col = disabled_col(bg_col);
        text_col = disabled_col(text_col);
    }

    draw_cmds.push(Draw_Command::Rect {
        rect,
        props: bg_col.into(),
    });

    // @Speed: we're creating a Text just to get its size.
    let txt = render::create_text(text, gres.get_font(ui.font), props.font_size);
    let text_size = render::get_text_size(&txt);
    // @Incomplete: consider Align
    let pos = v2!(rect.x, rect.y) + v2!(rect.width * 0.5, rect.height * 0.5) - text_size * 0.5;

    draw_cmds.push(Draw_Command::Text {
        text: String::from(text),
        pos,
        font_size: props.font_size,
        props: text_col.into(),
    });

    draw_cmds
}
