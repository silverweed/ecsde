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
	// list of currently free slots
	free_slots: Vec<usize>,
}

impl Generational_Allocator {
	fn new(initial_size: usize) -> Generational_Allocator {
		let mut alloc = Generational_Allocator {
			alive: Vec::new(),
			gens: Vec::new(),
			free_slots: Vec::new(),
		};
		alloc.alive.resize(initial_size, false);
		alloc.gens.resize(initial_size, 0);
		alloc.free_slots.reserve(initial_size);
		for i in (0..initial_size).rev() {
			alloc.free_slots.push(i);
		}

		return alloc;
	}

	fn size(&self) -> usize {
		return self.alive.len();
	}

	fn allocate(&mut self) -> Generational_Index {
		let i = self.first_free_slot();
		if i == self.alive.len() {
			// Grow the vectors
			let oldsize = self.alive.len();
			let newsize = self.alive.len() * 2;
			self.alive.resize(newsize, false);
			self.gens.resize(newsize, 0);
			self.free_slots.reserve(newsize);
			for i in (oldsize + 1..newsize).rev() {
				self.free_slots.push(i);
			}
		}

		self.alive[i] = true;
		self.gens[i] += 1;

		return Generational_Index {
			index: i,
			gen: self.gens[i],
		};
	}

	// @return either a valid index inside `slots` or `self.alive.len()` if all are occupied.
	fn first_free_slot(&mut self) -> usize {
		match self.free_slots.pop() {
			Some(slot) => slot,
			None => self.alive.len(),
		}
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
		self.free_slots.push(idx.index);
	}
}

#[cfg(test)]
mod tests_gen_allocator {
	use super::*;

	fn assert_invariant_free_slots_alive(alloc : &Generational_Allocator) {
		for free in &alloc.free_slots {
			assert!(!alloc.alive[*free], "Slot {} should not be alive but is!", *free);
		}
	}

	#[test]
	fn test_create_gen_alloc() {
		let n = 10;
		let alloc = Generational_Allocator::new(n);
		assert_eq!(alloc.alive.len(), n);
		assert_eq!(alloc.gens.len(), n);
		assert_eq!(alloc.free_slots.len(), n);
		assert_invariant_free_slots_alive(&alloc);
	}

	#[test]
	fn test_gen_alloc_allocate() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);

		for i in 0..2*n {
			let i1 = alloc.allocate();
			assert!(i1.index == i, "Index should be {} but is {}!", i, i1.index);
			assert!(i1.gen == 1);
			assert_invariant_free_slots_alive(&alloc);
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
			assert_invariant_free_slots_alive(&alloc);
		}

		for i in 0..n {
			alloc.deallocate(v[i]);
			assert_invariant_free_slots_alive(&alloc);
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
		let _e2 = alloc.allocate();
		alloc.deallocate(e1);
	}

	#[test]
	#[should_panic(expected = "Tried to deallocate a Generational_Index with a generation greater than current!")]
	fn test_gen_alloc_bad_deallocate_3() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);
		let _e1 = alloc.allocate();
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
		let _e2 = alloc.allocate();
		alloc.deallocate(e1);
		assert_invariant_free_slots_alive(&alloc);
		let e3 = alloc.allocate();
		assert!(e3.index == 0 && e3.gen == 2);
		assert_invariant_free_slots_alive(&alloc);
	}

	#[test]
	fn test_gen_alloc_allocate_past_initial_size() {
		let n = 10;
		let mut alloc = Generational_Allocator::new(n);

		let _v : Vec<Generational_Index> = Vec::new();
		for _i in 0..3 * n {
			let _i1 = alloc.allocate();
			assert_invariant_free_slots_alive(&alloc);
		}
	}
}

pub struct Entity_Manager {
	allocator: Generational_Allocator,
	// { CompType => Vec<CompType> }
	components: AnyMap,
}

pub type Entity = Generational_Index;
type VecOpt<T> = Vec<Option<T>>;

impl Entity_Manager {

	pub fn new() -> Entity_Manager {
		return Entity_Manager {
			allocator: Generational_Allocator::new(4),
			components: AnyMap::new(),
		};
	}

	fn get_comp_storage<C: Component + 'static>(&self) -> Option<&VecOpt<C>> {
		self.components.get::<VecOpt<C>>()
	}

	fn get_mut_comp_storage<C: Component + 'static>(&mut self) -> Option<&mut VecOpt<C>> {
		self.components.get_mut::<VecOpt<C>>()
	}

	pub fn new_entity(&mut self) -> Entity {
		self.allocator.allocate()
	}

	pub fn is_valid_entity(&self, e: Entity) -> bool {
		let a = &self.allocator;
		return e.index < a.size() && e.gen == a.gens[e.index] && a.alive[e.index];
	}

	pub fn destroy_entity(&mut self, e: Entity) {
		self.allocator.deallocate(e);
	}

	pub fn register_component<C: Component + 'static>(&mut self) {
		if let Some(_) = self.get_comp_storage::<C>() {
			panic!("Tried to register the same component {} twice!", C::type_name());
		}
		let v: VecOpt<C> = Vec::new();
		self.components.insert(v);
	}

	// Adds a component of type C to `e`.
	pub fn add_component<C: Component + 'static>(&mut self, e: Entity) {
		if !self.is_valid_entity(e) {
			panic!("Tried to add component to invalid entity {:?}", e);
		}

		let alloc_size = self.allocator.size();
		match self.get_mut_comp_storage::<C>() {
			Some(vec) => {
				vec.resize(alloc_size, None);
				let mut c = C::default();
				vec[e.index] = Some(c);
			},
			None => panic!("Tried to add unregistered component {} to entity!", C::type_name()),
		}
	}

	pub fn get_component<'a, C: Component + 'static>(&'a self, e: Entity) -> Option<&'a C> {
		if !self.is_valid_entity(e) {
			panic!("Tried to get component of invalid entity {:?}", e);
		}

		match self.get_comp_storage::<C>() {
			Some(vec) => {
				// Note: we may not have added any component yet, so the components Vec is of len 0
				if e.index < vec.len() { return vec[e.index].as_ref(); }
				return None;
			},
			None => panic!("Tried to get unregistered component {}!", C::type_name()),
		}
	}

	// @Refactoring: this code is almost exactly the same as `get_component`. Can we do something about it?
	pub fn get_component_mut<'a, C: Component + 'static>(&'a mut self, e: Entity) -> Option<&'a mut C> {
		if !self.is_valid_entity(e) {
			panic!("Tried to get component of invalid entity {:?}", e);
		}

		match self.get_mut_comp_storage::<C>() {
			Some(vec) => {
				if e.index < vec.len() { return vec[e.index].as_mut(); }
				return None;
			},
			None => panic!("Tried to get unregistered component {}!", C::type_name()),
		}
	}

	pub fn has_component<C: Component + 'static>(&self, e: Entity) -> bool {
		self.get_component::<C>(e).is_some()
	}
}

#[cfg(test)]
mod tests_entity_manager {
	use super::*;

	#[derive(Copy, Clone, Debug, Default, TypeName)]
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
			let c = em.get_component_mut::<C_Test>(e).unwrap();
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
