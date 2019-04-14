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
    fn pod_cmp(&self, rhs: &Pod) -> Ordering;
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
        if self.limbs() > other.limbs() {
            for i in (0..self.limbs()).rev() {
                if i < other.limbs() {
                    if self.get_limb(i) != other.get_limb(i) {
                        return false;
                    }
                } else {
                    if self.get_limb(i) != 0 {
                        return false;
                    }
                }
            }
        } else {
            for i in (0..other.limbs()).rev() {
                if i < self.limbs() {
                    if self.get_limb(i) != other.get_limb(i) {
                        return false;
                    }
                } else {
                    if 0 != other.get_limb(i) {
                        return false;
                    }
                }
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
    fn pod_cmp(&self, rhs: &Pod) -> Ordering {
        let lhs = self;
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
}

// Mutable stuff **********************************************************

pub trait PodMut: Pod {
    fn set_limb(&mut self, i: BigSize, l: Limb);
}

pub trait PodMutOps {
    fn pod_add_assign(&mut self, a: &Pod);
    fn pod_sub_assign(&mut self, a: &Pod);
    fn pod_backwards_sub_assign(&mut self, a: &Pod);
}

impl<T> PodMutOps for T where T: PodMut {
    fn pod_add_assign(&mut self, a: &Pod) {
        let dest = self;
        let mut carry : Limb = 0;
        let sz = dest.limbs();
        for i in 0..sz {
            let ai: Limb;
            if i < a.limbs() {
                ai = a.get_limb(i);
            } else {
                ai = 0;
            }
            let (s1, o1) = Limb::overflowing_add(dest.get_limb(i), carry);
            let (s2, o2) = Limb::overflowing_add(s1, ai);
            dest.set_limb(i, s2);
            if o1 {
                carry = 1;
            } else {
                carry = 0;
            }
            if o2 {
                carry += 1;
            }
        }
        for i in sz..a.limbs() {
            if a.get_limb(i) != 0 {
                panic!("Vast overflow in add_assign(Vast): other too long!")
            }
        }
        if carry > 0 {
            panic!("Vast overflow in add_assign(Vast)!");
        }
    }
    fn pod_sub_assign(&mut self, a: &Pod) {
        let dest = self;
        let mut borrow : Limb = 0;
        let sz = dest.limbs();
        for i in 0..sz {
            let s : Limb;
            let ai = a.get_limb(i);
            let di = dest.get_limb(i);
            s = di.wrapping_sub(borrow);
            if di >= borrow {
                borrow = 0;
            } else {
                borrow = 1;
            }
            let s2 = s.wrapping_sub(ai);
            if s < ai {
                borrow = borrow + 1;
            }
            dest.set_limb(i, s2);
        }
        for i in sz..a.limbs() {
            if a.get_limb(i) != 0 {
                panic!("Vast underflow in sub_assign(Vast): other too long")
            }
        }
        if borrow > 0 {
            panic!("Vast underflow in sub_assign(Vast)");
        }
    }
    fn pod_backwards_sub_assign(&mut self, a: &Pod) {
        let dest = self;
        let mut borrow : Limb = 0;
        let sz = dest.limbs();
        for i in 0..sz {
            let s : Limb;
            // these two are flipped!
            let ai = dest.get_limb(i);
            let di = a.get_limb(i);
            s = di.wrapping_sub(borrow);
            if di >= borrow {
                borrow = 0;
            } else {
                borrow = 1;
            }
            let s2 = s.wrapping_sub(ai);
            if s < ai {
                borrow = borrow + 1;
            }
            dest.set_limb(i, s2);
        }
        for i in sz..a.limbs() {
            if a.get_limb(i) != 0 {
                panic!("Vast underflow in sub_assign(Vast): other too long")
            }
        }
        if borrow > 0 {
            panic!("Vast underflow in sub_assign(Vast)");
        }
    }

}


