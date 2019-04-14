use std::cmp::Ordering;

use crate::limb::{*};

pub trait Pod {
    // Anything with limbs
    fn limbs(&self) -> BigSize;
    fn get_limb(&self, i: BigSize) -> Limb;
}

pub trait PodOps {
    fn bits(&self) -> BigSize;
    fn pod_eq(&self, other: &Pod) -> bool;
    fn min_limbs(&self) -> BigSize;
}

impl<T> PodOps for T where T: Pod {
    fn bits(&self) -> BigSize {
        let mut b : BigSize = (self.limbs()) * (LIMB_SHIFT as BigSize);
        for i in (0..self.limbs()).rev() {
            let l = self.get_limb(i);
            for j in (0..LIMB_SHIFT).rev() {
                let m = 1u64 << j;
                if (l & m) == 0 {
                    b -= 1;
                } else {
                    return b;
                }
            }
        }
        return b;
    }
    fn pod_eq(&self, other: &Pod) -> bool {
        assert_eq!(self.limbs(), other.limbs());
        for i in (0..self.limbs()).rev() {
            if self.get_limb(i) != other.get_limb(i) {
                return false;
            }
        }
        return true;
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

impl Pod for Limb {
    fn limbs(&self) -> BigSize {
        return 1;
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        if i == 0 {
            return *self;
        } else {
            panic!("Tried to index into a Limb other than index 0")
        }
    }
//     fn min_limbs(&self) -> BigSize {
//         if *self == 0 {
//             return 0;
//         } else {
//             return 1;
//         }
//     }
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


