#![allow(unused)]

use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

use crate::big::{*};

pub struct VastMut<'a> {
    v: &'a mut[Limb]
}

pub struct Vast<'a> {
    v: &'a[Limb]
}

impl<'a> Vast<'a> {
    pub fn from_big(b: &'a Big) -> Vast<'a> {
        return Vast {v: b.as_slice()}
    }
    pub fn length(& self) -> BigSize {
        self.v.len() as BigSize
    }
}

trait AvastOps {
    fn min_length(&self) -> BigSize;
}

trait Avast {
    fn as_slice(&self) -> &[Limb];
}

impl<'a, T> AvastOps for T where T: Avast {
    fn min_length(& self) -> BigSize {
        let v = self.as_slice();
        for i in (0..v.len()).rev() {
            if v[i] != 0 {
                return (i + 1) as BigSize;
            }
        }
        return 0;
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
    pub fn from_big(b: &'a mut Big) -> VastMut<'a> {
        return VastMut {v: b.as_mut_slice()}
    }
    pub fn zero(& mut self) {
        for i in 0..self.v.len() {
            self.v[i] = 0;
        }
    }
    pub fn as_vast(&mut self) -> Vast {
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

pub fn multiply_vast(mut p: VastMut, a: Vast, b: Vast) {
    let a_sz = a.min_length();
    let b_sz = b.min_length();
    let p_sz = p.as_vast().length();
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
        let mut b: VastMut = VastMut::from_big(&mut a);
        assert_eq!(b[0], 0);
        assert_eq!(b[1], 0);
        b[0] = 2;
        assert_eq!(b[0], 2);
    }
    #[test]
    fn multiply() {
        let mut ab = Big::new(2);
        let mut a = Vast::from_big(&ab);
        let mut bb = Big::new(2);
        let mut b = Vast::from_big(&bb);
        let mut pb = Big::new(2);
        let mut p = VastMut::from_big(&mut pb);
        multiply_vast(p, a, b);
    }
}
