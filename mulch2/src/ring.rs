#![warn(rust_2018_idioms)]
#![allow(unused)]

use std::ptr;
use std::mem;
use crate::limb::*;

pub struct Ring {
    n: usize,
    sz: usize,
    b: *mut Limb,
    limbs: usize,
    total_limbs: usize,
}

pub struct Residue {
    b: *mut Limb
}

impl Ring {
    pub fn new(n: usize, sz: usize) -> Ring {
        assert!(n % LIMB_BITS == 0);
        let limbs_per_residue = n/LIMB_BITS+1;
        let total_limbs = limbs_per_residue * sz;
        let mut new_v: Vec<Limb> = std::vec::from_elem(0, total_limbs);
        let p = new_v.as_mut_ptr();
        let len = new_v.len();
        let cap = new_v.capacity();
        assert_eq!(cap, len);
        unsafe {
            mem::forget(new_v);
        }
        return Ring { 
            n: n, 
            sz: sz, 
            b: p,
            limbs: limbs_per_residue,
            total_limbs: total_limbs,
        };
    }
    fn get_residue(&self, i: usize) -> Residue {
        #[cfg(debug_assertions)]
        { if i >= self.sz {
            panic!("Tried to get residue {} but only have {} residues",
                i, self.sz
            );
        } }
        let o = i * self.limbs;
        unsafe {
            return Residue { self.b.add(o) }
        }
    }
}

impl Drop for Ring {
    fn drop(&mut self) {
        println!("Ring {:?} dropped", self.b);
        unsafe {
            let rebuilt = Vec::from_raw_parts(
                self.b, self.total_limbs, self.total_limbs);
            mem::drop(rebuilt);
        }
    }
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    #[test]
    fn new_ring() {
        let r = Ring::new(64, 2);
    }
}


