#![allow(non_camel_case_types)]

extern crate typename;
extern crate sfml;

mod core;
mod gfx;

use std::time::{SystemTime, Duration};
use std::f32;
use std::env;
use std::error::Error;
use std::str::FromStr;

use core::entity_manager::*;
use core::systems::*;
use gfx::window as win;
use sfml::graphics::{RenderWindow, RenderTarget, Vertex, VertexArray, PrimitiveType, Color};
use sfml::system::Vector2f;
use typename::TypeName;

#[derive(Copy, Clone, Debug, Default, TypeName)]
struct C_Horiz_Pos {
	x: i8
}

#[derive(Copy, Clone, Debug, Default, TypeName)]
struct C_Sine_Wave {
	ampl: u8,
	freq: f32,
	phase: f32,
}

#[derive(Default)]
struct S_Sine_Update {
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
				(sine_wave.ampl as f32, sine_wave.freq, sine_wave.phase)
			} else { panic!("Should have C_Sine_Wave but dont!?!?!?") };

			let mut pos = em.get_component_mut::<C_Horiz_Pos>(e).unwrap();
			pos.x = (ampl * (freq * self.t + phase).sin()) as i8;
		}
	}
}

struct S_Particle_Draw_Console {
	width: i32
}

impl System for S_Particle_Draw_Console {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		for &e in entities {
			if let Some(&C_Horiz_Pos { x }) = em.get_component::<C_Horiz_Pos>(e) {
				self.draw_at_pos(x);
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

fn console_test(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
	let mut em = Entity_Manager::new();

	em.register_component::<C_Horiz_Pos>();
	em.register_component::<C_Sine_Wave>();

	let sleep_ms = Duration::from_millis(16);
	let n =  parse_nth_argument_or(&args, 1, 2usize)?;
	let n_sin = parse_nth_argument_or(&args, 2, n)?;

	let mut entities: Vec<Entity> = Vec::with_capacity(n);

	for i in 1..n+1 {
		let e = em.new_entity();
		entities.push(e);
		em.add_component::<C_Horiz_Pos>(e);
		if i % n_sin != 0 { em.add_component::<C_Sine_Wave>(e); }
		init_components(&mut em, e, 0f32 + i as f32 * 3.1415f32 / (n as f32));
	}

	let mut sine_update_system = S_Sine_Update::default();
	let mut draw_system = S_Particle_Draw_Console { width: 180 };

	let mut time = SystemTime::now();

	loop {
		let dtr = time.elapsed().unwrap();
		let dt = dtr.as_secs() as f32 + dtr.subsec_nanos() as f32 * 1e-9;
		time = SystemTime::now();

		sine_update_system.update(dt, &mut em, &entities);
		draw_system.update(dt, &mut em, &entities);

		std::thread::sleep(sleep_ms);
	}
}

struct S_Particle_Draw_Gfx {
	window: RenderWindow,
	vertex_array: VertexArray,
	vertices: Vec<Vertex>,
}

impl System for S_Particle_Draw_Gfx {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		self.vertices.clear();
		self.vertices.reserve(entities.len());
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
				PrimitiveType::Points,
				64
			),
			vertices: Vec::new()
		}
	}

	pub fn should_close(&self) -> bool { !self.window.is_open() }

	pub fn event_loop(&mut self) {
		win::event_loop(&mut self.window);
	}

	fn add_particle_to_draw(&mut self, x: i8) {
		let half_height = self.window.size().y as f32 / 2f32;
		self.vertices.push(Vertex::new(
			Vector2f::new(x as f32, half_height),
			Color::RED,
			Vector2f::new(0., 0.)
		));
	}

	fn draw(&mut self) {
		self.vertex_array.clear();
		self.vertex_array.resize(self.vertices.len());
		for i in 0..self.vertices.len() {
			self.vertex_array[i] = self.vertices[i];
		}
		self.window.clear(&Color::BLACK);
		self.window.draw(&self.vertex_array);
	}
}

fn gfx_test(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
	let window = win::create_render_window((800, 600), "Gfx test");
	let mut em = Entity_Manager::new();

	em.register_component::<C_Horiz_Pos>();
	em.register_component::<C_Sine_Wave>();

	let n = parse_nth_argument_or(&args, 1, 2usize)?;
	let n_sin = parse_nth_argument_or(&args, 2, n)?;

	let mut entities: Vec<Entity> = Vec::with_capacity(n);

	for i in 1..n+1 {
		let e = em.new_entity();
		entities.push(e);
		em.add_component::<C_Horiz_Pos>(e);
		if i % n_sin != 0 { em.add_component::<C_Sine_Wave>(e); }
		init_components(&mut em, e, 0f32 + i as f32 * 3.1415f32 / (n as f32));
	}

	let mut sine_update_system = S_Sine_Update::default();
	let mut draw_system = S_Particle_Draw_Gfx::new(window);

	let mut time = SystemTime::now();

	while !draw_system.should_close() {
		let dtr = time.elapsed().unwrap();
		let dt = dtr.as_secs() as f32 + dtr.subsec_nanos() as f32 * 1e-9;
		time = SystemTime::now();

		draw_system.event_loop();

		sine_update_system.update(dt, &mut em, &entities);
		draw_system.update(dt, &mut em, &entities);
	}
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = env::args().collect();
	//console_test(&args)
	gfx_test(&args)
}

fn init_components(em: &mut Entity_Manager, wave: Entity, phase: f32) {
	if let Some(sine_wave) = em.get_component_mut::<C_Sine_Wave>(wave) {
		sine_wave.ampl = 40;
		sine_wave.freq = 6f32;
		sine_wave.phase = phase;
	}
}

fn parse_nth_argument_or<T: FromStr + TypeName>(args: &Vec<String>, n: usize, default: T) -> Result<T, String> {
	if args.len() > n {
		if let Ok(x) = args[n].parse::<T>() {
			Ok(x)
		} else {
			Err(format!("Expected a type compatible with {} as first argument.", T::type_name()))
		}
	} else {
		Ok(default)
	}
}
