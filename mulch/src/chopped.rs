#![warn(rust_2018_idioms)]

use std::fmt;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};

pub struct Chopped<T: Pod> {
    u: T,
    start: BigSize,
    length: BigSize
}

impl<T: Pod> Chopped<T> {
    pub fn chop(v: T, start: BigSize, length: BigSize) -> Chopped<T> {
        Chopped {u: v, start: start, length: length}
    }
}

impl<T: Pod> Pod for Chopped<T> {
    fn limbs(&self) -> BigSize {
        div_up(self.length, LIMB_SIZE)
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        /* I would like to use std::ops::Index but it requires we return
         * a reference and I can't create a limb and then return it as a
         * a reference */
//         println!("i: {}", i);
        let sz = div_up(self.length, LIMB_SIZE);
        if i >= sz {
            panic!("Attempted to index past the end of chop: sz is {} but index is {}", sz, i);
        }
        let src_limb_start = self.start / LIMB_SIZE;
        let src_bit_start = self.start % LIMB_SIZE;
        let src_lower_bits = LIMB_SIZE - src_bit_start;
        let src_upper_bits = src_bit_start;
        // we need a total of LIMB_SIZE bits for each limb
        // this is like a shift left
        // the lower destination limb bits come from 
        // the upper LIMB_SIZE - start source limb bits
        let dst_lower = self.u.get_limb(src_limb_start + i) >> src_upper_bits;
        let dst_upper;
        let over = src_limb_start + i + 1 >= self.u.limbs();
        if src_lower_bits < 64 && !over {
            dst_upper = self.u.get_limb(src_limb_start + i + 1)
            << src_lower_bits;
        } else {
            dst_upper = 0;
        }
        let r = dst_lower | dst_upper;
        let last = i == sz - 1;
        if last {
            // we need to zero some top bits
            let last_bits = self.length % LIMB_SIZE;
            if last_bits != 0 {
                let zero_bits = LIMB_SIZE - last_bits;
                return (r << zero_bits) >> zero_bits;
            } else {
                // actually we don't because this last limb is a whole limb
                return r;
            }
        } else {
            return r;
        }
    }
}

pod_eq! {
    lifetime 'a;
    Chopped<Vast<'a>>;
}

pub struct Shifted<T: Pod> {
    u: T,
    shl: isize,
}

impl<T: Pod> Shifted<T> {
    pub fn shl(v: T, shift: isize) -> Shifted<T> {
        Shifted {u: v, shl: shift}
    }
}

impl<T: Pod> Pod for Shifted<T> {
    fn limbs(&self) -> BigSize {
        div_up(self.u.limbs()*LIMB_SIZE + self.shl, LIMB_SIZE)
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        let n = self.shl;
        let n_limbs = n / LIMB_SIZE;
        let n_bits = n - (n_limbs * LIMB_SIZE);
        let sz = self.u.limbs();
        if n_limbs <= i && i <= (n_limbs+sz) {
            let src_lower = (i-n_limbs)-1;
            let src_upper = i-n_limbs;
            // we need a total of LIMB_SIZE bits for each limb
            // the upper LIMB_SIZE - n_bits of the destination comes
            // from the lower LIMB_SIZE - n_bits of the upper source
            let upper : Limb;
            let lower : Limb;
            if src_lower < 0 || n_bits == 0 {
                lower = 0;
            } else {
                // the lower n_bits of the destination comes
                // from the upper n_bits of the source
                // so we discard LIMB_SIZE - n_bits of the lower source
                lower = self.u.get_limb(src_lower) >> (LIMB_SIZE - n_bits);
            }
            if src_upper >= sz {
                upper = 0;
            } else {
                upper = self.u.get_limb(src_upper) << n_bits;
            }
            return upper | lower;
        } else if i < n_limbs {
            return 0;
        } else if i > n_limbs+sz {
            return 0;
        };
        unreachable!();
    }
}


pod_eq! {
    lifetime 'a;
    Shifted<Vast<'a>>;
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    use crate::big::{*};
    #[test]
    fn shifted_() {
        let ab = Big::from_hex("2F6DC70EE58ED84B800000000000000000");
        let a = Vast::from(&ab);
        let r = Shifted::shl(a, 60);
        println!("{:?}", r);
        assert_eq!(r.to_hex(), 
            "2F6DC70EE58ED84B800000000000000000000000000000000"
        );
    }
}




