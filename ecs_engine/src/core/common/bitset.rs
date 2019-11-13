use std::vec::Vec;

#[derive(Clone, Default)]
pub struct Bit_Set {
    fast_bits: u64,
    slow_bits: Vec<u64>,
}

impl Bit_Set {
    pub fn set(&mut self, index: usize, value: bool) {
        let element_idx = index / 64;
        if element_idx == 0 {
            // fast bit
            if value {
                self.fast_bits |= 1 << index;
            } else {
                self.fast_bits &= !(1 << index);
            }
        } else {
            // slow bit
            let size_diff = ((element_idx - 1) - self.slow_bits.len()).max(0);
            for _ in 0..size_diff {
                self.slow_bits.push(0);
            }

            let element_idx = element_idx - 1;
            let bit_offset = index % 64;

            if value {
                self.slow_bits[element_idx] |= 1 << bit_offset;
            } else {
                self.slow_bits[element_idx] &= !(1 << bit_offset);
            }
        }
    }

    pub fn get(&self, index: usize) -> bool {
        let element_idx = index / 64;
        if element_idx == 0 {
            (self.fast_bits & (1 << index)) != 0
        } else if self.slow_bits.len() < element_idx {
            false
        } else {
            (self.slow_bits[element_idx - 1] & (1 << (index % 64))) != 0
        }
    }
}
