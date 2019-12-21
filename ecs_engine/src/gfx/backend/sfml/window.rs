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
    target_size: (u32, u32),
}

impl std::ops::Deref for Window_Handle {
    type Target = RenderWindow;

    fn deref(&self) -> &RenderWindow {
        &self.handle
    }
}

impl std::ops::DerefMut for Window_Handle {
    fn deref_mut(&mut self) -> &mut RenderWindow {
        &mut self.handle
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
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
    window.set_key_repeat_enabled(false);
    Window_Handle {
        handle: window,
        clear_color: colors::rgb(0, 0, 0),
        blend_mode: BlendMode::ALPHA,
        target_size,
    }
}

pub fn destroy_render_window(window: &mut Window_Handle) {
    window.handle.close();
}

pub fn set_clear_color(window: &mut Window_Handle, color: Color) {
    window.clear_color = color;
}

pub fn clear(window: &mut Window_Handle) {
    let c = window.clear_color;
    window.handle.clear(c);
}

pub fn display(window: &mut Window_Handle) {
    window.handle.display();
}

pub fn resize_keep_ratio(window: &mut Window_Handle, new_width: u32, new_height: u32) {
    use sfml::graphics as sfgfx;
    use std::cmp::Ordering;

    let (target_width, target_height) = window.target_size;
    let screen_width = new_width as f32 / target_width as f32;
    let screen_height = new_height as f32 / target_height as f32;

    // @Robustness: what do we do if width or height are zero?
    debug_assert!(screen_width.is_normal());
    debug_assert!(screen_height.is_normal());

    let mut viewport = sfgfx::FloatRect::new(0.0, 0.0, 1.0, 1.0);
    match screen_width.partial_cmp(&screen_height) {
        Some(Ordering::Greater) => {
            viewport.width = screen_height / screen_width;
            viewport.left = 0.5 * (1.0 - viewport.width);
        }
        Some(Ordering::Less) => {
            viewport.height = screen_width / screen_height;
            viewport.top = 0.5 * (1.0 - viewport.height);
        }
        _ => {}
    }
    let mut view = sfgfx::View::from_rect(&sfgfx::FloatRect::new(
        0.0,
        0.0,
        target_width as f32,
        target_width as f32,
    ));
    view.set_viewport(&viewport);
    window.set_view(&view);
}

pub fn get_window_target_size(window: &Window_Handle) -> (u32, u32) {
    window.target_size
}
