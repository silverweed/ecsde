extern crate anymap;

use self::anymap::AnyMap;
use std::vec::Vec;
use std::option::Option;
use components::Component;

#[derive(Copy, Clone, Debug)]
pub struct Generational_Index {
	pub index: usize,
	pub gen: u64
}

struct Generational_Allocator {
	// true if i-th slot is in use, false otherwise
	alive: Vec<bool>,
	// generation of i-th slot
	gens: Vec<u64>,
}

impl Generational_Allocator {
	fn new(initial_size: usize) -> Generational_Allocator {
		let mut alloc = Generational_Allocator {
			alive: Vec::new(),
			gens: Vec::new(),
		};
		alloc.alive.resize(initial_size, false);
		alloc.gens.resize(initial_size, 0);

		return alloc;
	}

	fn size(&self) -> usize {
		return self.alive.len();
	}

	fn allocate(&mut self) -> Generational_Index {
		let i = self.first_free_slot();
		if i == self.alive.len() {
			// Grow the vectors
			let size = self.alive.len() * 2;
			self.alive.resize(size, false);
			self.gens.resize(size, 0);
		}

		self.alive[i] = true;
		self.gens[i] += 1;

		return Generational_Index {
			index: i,
			gen: self.gens[i],
		};
	}

	// @return either a valid index inside `slots` or `self.alive.len()` if all are occupied.
	fn first_free_slot(&self) -> usize {
		let mut i = 0;
		while i < self.alive.len() {
			if !self.alive[i] { break; }
			i += 1;
		}
		return i;
	}

	fn deallocate(&mut self, idx: Generational_Index) {
		if idx.index >= self.alive.len() {
			panic!("Tried to deallocate a Generational_Index whose index is greater than biggest one!");
		}
		if self.gens[idx.index] > idx.gen {
			panic!("Tried to deallocate an old Generational_Index! Double free?");
		}
		if self.gens[idx.index] < idx.gen {
			panic!("Tried to deallocate a Generational_Index with a generation greater than current!");
		}
		if !self.alive[idx.index] {
			panic!("Tried to deallocate a Generational_Index that is not allocated! Double free?");
		}
		self.alive[idx.index] = false;
	}
}

#[cfg(test)]
mod tests_gen_allocator {
	use super::*;

	#[test]
	fn test_create_gen_alloc() {
		let n = 10;
		let alloc = Generational_Allocator::new(n);
		assert_eq!(alloc.alive.len(), n);
		assert_eq!(alloc.gens.len(), n);
	}

	#[test]
	fn test_gen_alloc_allocate() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);

		for i in 0..2*n {
			let i1 = alloc.allocate();
			assert!(i1.index == i);
			assert!(i1.gen == 1);
		}
	}

	#[test]
	fn test_gen_alloc_deallocate() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);

		let mut v : Vec<Generational_Index> = Vec::new();
		for i in 0..n {
			let i1 = alloc.allocate();
			v.push(i1);
			assert!(i1.index == i);
			assert!(i1.gen == 1);
		}

		for i in 0..n {
			alloc.deallocate(v[i]);
		}
	}

	#[test]
	#[should_panic(expected = "Tried to deallocate a Generational_Index whose index is greater than biggest one!")]
	fn test_gen_alloc_bad_deallocate_1() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		alloc.deallocate(Generational_Index{ index: 11, gen: 0 });
	}

	#[test]
	#[should_panic(expected = "Tried to deallocate an old Generational_Index! Double free?")]
	fn test_gen_alloc_bad_deallocate_2() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		let e1 = alloc.allocate();
		alloc.deallocate(e1);
		let e2 = alloc.allocate();
		alloc.deallocate(e1);
	}

	#[test]
	#[should_panic(expected = "Tried to deallocate a Generational_Index with a generation greater than current!")]
	fn test_gen_alloc_bad_deallocate_3() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		let e1 = alloc.allocate();
		alloc.deallocate(Generational_Index{ index: 0, gen: 2 });
	}

	#[test]
	#[should_panic(expected = "Tried to deallocate a Generational_Index that is not allocated! Double free?")]
	fn test_gen_alloc_bad_deallocate_4() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		alloc.deallocate(Generational_Index{ index: 0, gen: 0 });
	}

	#[test]
	fn test_reuse_empty_slot() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		let e1 = alloc.allocate();
		let e2 = alloc.allocate();
		alloc.deallocate(e1);
		let e3 = alloc.allocate();
		assert!(e3.index == 0 && e3.gen == 2);
	}

	#[test]
	fn test_gen_alloc_allocate_past_initial_size() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);

		let mut v : Vec<Generational_Index> = Vec::new();
		for i in 0..3 * n {
			let i1 = alloc.allocate();
		}
	}
}

pub struct Entity_Manager {
	allocator: Generational_Allocator,
	// { CompType => Vec<CompType> }
	components: AnyMap,
}

pub type Entity = Generational_Index;

impl Entity_Manager {
	pub fn new() -> Entity_Manager {
		return Entity_Manager {
			allocator: Generational_Allocator::new(4),
			components: AnyMap::new(),
		};
	}

	pub fn new_entity(&mut self) -> Entity {
		let entity = self.allocator.allocate();
		return entity;
	}

	pub fn is_valid_entity(&self, e: Entity) -> bool {
		let a = &self.allocator;
		return e.index < a.size() && e.gen == a.gens[e.index] && a.alive[e.index];
	}

	pub fn destroy_entity(&mut self, e: Entity) {
		self.allocator.deallocate(e);
	}

	pub fn register_component<C: Component + 'static>(&mut self) {
		if let Some(_) = self.components.get::<Vec<Option<C>>>() {
			panic!("Tried to register the same component twice!"); // @Clarity: add component type name to err msg
		}
		let mut v: Vec<Option<C>> = Vec::new();
		self.components.insert(v);
	}

	// Adds a component of type C to `e`.
	pub fn add_component<C: Component + 'static>(&mut self, e: Entity) {
		if !self.is_valid_entity(e) {
			panic!("Tried to add component to invalid entity {:?}", e);
		}

		match self.components.get_mut::<Vec<Option<C>>>() {
			Some(vec) => {
				vec.resize(self.allocator.size(), None);
				let mut c = C::default();
				vec[e.index] = Some(c);
			},
			None => panic!("Tried to add unregistered component to entity!"), // @Clarity: add component type name to err msg
		}
	}

	pub fn get_component<'a, C: Component + 'static>(&'a self, e: Entity) -> Option<&'a C> {
		if !self.is_valid_entity(e) {
			panic!("Tried to get component of invalid entity {:?}", e);
		}

		match self.components.get::<Vec<Option<C>>>() {
			Some(vec) => {
				// Note: we may not have added any component yet, so the components Vec is of len 0
				if e.index < vec.len() { return vec[e.index].as_ref(); }
				return None;
			},
			None => panic!("Tried to get unregistered component!"), // @Clarity: add component type name to err msg
		}
	}

	// @Refactoring: this code is almost exactly the same as `get_component`. Can we do something about it?
	pub fn get_component_mut<'a, C: Component + 'static>(&'a mut self, e: Entity) -> Option<&'a mut C> {
		if !self.is_valid_entity(e) {
			panic!("Tried to get component of invalid entity {:?}", e);
		}

		match self.components.get_mut::<Vec<Option<C>>>() {
			Some(vec) => {
				if e.index < vec.len() { return vec[e.index].as_mut(); }
				return None;
			},
			None => panic!("Tried to get unregistered component!"), // @Clarity: add component type name to err msg
		}
	}
}

#[cfg(test)]
mod tests_entity_manager {
	use super::*;

	#[derive(Copy, Clone, Debug, Default)]
	struct C_Test {
		foo: i32
	}

	#[test]
	#[should_panic]
	fn test_register_same_component_twice() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();
		em.register_component::<C_Test>();
	}

	#[test]
	fn test_get_component() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();

		let e = em.new_entity();
		assert!(em.get_component::<C_Test>(e).is_none());

		em.add_component::<C_Test>(e);
		assert!(em.get_component::<C_Test>(e).is_some());
	}

	#[test]
	fn test_get_component_mut() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();

		let e = em.new_entity();
		assert!(em.get_component_mut::<C_Test>(e).is_none());

		em.add_component::<C_Test>(e);
		assert!(em.get_component_mut::<C_Test>(e).is_some());
	}

	#[test]
	fn test_mutate_component() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();

		let e = em.new_entity();

		em.add_component::<C_Test>(e);
		{
			let mut c = em.get_component_mut::<C_Test>(e).unwrap();
			c.foo = 4242;
		}
		assert!(em.get_component::<C_Test>(e).unwrap().foo == 4242);
	}

	#[test]
	#[should_panic]
	fn test_add_component_inexisting_entity() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();
		em.add_component::<C_Test>(Entity{ index: 0, gen: 1 });
	}

	#[test]
	#[should_panic]
	fn test_get_component_inexisting_entity() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();
		em.get_component::<C_Test>(Entity{ index: 0, gen: 1 });
	}

	#[test]
	#[should_panic]
	fn test_get_component_mut_inexisting_entity() {
		let mut em = Entity_Manager::new();
		em.register_component::<C_Test>();
		em.get_component_mut::<C_Test>(Entity{ index: 0, gen: 1 });
	}
}
