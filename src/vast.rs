#![allow(unused)]

use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::AddAssign;
use std::cmp::Ordering;

use crate::limb::{*};
use crate::big::Big;

pub struct VastMut<'a> {
    v: &'a mut[Limb]
}

pub struct Vast<'a> {
    v: &'a[Limb]
}

impl<'a> Vast<'a> {
}

pub trait AvastOps {
    fn min_length(&self) -> BigSize;
    fn length(&self) -> BigSize;
    fn bits(&self) -> BigSize;
}

pub trait Avast {
    fn as_slice(&self) -> &[Limb];
}

impl<'a, T> AvastOps for T where T: Avast {
    fn min_length(&self) -> BigSize {
        let v = self.as_slice();
        for i in (0..v.len()).rev() {
            if v[i] != 0 {
                return (i + 1) as BigSize;
            }
        }
        return 0;
    }
    fn length(&self) -> BigSize {
        self.as_slice().len() as BigSize
    }
    fn bits(&self) -> BigSize {
        let v = self.as_slice();
        let mut b : BigSize = (v.len() as BigSize) * (LIMB_SHIFT as BigSize);
        for i in (0..v.len()).rev() {
            let l = v[i];
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
}

impl<'a> Avast for Vast<'a> {
    fn as_slice(&self) -> &[Limb] {
        self.v
    }
}
impl<'a> Avast for VastMut<'a> {
    fn as_slice(&self) -> &[Limb] {
        self.v
    }
}

impl<'a> VastMut<'a> {
    pub fn zero(& mut self) {
        for i in 0..self.v.len() {
            self.v[i] = 0;
        }
    }
}

impl<'a> From<&'a VastMut<'a>> for Vast<'a> {
    fn from(m: &'a VastMut<'a>) -> Vast<'a> {
        Vast {v:  m.v}
    }
}

impl<'a> From<VastMut<'a>> for Vast<'a> {
    fn from(m: VastMut<'a>) -> Vast<'a> {
        Vast {v:  m.v}
    }
}

impl<'a> From<&'a Big> for Vast<'a> {
    fn from(b: &'a Big) -> Vast<'a> {
        Vast {v: b.as_slice()}
    }
}

impl<'a> From<&'a mut Big> for VastMut<'a> {
    fn from(b: &'a mut Big) -> VastMut<'a> {
        VastMut {v: b.as_mut_slice()}
    }
}


impl<'a> Index<BigSize> for Vast<'a> {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i as usize] }
}

impl<'a> Index<BigSize> for VastMut<'a> {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i as usize] }
}

impl<'a> IndexMut<BigSize> for VastMut<'a> {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb { &mut self.v[i as usize] }
}

impl<'a> AddAssign<Vast<'a>> for VastMut<'a> {
    fn add_assign(&mut self, a: Vast) {
        let mut carry : Limb = 0;
        let sz = self.length();
        for i in 0..sz {
            let ai: Limb;
            if i < a.length() {
                ai = a[i];
            } else {
                ai = 0;
            }
            let (s1, o1) = Limb::overflowing_add(self[i], carry);
            let (s2, o2) = Limb::overflowing_add(s1, ai);
            self[i] = s2;
            if o1 {
                carry = 1;
            } else {
                carry = 0;
            }
            if o2 {
                carry += 1;
            }
        }
        for i in sz..a.length() {
            if (a[i] != 0) {
                panic!("Vast overflow in add_assign(Vast): other too long!")
            }
        }
        if carry > 0 {
            panic!("Vast overflow in add_assign(Vast)!");
        }
    }
}

impl<'a> AddAssign<Limb> for VastMut<'a> {
    fn add_assign(&mut self, a: Limb) {
        let mut carry : Limb = a;
        let sz = self.length();
        for i in 0..sz {
            let (s1, o1) = Limb::overflowing_add(self[i], carry);
            self[i] = s1;
            if o1 {
                carry = 1;
            } else {
                carry = 0;
            }
        }
        if carry > 0 {
            panic!("Vast overflow in add_assign(Limb)");
        }
    }
}

impl<'a> PartialEq for Vast<'a> {
    fn eq (&self, other: &Vast) -> bool {
        assert_eq!(self.v.len(), other.v.len());
        for i in (0..self.v.len()).rev() {
            if self.v[i] != other.v[i] {
                return false;
            }
        }
        return true;
    }
}
impl<'a> Eq for Vast<'a> {}

impl<'a> PartialEq for VastMut<'a> {
    fn eq (&self, other: &VastMut) -> bool {
        Vast::from(self).eq(&Vast::from(other))
    }
}

impl<'a> Ord for Vast<'a> {
    fn cmp(&self, other: &Vast) -> Ordering {
        if self.v.len() > other.v.len() {
            for i in (0..self.v.len()).rev() {
                let selfi = self.v[i];
                let otheri : Limb;
                if i < other.v.len() {
                    otheri = other.v[i];
                } else {
                    otheri = 0;
                }
                if selfi > otheri {
                    return Ordering::Greater;
                } else if selfi < otheri {
                    return Ordering::Less;
                }
            }
        } else {
            for i in (0..other.v.len()).rev() {
                let otheri = other.v[i];
                let selfi : Limb;
                if i < self.v.len() {
                    selfi = self.v[i];
                } else {
                    selfi = 0;
                }
                if selfi > otheri {
                    return Ordering::Greater;
                } else if selfi < otheri {
                    return Ordering::Less;
                }
            }
        }
        return Ordering::Equal;
    }
}
impl<'a> PartialOrd for Vast<'a> {
    fn partial_cmp(&self, other: &Vast) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq<Limb> for Vast<'a> {
    fn eq (&self, other: &Limb) -> bool {
        for i in 1..self.v.len() {
            if self.v[i] != 0 {
                return false;
            }
        }
        return self.v[0] == *other;
    }
}
impl<'a> PartialEq<Limb> for VastMut<'a> {
    fn eq (&self, other: &Limb) -> bool {
        Vast::from(self).eq(other)
    }
}

pub trait VastMutOps {
    fn assign_mul(self, a: Vast, b: Vast);
}

impl<'a> VastMutOps for VastMut<'a> {
    fn assign_mul(self, a: Vast, b: Vast) {
        let mut p = self;
        let a_sz = a.min_length();
        let b_sz = b.min_length();
        let p_sz = Vast::from(&p).length();
        p.zero();
        assert!(p_sz >= a_sz + b_sz);
        for j in 0..b_sz {
            let mut carry : Limb2 = 0;
            for i in 0..a_sz {
    //             println!("i: {} j: {}, i+j: {}", i, j, i + j);
                let mut old = p[i + j] as Limb2;
    //             println!("old: {:X} carry: {:X}", old, carry);
                old += carry;
    //             println!("a[i]: {:X} b[j]: {:X}", a[i], b[j]);
                let x = (a[i] as Limb2) * (b[j] as Limb2);
                let new = old + x;
    //             println!("x: {:X} new: {:X}", x, new);
                if new < x || new < old {
                    panic!("Wrapped!");
                }
                carry = new >> LIMB_SHIFT;
                p[i + j] = (new & LIMB_MASK) as Limb;
            }
    //         println!("Final carry: {:X}", carry);
            // we don't have anywhere left to put the final carry :(
            assert_eq!(carry & 0xFFFFFFFFFFFFFFFF0000000000000000u128, 0);
            p[a_sz+j] = carry as Limb;
        }
    }
}



// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big::{*};
    use crate::vast::{*};
    #[test]
    fn create() {
        let mut a = Big::new(2);
        let mut b: VastMut = VastMut::from(&mut a);
        assert_eq!(b[0], 0);
        assert_eq!(b[1], 0);
        b[0] = 2;
        assert_eq!(b[0], 2);
    }
    #[test]
    fn multiply() {
        let mut ab = Big::from_hex("F99527E2862042DBB66313F44C4C47B6C0259E16F63F000194C4D5BBE3BB39075C068A34E30288DED00B063876877E9D68E100A50B479104B85497A9BA510638");
        let mut a = Vast::from(&ab);
        let mut bb = Big::from_hex("D517B4B082CB3651E1CEE7FF12C1F985D94E89EF3FBA74A9314E05B5D1533B48AE9F0C710ED2A2C8885CAD9F5757B8FB27CC95B7B89BF33DDCE184822C1376C");
        let mut b = Vast::from(&bb);
        let mut pb = Big::new(16);
        let mut p = VastMut::from(&mut pb);
        p.assign_mul(a, b);
        assert_eq!(pb.hex_str(), "CFC036BF050D730EA92C3A8E66BF44B94319958CC3C0E8FD8570CC61A7CD39CD66EFBE891948DD59F4AF2FCFC7CB63B8682B9660B3AC2142DF54E37DA1A4EDF3D0962A14463B0E5CDE726E2FD903B8FFA53AC9E2ECCCDB93B0D4078912B98887A54AA1782704F6E7AF894DA712689FDFCCDFCF33B91DB702A68AC4B22BCA7A0");
    }
}
