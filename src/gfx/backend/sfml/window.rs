use crate::core::common::colors::{self, Color};
use crate::gfx;
use sfml::graphics::blend_mode::BlendMode;
use sfml::graphics::{RenderTarget, RenderWindow};
use sfml::window;

// @Incomplete: this will probably be the ContextSettings
pub type Create_Render_Window_Args = ();

pub struct Window_Handle {
    pub(super) handle: RenderWindow,
    clear_color: Color,
    pub(super) blend_mode: gfx::render::Blend_Mode,
}

pub fn create_render_window(
    _: &Create_Render_Window_Args,
    target_size: (u32, u32),
    title: &str,
) -> Window_Handle {
    let mut window = RenderWindow::new(
        target_size,
        title,
        window::Style::DEFAULT,
        &window::ContextSettings::default(), // @Incomplete
    );
    window.set_vertical_sync_enabled(true);
    Window_Handle {
        handle: window,
        clear_color: colors::rgb(0, 0, 0),
        blend_mode: BlendMode::NONE,
    }
}

pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    window.clear_color = color;
}

pub fn clear(window: &mut Window_Handle) {
    let c = window.clear_color;
    window.handle.clear(&c);
}

pub fn display(window: &mut Window_Handle) {
    window.handle.display();
}
