mod systems;

use crate::core::entity_manager::{Entity_Manager, Entity};
use crate::core::components::C_Position2D;
use crate::core::systems::System;
use crate::core::time;
use crate::gfx::render;
use crate::gfx::window as win;

use self::systems::*;

use sfml::graphics as sfgfx;
use sfml::graphics::RenderTarget;
use sfml::system as sfsys;

pub fn run() {
	let mut em = Entity_Manager::new();
	em.register_component::<C_Charge>();
	em.register_component::<C_Field>();
	em.register_component::<C_Position2D>();

	let cfg = Create_Config {
		n_particles: 1000,
		world_size: World_Size {
			x: 800,
			y: 600
		},
		max_charge: 200f32,
	};
	let entities = create_particles(&mut em, &cfg);

	let window = win::create_render_window((cfg.world_size.x as u32, cfg.world_size.y as u32), "Particles Demo");
	let winsize = window.sf_win.size();

	let mut s_calc_field = S_Calculate_Field {};
	let mut s_move_particles = S_Move_Particles::new(&sfsys::Vector2f::new(winsize.x as f32, winsize.y as f32));
	let mut s_draw = S_Particle_Draw_Gfx::new(&winsize);

	let mut renderer = render::Renderer::new(window);

	let mut t = time::Time::new();

	while !renderer.should_close() {
		t.update();
		renderer.event_loop();

		s_calc_field.update(t.dt(), &mut em, &entities);
		s_move_particles.update(t.dt(), &mut em, &entities);
		s_draw.update(t.dt(), &mut em, &entities);

		let drawables: [&sfgfx::Drawable; 1] = [&s_draw];
		renderer.draw(&drawables[..]);
	}
}

struct World_Size {
	pub x: usize,
	pub y: usize
}

struct Create_Config {
	pub n_particles: usize,
	pub world_size: World_Size,
	pub max_charge: f32
}

fn create_particles(em: &mut Entity_Manager, cfg: &Create_Config) -> Vec<Entity> {
	use rand::Rng;

	let mut entities: Vec<Entity> = vec![];
	let mut rng = rand::thread_rng();

	for _i in 0..cfg.n_particles {
		let e = em.new_entity();
		{
			let pos = em.add_component::<C_Position2D>(e);
			pos.x = rng.gen_range(0f32, cfg.world_size.x as f32);
			pos.y = rng.gen_range(0f32, cfg.world_size.y as f32);
		}
		{
			let charge = em.add_component::<C_Charge>(e);
			charge.charge = systems::Charge::new(rng.gen_range(-cfg.max_charge, cfg.max_charge));
			//charge.charge = systems::Charge::new(rng.gen_range(0f32, cfg.max_charge));
			/*charge.charge = systems::Charge::new(100f32);*/
		}
		em.add_component::<C_Field>(e);
		entities.push(e);
	}

	entities
}
