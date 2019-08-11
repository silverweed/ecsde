use crate::core::common::colors::{self, Color};
use crate::core::common::vector::{to_framework_vec, Vec2f};
use crate::core::common::Maybe_Error;
use crate::core::env::Env_Info;
use crate::gfx;
use crate::gfx::window::{self, Window_Handle};
use crate::resources;
use crate::resources::gfx::{Font_Handle, Gfx_Resources};

#[cfg(feature = "use-sfml")]
use sfml::graphics::Text;
#[cfg(feature = "use-sfml")]
use sfml::graphics::Transformable;

struct Debug_Line {
    pub text: String,
    pub color: Color,
}

pub struct Debug_Overlay {
    lines: Vec<Debug_Line>,
    font: Font_Handle,
}

// @Cleanup: much of this guy's functionality overlaps with UI
impl Debug_Overlay {
    const PAD_X: f32 = 5.0;
    const PAD_Y: f32 = 5.0;
    const ROW_HEIGHT: usize = 20;
    const FONT: &'static str = "Hack-Regular.ttf";
    const FONT_SIZE: u16 = 16;

    pub fn new() -> Debug_Overlay {
        Debug_Overlay {
            lines: vec![],
            font: None,
        }
    }

    pub fn init(&mut self, env: &Env_Info, gres: &mut Gfx_Resources) -> Maybe_Error {
        self.font = gres.load_font(&resources::gfx::font_path(env, Self::FONT));
        if self.font.is_none() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to load font for Debug_Overlay!",
            )))
        } else {
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn add_line(&mut self, line: &str) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color: colors::rgb(255, 255, 255),
        });
    }

    pub fn add_line_col(&mut self, line: &str, color: Color) {
        self.lines.push(Debug_Line {
            text: String::from(line),
            color,
        });
    }

    pub fn draw(&self, window: &mut Window_Handle, gres: &mut Gfx_Resources) {
        // @Cutnpaste from UI
        for (i, line) in self.lines.iter().enumerate() {
            let Debug_Line { text, color } = line;
            let text = {
                let mut text = Text::new(text, gres.get_font(self.font), Self::FONT_SIZE.into());
                text.set_fill_color(&color);
                text.set_position(to_framework_vec(Vec2f::new(
                    window::get_window_target_size(window).0 as f32
                        - text.local_bounds().width
                        - Self::PAD_X,
                    Self::PAD_Y + (i * Self::ROW_HEIGHT) as f32,
                )));
                text
            };

            gfx::render::render_text(window, &text);
        }
    }
}
