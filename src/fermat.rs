#![allow(unused)]

use std::cmp::Ordering;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::svast::{*};
use crate::chopped::{*};

#[derive(Clone,Copy,Debug)]
pub struct Fermat {
    pub n: BigSize
}

impl Fermat {
    pub fn new(n: BigSize) -> Fermat {
        Fermat {n: n}
    }
}

impl Pod for Fermat {
    fn limbs(&self) -> BigSize {
        let sz = div_up(self.n+1, LIMB_SIZE);
        return sz;
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        let n = self.n;
        let sz = div_up(n+1, LIMB_SIZE);
        let bit = n % LIMB_SIZE;
        let limb = n / LIMB_SIZE;
        let mut l: Limb = 0;
        if i == 0 {
            l |= 1;
        }
        if i == limb {
            l |= 1 << bit;
        }
        return l;
    }
}

pub trait FermatOps {
//     fn add_fermat(&mut self, n: BigSize);
    fn assign_mod_fermat(&mut self, src: &Vast, f: Fermat);
}

impl<'a> FermatOps for SVastMut<'a> {
    fn assign_mod_fermat(&mut self, src: &Vast, f: Fermat) {
        self.zero();
        let sz = f.limbs();
        let mut mod_f = self;
        let src_bits = src.bits();
        let iters = div_up(src_bits, f.n);
        for i in 0..iters {
            let chunk: BigSize;
            if (f.n*i + f.n) > src_bits {
                chunk = src_bits - f.n*i;
            } else {
                chunk = f.n;
            }
            if chunk == 0 {
                break;
            }
            let piece = Chopped::chop(src.clone(), f.n*i, chunk);
            if i % 2 == 0 {
                mod_f.pod_add_assign(&piece);
            } else {
                mod_f.pod_sub_assign(&piece);
            }
        }
        if mod_f.negative {
            mod_f.pod_add_assign(&f);
            if mod_f.negative {
                panic!("Still negative!");
            }
        } else {
            if mod_f.pod_cmp(&f) != Ordering::Less {
                mod_f.pod_sub_assign(&f);
                if mod_f.pod_cmp(&f) != Ordering::Less {
                    panic!("Still too big!")
                }
            }
        }
    }
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::fermat::{*};
    #[test]
    fn fermat_1() {
        let f = Fermat::new(64);
        assert_eq!(f.to_hex(),"10000000000000001");
        let f = Fermat::new(32);
        assert_eq!(f.to_hex(),"100000001");
    }
}
