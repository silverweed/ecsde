use entity_manager::{Entity, Entity_Manager};

pub trait System {
	fn update(&mut self, dt: f32, em: &mut Entity_Manager, entities: &Vec<Entity>);
}
