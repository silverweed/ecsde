#![allow(non_camel_case_types)]

extern crate typename;
extern crate sfml;

mod core;
mod gfx;
mod demo_priv;

use std::time::{SystemTime, Duration};
use std::f32;

use gfx::window as win;
use demo_priv::*;
use core::entity_manager::*;

pub mod demo {
	use super::*;

	pub enum Demo_Type {
		Console,
		Gfx,
	}

	pub struct Config {
		pub demo_type: Demo_Type,
		pub n_particles: usize,
		pub n_sine_waves: usize,
	}

	impl Config {
		const default_particles: usize = 2usize;

		pub fn new(mut args: std::env::Args) -> Self {
			// ignore program name
			args.next();
			let demo_type = args.next().map_or(Demo_Type::Console, |s|
				match s.as_ref() {
					"gfx" | "g" => Demo_Type::Gfx,
					_ => Demo_Type::Console,
				}
			);
			let n_particles = args.next().map_or(Self::default_particles, |n|
				n.parse::<usize>().unwrap_or(Self::default_particles)
			);
			let n_sine_waves = args.next().map_or(n_particles, |n|
				n.parse::<usize>().unwrap_or(n_particles)
			);
			Config {
				demo_type,
				n_particles,
				n_sine_waves
			}
		}
	}

	pub fn run(config: &Config) {
		match config.demo_type {
			Demo_Type::Console => console_test(&config),
			Demo_Type::Gfx => gfx_test(&config)
		}
	}

	fn console_test(config: &Config) {
		let mut em = Entity_Manager::new();

		em.register_component::<C_Horiz_Pos>();
		em.register_component::<C_Sine_Wave>();

		let sleep_ms = Duration::from_millis(16);
		let n = config.n_particles;
		let n_sin = config.n_sine_waves;

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

	fn gfx_test(config: &Config) {
		let window = win::create_render_window((800, 600), "Gfx test");
		let mut em = Entity_Manager::new();

		em.register_component::<C_Horiz_Pos>();
		em.register_component::<C_Sine_Wave>();

		let n = config.n_particles;
		let n_sin = config.n_sine_waves;

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
	}

	fn init_components(em: &mut Entity_Manager, wave: Entity, phase: f32) {
		if let Some(sine_wave) = em.get_component_mut::<C_Sine_Wave>(wave) {
			sine_wave.ampl = 40;
			sine_wave.freq = 6f32;
			sine_wave.phase = phase;
		}
	}
}
