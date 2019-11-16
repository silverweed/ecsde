use std::ops::BitAnd;
use std::vec::Vec;

#[derive(Clone, PartialEq, Default, Debug)]
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
            let bits_len = self.slow_bits.len();
            if element_idx > bits_len {
                let size_diff = ((element_idx - 1) - self.slow_bits.len()).max(0);
                for _ in 0..=size_diff {
                    self.slow_bits.push(0);
                }
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

impl BitAnd for &Bit_Set {
    type Output = Bit_Set;

    fn bitand(self, rhs: Self) -> Bit_Set {
        let mut res = Bit_Set::default();
        res.fast_bits = self.fast_bits & rhs.fast_bits;

        let my_size = self.slow_bits.len();
        let rhs_size = rhs.slow_bits.len();
        let min_size = my_size.min(rhs_size);
        let max_size = my_size.max(rhs_size);

        res.slow_bits.resize(max_size, 0);

        for i in 0..min_size {
            res.slow_bits[i] = self.slow_bits[i] & rhs.slow_bits[i];
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset_default() {
        let a = Bit_Set::default();
        assert_eq!(a.get(0), false);
        assert_eq!(a.get(10), false);
        assert_eq!(a.get(100), false);
        assert_eq!(a.get(1000), false);
    }

    #[test]
    fn bitset_setget() {
        let mut a = Bit_Set::default();

        a.set(1, true);
        assert_eq!(a.get(1), true);

        a.set(1, false);
        assert_eq!(a.get(1), false);

        a.set(14, true);
        assert_eq!(a.get(14), true);

        a.set(14, false);
        assert_eq!(a.get(14), false);

        a.set(1001, true);
        assert_eq!(a.get(1001), true);

        a.set(1001, false);
        assert_eq!(a.get(1001), false);
    }

    #[test]
    fn bitset_bitand() {
        let mut a = Bit_Set::default();
        let mut b = Bit_Set::default();

        a.set(0, true);
        a.set(2, true);
        a.set(6, true);
        a.set(16, true);

        b.set(2, true);
        b.set(6, true);

        let c = &a & &b;
        assert_eq!(c.get(0), false);
        assert_eq!(c.get(2), true);
        assert_eq!(c.get(6), true);
        assert_eq!(c.get(8), false);
        assert_eq!(c.get(16), false);
    }

    #[test]
    fn bitseq_ref_equality() {
        let mut a = Bit_Set::default();
        let mut b = Bit_Set::default();

        a.set(2, true);
        a.set(100, true);
        b.set(2, true);
        b.set(100, true);
        assert_eq!(&a, &b);

        b.set(100, false);
        assert_ne!(&a, &b);
    }
}
