#![allow(unused)]
#![warn(rust_2018_idioms)]

use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::AddAssign;
use std::cmp::Ordering;
use std::ops::SubAssign;
use std::fmt;

use crate::limb::{*};
use crate::pod::{*};
use crate::big::Big;

pub struct VastMut<'a> {
    pub v: &'a mut[Limb]
}

pub struct Vast<'a> {
    pub v: &'a[Limb]
}

impl<'a> Vast<'a> {
}

pub trait Avast {
    fn as_slice(&self) -> &[Limb];
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
    fn as_mut_slice(&mut self) -> &mut [Limb] {
        self.v
    }
}

impl<'a> From<&'a VastMut<'a>> for Vast<'a> {
    fn from(m: &'a VastMut<'a>) -> Vast<'a> {
        Vast {v:  m.v}
    }
}

// for Vast::from(x: &mut VastMut) do Vast::from(&*x)

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

impl<'a> Clone for Vast<'a> {
    fn clone(&self) -> Self {
        Vast {v: self.v}
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

impl<'a, T> Pod for T where T: Avast {
    fn limbs(&self) -> BigSize {
        self.as_slice().len() as BigSize
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        self.as_slice()[i as usize]
    }
}

impl<'a> PodMut for VastMut<'a> {
    fn set_limb(&mut self, i: BigSize, l: Limb) {
        self.as_mut_slice()[i as usize] = l;
    }
}

impl<'a> AddAssign<Vast<'a>> for VastMut<'a> {
    fn add_assign(&mut self, a: Vast<'a>) {
        self.pod_add_assign(&a);
    }
}

impl<'a> AddAssign<Limb> for VastMut<'a> {
    fn add_assign(&mut self, a: Limb) {
        self.pod_add_assign(&a);
    }
}

impl<'a> Ord for Vast<'a> {
    fn cmp(&self, other: &Vast<'a>) -> Ordering {
        self.pod_cmp(other)
    }
}
impl<'a> PartialOrd for Vast<'a> {
    fn partial_cmp(&self, other: &Vast<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait VastMutOps<'a> {
    fn assign_mul(&mut self, a: Vast<'a>, b: Vast<'a>);
}

impl<'a> VastMutOps<'a> for VastMut<'a> {
    fn assign_mul(&mut self, a: Vast<'a>, b: Vast<'a>) {
        self.pod_assign_mul(&a, &b);
    }
}


impl<'a> SubAssign<Vast<'a>> for VastMut<'a> {
    fn sub_assign(&mut self, a: Vast<'a>) {
        self.pod_sub_assign(&a);
    }
}

pod_eq! {
    lifetime 'a;
    Vast<'a>;
    VastMut<'a>;
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
        let mut b: VastMut<'_> = VastMut::from(&mut a);
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
