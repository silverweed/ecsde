mod entity_manager;
mod components;

use entity_manager::*;
use components::*;

fn main() {
	let mut em = Entity_Manager::new();

	em.register_component::<C_Position>();

	let mut entities : Vec<Entity> = Vec::new();
	for i in 0..10 {
		let e = em.new_entity();
		entities.push(e);
		println!("{:?}", e);
	}

	em.add_component::<C_Position>(entities[0]);

	println!("{:?}", em.get_component::<C_Position>(entities[0]));
	println!("{:?}", em.get_component::<C_Position>(entities[1]));

	{
		let mut pos1 = em.get_component_mut::<C_Position>(entities[0]).unwrap();
		pos1.x = 42f32;
		pos1.y = 64f32;
	}

	println!("{:?}", em.get_component::<C_Position>(entities[0]));

	{
		let pos2 = em.get_component::<C_Position>(entities[2]);
		println!("{:?}", pos2);
	}

	/*em.destroy_entity(entities[4]);*/
	{
		let pos3 = em.get_component::<C_Position>(entities[4]);
		println!("{:?}", pos3);
	}
}
