use std::vec::Vec;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Generational_Index {
    pub index: usize,
    pub gen: u64,
}

// Generational_Allocator provides an interface to allocate/deallocate
// Generational Indices and check if an index is valid.
// The allocator is given an initial size and grows automatically when
// more indices than initially available are requested.
// Deallocated slots are reused whenever possible.
pub struct Generational_Allocator {
    // true if i-th slot is in use, false otherwise
    alive: Vec<bool>,
    // generation of i-th slot
    gens: Vec<u64>,
    // list of currently free slots. Used to retrieve the next available slot in O(1).
    free_slots: Vec<usize>,
}

impl Generational_Allocator {
    pub fn new(initial_size: usize) -> Generational_Allocator {
        let mut alloc = Generational_Allocator {
            alive: Vec::new(),
            gens: Vec::new(),
            free_slots: Vec::new(),
        };
        alloc.alive.resize(initial_size, false);
        alloc.gens.resize(initial_size, 0);
        alloc.free_slots = (0..initial_size).rev().collect();

        return alloc;
    }

    // Note: this returns the size of internal arrays, not the number of LIVE entities.
    pub fn size(&self) -> usize {
        self.alive.len()
    }

    pub fn live_size(&self) -> usize {
        self.alive
            .iter()
            .fold(0, |acc, &e| if e { acc + 1 } else { acc })
    }

    pub fn allocate(&mut self) -> Generational_Index {
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

        Generational_Index {
            index: i,
            gen: self.gens[i],
        }
    }

    // @return either a valid index inside `slots` or `self.alive.len()` if all are occupied.
    fn first_free_slot(&mut self) -> usize {
        match self.free_slots.pop() {
            Some(slot) => slot,
            None => self.alive.len(),
        }
    }

    pub fn deallocate(&mut self, idx: Generational_Index) {
        if idx.index >= self.alive.len() {
            panic!(
                "Tried to deallocate a Generational_Index whose index is greater than biggest one!"
            );
        }
        if self.gens[idx.index] > idx.gen {
            panic!("Tried to deallocate an old Generational_Index! Double free?");
        }
        if self.gens[idx.index] < idx.gen {
            panic!(
                "Tried to deallocate a Generational_Index with a generation greater than current!"
            );
        }
        if !self.alive[idx.index] {
            panic!("Tried to deallocate a Generational_Index that is not allocated! Double free?");
        }
        self.alive[idx.index] = false;
        self.free_slots.push(idx.index);
    }

    pub fn is_valid(&self, idx: Generational_Index) -> bool {
        (idx.index < self.alive.len()) && (idx.gen == self.gens[idx.index]) && self.alive[idx.index]
    }
}

#[cfg(test)]
mod tests_gen_allocator {
    use super::*;

    fn assert_invariant_free_slots_alive(alloc: &Generational_Allocator) {
        for free in &alloc.free_slots {
            assert!(
                !alloc.alive[*free],
                "Slot {} should not be alive but is!",
                *free
            );
        }
        for i in 0..alloc.alive.len() {
            if !alloc.alive[i] {
                assert!(
                    alloc.free_slots.contains(&i),
                    "Slot {} is not alive but is not in free_slots!",
                    i
                );
            }
        }
    }

    #[test]
    fn test_gen_alloc_create() {
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

        for i in 0..2 * n {
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

        let mut v: Vec<Generational_Index> = Vec::new();
        for i in 0..n {
            let i1 = alloc.allocate();
            v.push(i1);
            assert!(i1.index == i);
            assert!(i1.gen == 1);
            assert_invariant_free_slots_alive(&alloc);
        }

        for idx in v {
            alloc.deallocate(idx);
            assert_invariant_free_slots_alive(&alloc);
        }
    }

    #[test]
    #[should_panic(
        expected = "Tried to deallocate a Generational_Index whose index is greater than biggest one!"
    )]
    fn test_gen_alloc_bad_deallocate_1() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.deallocate(Generational_Index { index: 11, gen: 0 });
    }

    #[test]
    #[should_panic(expected = "Tried to deallocate an old Generational_Index! Double free?")]
    fn test_gen_alloc_bad_deallocate_2() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        let e1 = alloc.allocate();
        alloc.deallocate(e1);
        alloc.allocate();
        alloc.deallocate(e1);
    }

    #[test]
    #[should_panic(
        expected = "Tried to deallocate a Generational_Index with a generation greater than current!"
    )]
    fn test_gen_alloc_bad_deallocate_3() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.allocate();
        alloc.deallocate(Generational_Index { index: 0, gen: 2 });
    }

    #[test]
    #[should_panic(
        expected = "Tried to deallocate a Generational_Index that is not allocated! Double free?"
    )]
    fn test_gen_alloc_bad_deallocate_4() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.deallocate(Generational_Index { index: 0, gen: 0 });
    }

    #[test]
    fn test_reuse_empty_slot() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        let e1 = alloc.allocate();
        alloc.allocate();
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

        let _v: Vec<Generational_Index> = Vec::new();
        for _i in 0..3 * n {
            alloc.allocate();
            assert_invariant_free_slots_alive(&alloc);
        }
    }
}
