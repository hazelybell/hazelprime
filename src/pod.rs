use std::cmp::Ordering;
use std::fmt::Write;

use crate::limb::{*};

pub trait Pod {
    // Anything with limbs
    fn limbs(&self) -> BigSize;
    fn get_limb(&self, i: BigSize) -> Limb;
}

pub trait PodOps: Pod {
    fn bits(&self) -> BigSize;
    fn pod_eq(&self, other: &Pod) -> bool;
    fn min_limbs(&self) -> BigSize;
    fn pod_cmp(&self, rhs: &Pod) -> Ordering;
    fn to_hex(&self) -> String;
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
    fn to_hex(&self) -> String {
        let mut z = true;
        let mut s = String::new();
        for i in (0..self.limbs()).rev() {
            if z {
                if self.get_limb(i) == 0 && i > 0 {
                } else {
                    z = false;
                    write!(s, "{:X}", self.get_limb(i)).unwrap();
                }
            } else {
                write!(s, "{:016X}", self.get_limb(i)).unwrap();
            }
        }
        return s;
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
    fn zero(&mut self);
    fn pod_add_assign(&mut self, a: &Pod);
    fn pod_sub_assign(&mut self, a: &Pod);
    fn pod_backwards_sub_assign(&mut self, a: &Pod);
    fn pod_assign_mul(self, a: &PodOps, b: &PodOps);
    fn pod_assign_hex(&mut self, src: &str);
}

impl<T> PodMutOps for T where T: PodMut {
    fn zero(&mut self) {
        for i in 0..self.limbs() {
            self.set_limb(i, 0);
        }
    }
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
    fn pod_assign_mul(self, a: &PodOps, b: &PodOps) {
        let mut p = self;
        let a_sz = a.min_limbs();
        let b_sz = b.min_limbs();
        let p_sz = p.limbs();
        p.zero();
        assert!(p_sz >= a_sz + b_sz);
        for j in 0..b_sz {
            let mut carry : Limb2 = 0;
            for i in 0..a_sz {
    //             println!("i: {} j: {}, i+j: {}", i, j, i + j);
                let mut old = p.get_limb(i + j) as Limb2;
    //             println!("old: {:X} carry: {:X}", old, carry);
                old += carry;
    //             println!("a[i]: {:X} b[j]: {:X}", a[i], b[j]);
                let x = (a.get_limb(i) as Limb2) * (b.get_limb(j) as Limb2);
                let new = old + x;
    //             println!("x: {:X} new: {:X}", x, new);
                if new < x || new < old {
                    panic!("Wrapped!");
                }
                carry = new >> LIMB_SHIFT;
                p.set_limb(i + j, (new & LIMB_MASK) as Limb);
            }
    //         println!("Final carry: {:X}", carry);
            // we don't have anywhere left to put the final carry :(
            assert_eq!(carry & 0xFFFFFFFFFFFFFFFF0000000000000000u128, 0);
            p.set_limb(a_sz+j, carry as Limb);
        }
    }
    fn pod_assign_hex(&mut self, src: &str) {
        let chunk_size = (LIMB_SIZE / 4) as usize;
        let len = src.len();
        let chunks = len / chunk_size;
        let remaining = len % chunk_size;
        let sz: BigSize;
        if remaining > 0 {
            sz = (chunks+1) as BigSize;
        } else {
            sz = chunks as BigSize;
        }
        assert!(sz <= self.limbs()); // make sure there's enough space
        for i in 0..chunks {
            let end = len - i * chunk_size;
            let start = len - (i+1) * chunk_size;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            self.set_limb(i as BigSize, chunk);
        }
        if remaining > 0 {
            let end = len - chunks * chunk_size;
            let start = 0;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            self.set_limb(chunks as BigSize, chunk);
        }
    }
}


