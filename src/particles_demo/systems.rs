use crate::core::entity_manager::{Entity_Manager, Entity};
use crate::core::systems::System;
use crate::core::components;
use crate::core::components::C_Position2D;
use crate::gfx::window;

use sfml::graphics as sfgfx;
use sfml::graphics::RenderTarget;
use sfml::system as sfsys;

use typename::TypeName;

#[derive(Copy, Clone, Debug, Default)]
pub struct Charge(f32);

impl Charge {
	pub fn new(v: f32) -> Charge {
		Charge(v)
	}
}

#[derive(Copy, Clone, Debug, Default)]
struct Force2D {
	x: f32,
	y: f32
}

impl std::ops::AddAssign for Force2D {
	fn add_assign(&mut self, other: Self) {
		*self = Force2D {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}

impl std::ops::Mul<f32> for Force2D {
	type Output = Self;
	fn mul(self, other: f32) -> Self {
		Force2D {
			x: self.x * other,
			y: self.y * other
		}
	}
}

#[derive(Copy, Clone, Debug, Default, TypeName)]
pub struct C_Charge {
	pub charge: Charge
}

#[derive(Copy, Clone, Debug, Default, TypeName)]
pub struct C_Field {
	field: Force2D
}

pub struct S_Calculate_Field {}

impl System for S_Calculate_Field {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		let filtered: Vec<Entity> = entities.into_iter().filter(|&&e|
			em.has_component::<C_Position2D>(e) &&
			em.has_component::<C_Charge>(e) &&
			em.has_component::<C_Field>(e)
		).cloned().collect();

		let mut modified_fields: Vec<C_Field> = vec![];
		modified_fields.reserve(filtered.len());

		for &e1 in filtered.iter() {
			let pos1 = em.get_component::<C_Position2D>(e1).unwrap();
			let mut field = C_Field::default();
			for &e2 in filtered.iter() {
				if e1 == e2 { continue; }
				let charge = em.get_component::<C_Charge>(e2).unwrap().charge;
				let pos2 = em.get_component::<C_Position2D>(e2).unwrap();
				field.field += Self::calc_field_at(charge, &pos2, &pos1);
			}
			modified_fields.push(field);
		}

		let mut i = 0;
		for field in em.get_components_mut::<C_Field>() {
			*field = modified_fields[i];
			i += 1;
		}
	}
}

fn normalized_vector(vx: f32, vy: f32) -> (f32, f32) {
	let intensity = (vx * vx + vy * vy).sqrt();
	(vx / intensity, vy /intensity)
}

fn random_normalized_vector() -> (f32, f32) {
	normalized_vector(rand::random(), rand::random())
}

impl S_Calculate_Field {
	fn calc_field_at(charge: Charge, charge_pos: &C_Position2D, target_pos: &C_Position2D) -> Force2D {
		let intensity = charge.0 / 0.01f32.max(
			(charge_pos.x - target_pos.x).powi(2) + (charge_pos.y - target_pos.y).powi(2)
		);
		let direction = if target_pos == charge_pos {
			random_normalized_vector()
		} else {
			normalized_vector(target_pos.x - charge_pos.x, target_pos.y - charge_pos.y)
		};
		let force = Force2D {
			x: direction.0 * intensity,
			y: direction.1 * intensity
		};
		assert!(!force.x.is_nan());
		assert!(!force.y.is_nan());
		force
	}
}

pub struct S_Move_Particles {
	world_size: sfsys::Vector2f
}

impl S_Move_Particles {
	pub fn new(world_size: &sfsys::Vector2f) -> S_Move_Particles {
		S_Move_Particles { world_size: world_size.clone() }
	}
}

impl System for S_Move_Particles {
	fn update(&mut self, dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>) {
		let filtered: Vec<Entity> = entities.into_iter().filter(|&&e|
			em.has_component::<C_Position2D>(e) &&
			em.has_component::<C_Charge>(e) &&
			em.has_component::<C_Field>(e)
		).cloned().collect();

		for e in filtered {
			let field = em.get_component::<C_Field>(e).unwrap().field;
			let charge = em.get_component::<C_Charge>(e).unwrap().charge;
			let mut pos = em.get_component_mut::<C_Position2D>(e).unwrap();
			let Force2D { x: dx, y: dy } = field * charge.0 * dt;
			pos.x += dx;
			if pos.x > self.world_size.x { pos.x = 0f32; }
			pos.y += dy;
			if pos.y > self.world_size.y { pos.y = 0f32; }
		}
	}
}

pub struct S_Particle_Draw_Gfx {
	pub win_size: sfsys::Vector2u,
	pub point_width: f32,

	vertices: Vec<sfgfx::Vertex>,
}

impl System for S_Particle_Draw_Gfx {
	fn update(&mut self, _dt: f32, em: &mut Entity_Manager, _entities: &Vec<Entity>) {
		self.vertices.clear();

		let pos_comps = em.get_components::<C_Position2D>();

		self.vertices.reserve(3 * pos_comps.len()); // 3x because we're drawing triangles
		for c_pos in pos_comps.iter() {
			self.add_particle_to_draw(c_pos);
		}

		assert_eq!(pos_comps.len(), em.get_components::<C_Charge>().len());

		let mut i = 0;
		for &C_Charge{ charge } in em.get_components::<C_Charge>() {
			for j in 0..3 {
				self.vertices[i + j].color = if charge.0 > 0f32 {
					sfgfx::Color::RED
				} else {
					sfgfx::Color::BLUE
				};
			}
			i += 3;
		}
	}
}

impl S_Particle_Draw_Gfx {
	pub fn new(winsize: &sfsys::Vector2u) -> S_Particle_Draw_Gfx {
		let win_size = *winsize;
		S_Particle_Draw_Gfx {
			win_size,
			point_width: 5f32,
			vertices: Vec::new(),
		}
	}

	fn add_particle_to_draw(&mut self, pos: &C_Position2D) {
		let hpw = self.point_width * 0.5;

		self.vertices.push(sfgfx::Vertex::new(
			sfsys::Vector2f::new(pos.x - hpw, pos.y - hpw),
			sfgfx::Color::RED,
			sfsys::Vector2f::new(0., 0.)
		));
		self.vertices.push(sfgfx::Vertex::new(
			sfsys::Vector2f::new(pos.x + hpw, pos.y - hpw),
			sfgfx::Color::RED,
			sfsys::Vector2f::new(0., 0.)
		));
		self.vertices.push(sfgfx::Vertex::new(
			sfsys::Vector2f::new(pos.x, pos.y + hpw),
			sfgfx::Color::RED,
			sfsys::Vector2f::new(0., 0.)
		));
	}
}

impl sfgfx::Drawable for S_Particle_Draw_Gfx {
	fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
	    &'a self,
	    target: &mut RenderTarget,
	    states: sfgfx::RenderStates<'texture, 'shader, 'shader_texture>
	) {
		let mut vertex_array = sfgfx::VertexArray::new(
			sfgfx::PrimitiveType::Triangles,
			self.vertices.len()
		);
		for i in 0..self.vertices.len() {
			vertex_array[i] = self.vertices[i];
		}
		target.draw_with_renderstates(&vertex_array, states);
	}
}


