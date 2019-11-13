use std::vec::Vec;

pub type Index_Type = usize;
pub type Gen_Type = u32;

#[derive(Default, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub struct Generational_Index {
    pub index: Index_Type,
    pub gen: Gen_Type,
}

impl Generational_Index {
    pub const INVALID: Generational_Index = Generational_Index { index: 0, gen: 0 };
}

/// Generational_Allocator allows to allocate/deallocate
/// Generational Indices and check if an index is valid.
/// The allocator is given an initial size and grows automatically when
/// more indices than initially available are requested.
/// Deallocated slots are reused whenever possible.
pub struct Generational_Allocator {
    // generation of i-th slot
    gens: Vec<Gen_Type>,
    // list of currently free slots. Used to retrieve the next available slot in O(1).
    free_slots: Vec<usize>,
}

impl Generational_Allocator {
    pub fn new(initial_size: usize) -> Generational_Allocator {
        let mut alloc = Generational_Allocator {
            gens: vec![],
            free_slots: vec![],
        };
        // Start from gen -1 so we can use { 0, 0 } as "the invalid index" (and can e.g. detect invalid deallocations).
        alloc.gens.resize(initial_size, 1);
        alloc.free_slots = (0..initial_size).rev().collect();

        alloc
    }

    /// Returns the maximum number of generational indices that can be allocated before resizing.
    pub fn capacity(&self) -> usize {
        self.gens.len()
    }

    pub fn live_size(&self) -> usize {
        self.gens.len() - self.free_slots.len()
    }

    pub fn allocate(&mut self) -> Generational_Index {
        let i = self.first_free_slot();
        let cur_size = self.gens.len();
        if i == cur_size {
            // Grow the vectors (exponentially)
            let new_size = cur_size * 2;
            self.gens.resize(new_size, 1);
            self.free_slots.reserve(new_size);
            for i in (cur_size + 1..new_size).rev() {
                self.free_slots.push(i);
            }
        }

        let gen = &mut self.gens[i];

        Generational_Index {
            index: i,
            gen: *gen,
        }
    }

    /// Returns either a valid index inside `slots` or the total slot number if all are occupied.
    #[inline]
    fn first_free_slot(&mut self) -> usize {
        match self.free_slots.pop() {
            Some(slot) => slot,
            None => self.gens.len(),
        }
    }

    pub fn deallocate(&mut self, idx: Generational_Index) {
        #[cfg(debug_assertions)]
        {
            if idx.index >= self.gens.len() {
                panic!("Tried to deallocate a Generational_Index whose index is greater than biggest one!");
            }
            if self.gens[idx.index] > idx.gen {
                panic!("Tried to deallocate an old Generational_Index! Double free?");
            }
            if self.gens[idx.index] < idx.gen {
                panic!("Tried to deallocate a Generational_Index with a generation greater than current!");
            }
        }
        self.gens[idx.index] += 1;
        self.free_slots.push(idx.index);
    }

    // Note: currently this function is O(n). This is a tradeoff for not keeping an "alive" array in the
    // allocator. This is probably reasonable, as we don't expect to call is_valid() very often.
    // Should that not prove to be the case, consider accelerating this function somehow (e.g. with the alive array).
    pub fn is_valid(&self, idx: Generational_Index) -> bool {
        (idx.index < self.gens.len())
            && (idx.gen == self.gens[idx.index])
            && !self.free_slots.contains(&idx.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_alloc_create() {
        let n = 10;
        let alloc = Generational_Allocator::new(n);
        assert_eq!(alloc.gens.len(), n);
        assert_eq!(alloc.free_slots.len(), n);
    }

    #[test]
    fn gen_alloc_allocate() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);

        for i in 0..2 * n {
            let i1 = alloc.allocate();
            assert!(i1.index == i, "Index should be {} but is {}!", i, i1.index);
            assert!(i1.gen == 1);
        }
    }

    #[test]
    fn gen_alloc_deallocate() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);

        let mut v: Vec<Generational_Index> = Vec::new();
        for i in 0..n {
            let i1 = alloc.allocate();
            v.push(i1);
            assert!(i1.index == i);
            assert!(i1.gen == 1);
        }

        for idx in v {
            alloc.deallocate(idx);
        }
    }

    #[test]
    #[should_panic(
        expected = "Tried to deallocate a Generational_Index whose index is greater than biggest one!"
    )]
    fn gen_alloc_bad_deallocate_1() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.deallocate(Generational_Index { index: 11, gen: 0 });
    }

    #[test]
    #[should_panic(expected = "Tried to deallocate an old Generational_Index! Double free?")]
    fn gen_alloc_bad_deallocate_2() {
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
    fn gen_alloc_bad_deallocate_3() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.allocate();
        alloc.deallocate(Generational_Index { index: 0, gen: 2 });
    }

    #[test]
    #[should_panic(expected = "Tried to deallocate an old Generational_Index! Double free?")]
    fn gen_alloc_bad_deallocate_4() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        alloc.deallocate(Generational_Index { index: 0, gen: 0 });
    }

    #[test]
    fn gen_alloc_reuse_empty_slot() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);
        let e1 = alloc.allocate();
        alloc.allocate();
        alloc.deallocate(e1);
        let e3 = alloc.allocate();
        assert_eq!(e3.index, 0);
        assert_eq!(e3.gen, 2);
        alloc.deallocate(e3);
        let e4 = alloc.allocate();
        assert_eq!(e4.index, 0);
        assert_eq!(e4.gen, 3);
    }

    #[test]
    fn gen_alloc_allocate_past_initial_size() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);

        for _i in 0..3 * n {
            alloc.allocate();
        }
    }

    #[test]
    fn gen_alloc_is_valid() {
        let n = 10;
        let mut alloc = Generational_Allocator::new(n);

        let e1 = alloc.allocate();
        assert!(alloc.is_valid(e1));

        alloc.deallocate(e1);
        assert!(!alloc.is_valid(e1));

        let e2 = alloc.allocate();
        assert!(!alloc.is_valid(e1));
        assert!(alloc.is_valid(e2));

        let e3 = alloc.allocate();
        assert!(!alloc.is_valid(e1));
        assert!(alloc.is_valid(e2));
        assert!(alloc.is_valid(e3));

        alloc.deallocate(e3);
        assert!(!alloc.is_valid(e1));
        assert!(alloc.is_valid(e2));
        assert!(!alloc.is_valid(e3));

        alloc.deallocate(e2);
        assert!(!alloc.is_valid(e1));
        assert!(!alloc.is_valid(e2));
        assert!(!alloc.is_valid(e3));
    }

    #[test]
    fn gen_alloc_invalid_is_invalid() {
        let alloc = Generational_Allocator::new(4);
        assert!(!alloc.is_valid(Generational_Index::INVALID));
    }

    #[test]
    fn gen_alloc_uncreated_entity_is_invalid() {
        let alloc = Generational_Allocator::new(4);
        assert!(!alloc.is_valid(Generational_Index { index: 0, gen: 1 }));
    }
}
