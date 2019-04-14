use std::cmp::Ordering;

use crate::limb::{*};

pub trait Pod {
    fn limbs(&self) -> BigSize;
    fn get_limb(&self, i: BigSize) -> Limb;
}

pub fn cmp_pod(lhs: &Pod, rhs: &Pod) -> Ordering {
    if lhs.limbs() > rhs.limbs() {
        for i in (0..lhs.limbs()).rev() {
            let lhsi = lhs.get_limb(i);
            let rhsi : Limb;
            if i < rhs.limbs() {
                rhsi = rhs.get_limb(i);
            } else {
                rhsi = 0;
            }
            if lhsi > rhsi {
                return Ordering::Greater;
            } else if lhsi < rhsi {
                return Ordering::Less;
            }
        }
    } else {
        for i in (0..rhs.limbs()).rev() {
            let rhsi = rhs.get_limb(i);
            let lhsi : Limb;
            if i < lhs.limbs() {
                lhsi = lhs.get_limb(i);
            } else {
                lhsi = 0;
            }
            if lhsi > rhsi {
                return Ordering::Greater;
            } else if lhsi < rhsi {
                return Ordering::Less;
            }
        }
    }
    return Ordering::Equal;
}


