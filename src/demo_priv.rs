use core::entity_manager::*;
pub use core::systems::*;
use gfx::window as win;
use sfml::graphics::{RenderWindow, RenderTarget, Vertex, VertexArray, PrimitiveType, Color};
use sfml::system::Vector2f;
use typename::TypeName;

#[derive(Copy, Clone, Debug, Default, TypeName)]
pub struct C_Horiz_Pos {
	x: f32
}

#[derive(Copy, Clone, Debug, Default, TypeName)]
pub struct C_Sine_Wave {
	pub ampl: f32,
	pub freq: f32,
	pub phase: f32,
}

#[derive(Default)]
pub struct S_Sine_Update {
	t: f32
}

impl System for S_Sine_Update {
	fn update(&mut self, dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		self.t += dt;

		// @Refactoring: the entity filtering should probably be done before this step
		let filtered: Vec<Entity> = entities.into_iter().filter(|&&e|
			em.has_component::<C_Sine_Wave>(e) &&
			em.has_component::<C_Horiz_Pos>(e)
		).cloned().collect();

		for e in filtered {
			let (ampl, freq, phase) = if let Some(sine_wave) = em.get_component::<C_Sine_Wave>(e) {
				(sine_wave.ampl, sine_wave.freq, sine_wave.phase)
			} else { panic!("Should have C_Sine_Wave but dont!?!?!?") };

			let mut pos = em.get_component_mut::<C_Horiz_Pos>(e).unwrap();
			pos.x = ampl * (freq * self.t + phase).sin();
		}
	}
}

pub struct S_Particle_Draw_Console {
	pub width: i32
}

impl System for S_Particle_Draw_Console {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		for &e in entities {
			if let Some(&C_Horiz_Pos { x }) = em.get_component::<C_Horiz_Pos>(e) {
				self.draw_at_pos(x as i8);
			}
		}
	}
}

impl S_Particle_Draw_Console {
	fn draw_at_pos(&self, x: i8) {
		let mut buf: Vec<char> = Vec::with_capacity(self.width as usize);

		// x = 0 is at width / 2
		for _i in 0..(x as i32 + self.width / 2) {
			buf.push(' ');
		}
		buf.push('*');

		let s: String = buf.into_iter().collect();
		println!("{}", s);
	}
}

pub struct S_Particle_Draw_Gfx {
	window: RenderWindow,
	vertex_array: VertexArray,
	vertices: Vec<Vertex>,
	pub point_width: f32,
}

impl System for S_Particle_Draw_Gfx {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		self.vertices.clear();
		self.vertices.reserve(3 * entities.len()); // 3x because we're drawing triangles
		for &e in entities {
			if let Some(&C_Horiz_Pos { x }) = em.get_component::<C_Horiz_Pos>(e) {
				self.add_particle_to_draw(x);
			}
		}
		self.draw();
	}
}

impl S_Particle_Draw_Gfx {
	pub fn new(window: RenderWindow) -> S_Particle_Draw_Gfx {
		S_Particle_Draw_Gfx {
			window,
			vertex_array: VertexArray::new(
				PrimitiveType::Triangles,
				64
			),
			vertices: Vec::new(),
			point_width: 5f32,
		}
	}

	pub fn should_close(&self) -> bool { !self.window.is_open() }

	pub fn event_loop(&mut self) {
		win::event_loop(&mut self.window);
	}

	fn add_particle_to_draw(&mut self, x: f32) {
		let half_width = self.window.size().x as f32 / 2f32;
		let half_height = self.window.size().y as f32 / 2f32;
		let (vx, vy) = (half_width + x as f32, half_height);
		let hpw = self.point_width * 0.5;

		self.vertices.push(Vertex::new(
			Vector2f::new(vx - hpw, vy - hpw),
			Color::RED,
			Vector2f::new(0., 0.)
		));
		self.vertices.push(Vertex::new(
			Vector2f::new(vx + hpw, vy - hpw),
			Color::RED,
			Vector2f::new(0., 0.)
		));
		self.vertices.push(Vertex::new(
			Vector2f::new(vx, vy + hpw),
			Color::RED,
			Vector2f::new(0., 0.)
		));
	}

	pub fn draw(&mut self) {
		self.vertex_array.clear();
		self.vertex_array.resize(self.vertices.len());
		for i in 0..self.vertices.len() {
			self.vertex_array[i] = self.vertices[i];
		}
		self.window.clear(&Color::BLACK);
		self.window.draw(&self.vertex_array);
		self.window.display();
	}
}

