// use std::ops::Index;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};

pub struct Chopped<'a> {
    u: Vast<'a>,
    start: BigSize,
    length: BigSize
}

impl<'a> Chopped<'a> {
    pub fn chop(v: Vast<'a>, start: BigSize, length: BigSize) -> Chopped<'a> {
        Chopped {u: v, start: start, length: length}
    }
}

impl<'a> Pod for Chopped<'a> {
    fn limbs(&self) -> BigSize {
        div_up(self.length, LIMB_SIZE)
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        /* I would like to use std::ops::Index but it requires we return
         * a reference and I can't create a limb and then return it as a
         * a reference */
        let sz = div_up(self.length, LIMB_SIZE);
        if i >= sz {
            panic!("Attempted to index past the end of chop: sz is {} but index is {}", sz, i);
        }
        let mut r: Limb = 0;
        let src_limb_start = self.start / LIMB_SIZE;
        let src_bit_start = self.start % LIMB_SIZE;
        let src_lower_bits = LIMB_SIZE - src_bit_start;
        let src_upper_bits = src_bit_start;
        let last = sz - 1 + src_limb_start;
        let over = last >= self.u.length();
        let last_r_bits = (self.start + self.length - 1) % LIMB_SIZE + 1;
        let last_special = 
            self.length % LIMB_SIZE > 0 
            && (!over) 
            && last_r_bits > 0;
        let is_last = i == sz-1;
        if is_last && last_special {
            let shake_l = LIMB_SIZE - last_r_bits;
            let shake_r = LIMB_SIZE - self.length % LIMB_SIZE;
//             println!("shake_l: {} shake_r: {}", shake_l, shake_r);
            let last_r = (self.u[last] << shake_l) >> shake_r;
            return last_r;
        } else {
            // we need a total of LIMB_SIZE bits for each limb
            // this is like a shift left
            // the lower destination limb bits come from 
            // the upper LIMB_SIZE - start source limb bits
            let dst_lower = self.u[src_limb_start + i] >> src_upper_bits;
            let dst_upper;
            let over = src_limb_start + i + 1 >= self.u.length();
            if src_lower_bits < 64 && !over {
                dst_upper = self.u[src_limb_start + i + 1]
                << src_lower_bits;
            } else {
                dst_upper = 0;
            }
            let r = dst_lower | dst_upper;
            return r;
        }
    }
    fn min_limbs(&self) -> BigSize {
        for i in (0..self.limbs()).rev() {
            if self.get_limb(i) != 0 {
                return (i + 1) as BigSize;
            }
        }
        return 0;
    }
}

