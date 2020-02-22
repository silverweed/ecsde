use rustyline;
use crate::core::common::rect::Rect;
use crate::core::common::colors;
use crate::core::common::vector::Vec2u;
use crate::gfx::render;
use crate::gfx::window::Window_Handle;
use crate::resources::gfx;
use crate::prelude::*;

#[derive(PartialEq, Eq)]
pub enum Console_Status {
	Open,
	Closed
}

pub struct Console {
	editor: rustyline::Editor<()>,
	pub status: Console_Status,
	pub pos: Vec2u,
	pub size: Vec2u,
}

impl Console {
	pub fn new() -> Self {
		Self {
			editor: rustyline::Editor::new(),
			status: Console_Status::Closed,
			pos: Vec2u::default(),
			size: Vec2u::default(),
		}
	}

	pub fn draw(
		&self,
        window: &mut Window_Handle,
        gres: &mut gfx::Gfx_Resources,
        tracer: Debug_Tracer,
	) {
		if self.status == Console_Status::Closed {
			return;
		}

		let Vec2u { x, y } = self.pos;
		let Vec2u { x: w, y: h } = self.size;
		render::fill_color_rect(window, colors::rgba(0, 0, 0, 150), Rect::new(x, y, w, h));
	}
}
