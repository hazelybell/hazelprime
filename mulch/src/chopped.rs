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

