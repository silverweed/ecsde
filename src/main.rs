#![allow(non_camel_case_types)]

#[macro_use] extern crate typename;

mod entity_manager;
mod components;
mod systems;

use entity_manager::*;
use systems::*;
use std::time::{SystemTime, Duration};
use std::f32;

// Dummy components for testing
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

impl S_Sine_Update {
	fn update(&mut self, dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		self.t += dt;

		let filtered: Vec<Entity> = entities.into_iter().filter(
					|e| em.has_component::<C_Sine_Wave>(**e) && em.has_component::<C_Horiz_Pos>(**e)
				).cloned().collect();

		// @Refactoring: the entity filtering should probably be done before this step
		for e in filtered {
			let (ampl, freq, phase) = if let Some(sine_wave) = em.get_component::<C_Sine_Wave>(e) {
				(sine_wave.ampl as f32, sine_wave.freq, sine_wave.phase)
			} else { panic!("Should have C_Sine_Wave but dont!?!?!?") };
			let mut pos = em.get_component_mut::<C_Horiz_Pos>(e).unwrap();

			pos.x = (ampl * (freq * self.t + phase).sin()) as i8;
		}
	}
}

struct S_Particle_Draw {
	width: i32
}

impl System for S_Particle_Draw {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		for e in entities {
			let pos = em.get_component::<C_Horiz_Pos>(*e);
			if pos.is_none() { continue; }

			let &C_Horiz_Pos { x } = pos.unwrap();

			self.draw_at_pos(x);
		}
	}
}

impl S_Particle_Draw {
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

fn main() {
	let mut em = Entity_Manager::new();

	em.register_component::<C_Horiz_Pos>();
	em.register_component::<C_Sine_Wave>();

	let sleep_ms = Duration::from_millis(16);

	let n = 2;
	let mut entities: Vec<Entity> = Vec::with_capacity(n);

	for i in 0..n {
		let e = em.new_entity();
		entities.push(e);
		em.add_component::<C_Horiz_Pos>(e);
		em.add_component::<C_Sine_Wave>(e);
		init_components(&mut em, e, 0f32 + i as f32 * 3.1415f32 / (n as f32));
	}

	let mut sine_update_system = S_Sine_Update::default();
	let mut draw_system = S_Particle_Draw { width: 180 };

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

fn init_components(em: &mut Entity_Manager, wave: Entity, phase: f32) {
	let sine_wave = em.get_component_mut::<C_Sine_Wave>(wave).unwrap();
	sine_wave.ampl = 40;
	sine_wave.freq = 6f32;
	sine_wave.phase = phase;
}
