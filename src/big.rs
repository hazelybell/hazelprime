use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;

pub type BigSize = isize;
const SIZE_SHIFT : usize = 63;
pub type Limb = u64;
pub type Limb2 = u128;
const LIMB_SHIFT : usize = 64;
pub const LIMB_SIZE : BigSize = LIMB_SHIFT as BigSize;
const LIMB_MASK : Limb2 = 0xFFFFFFFFFFFFFFFFu128;

pub struct Big {
    v: Box<[Limb]>
}

impl Big {
    pub fn length(&self) -> BigSize { self.v.len() as BigSize }
    pub fn least_sig(&self) -> Limb { self.v[0] }
    pub fn zero(&mut self) { for i in 0..self.v.len() { self.v[i] = 0 } }
//     pub fn lt(&self, other: &Big) -> bool {
//         assert_eq!(self.v.len(), other.v.len());
//         for i in (0..self.v.len()).rev() {
//             if (self.v[i] < other.v[i]) {
//                 return true;
//             }
//         }
//         return false;
//     }
    pub fn bits(&self) -> BigSize {
        let mut b : BigSize = (self.v.len() as BigSize) * (LIMB_SHIFT as BigSize);
        for i in (0..self.v.len()).rev() {
            let l = self.v[i];
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
impl Index<BigSize> for Big {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i as usize] }
}
impl IndexMut<BigSize> for Big {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb { &mut self.v[i as usize] }
}

pub fn new_big(sz : BigSize) -> Big {
    assert_ne!(sz, 0);
    let new_v : Vec<Limb> = std::vec::from_elem(0, sz as usize);
    return Big { v: new_v.into_boxed_slice() };
}

pub fn big_extend(x: Big, sz: BigSize) -> Big {
    let x_sz = x.length();
    assert!(sz >= x_sz);
    let mut r : Big = new_big(sz);
    for i in 0..x_sz {
        r[i] = x[i];
    }
    return r;
}

pub fn multiply_long(p : &mut Big, a : &Big, b : &Big) {
    assert_ne!(p as *const _, a as *const _);
    assert_ne!(p as *const _, b as *const _);
    let a_sz = a.length();
    let b_sz = b.length();
    let p_sz = p.length();
    p.zero();
    assert_eq!(p_sz, a_sz + b_sz);
    for i in 0..a_sz {
        let mut carry : Limb2 = 0;
        for j in 0..b_sz {
            let pi : Limb2 = (p[i+j] as Limb2) + carry + ((a[i] as Limb2) * (b[j] as Limb2));
            carry = pi >> LIMB_SHIFT;
            p[i+j] = pi as Limb; // we think rust truncates so & LIMB_MASK is unnecessary
        }
        assert_eq!(carry, 0); // we don't have anywhere left to put the final carry :(
    }
}

pub fn multiply(a : Big, b : Big) -> Big {
    let a_sz = a.length();
    let b_sz = b.length();
    let mut p = new_big(a_sz + b_sz);
    multiply_long(&mut p, &a, &b);
    return p;
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big::{*};
    #[test]
    fn smoke() {
        let a = new_big(2);
        assert_eq!(a.length(), 2);
        let b = new_big(2);
        let p = multiply(a, b);
        assert_eq!(p.length(), 4);
        assert_eq!(p.least_sig(), 0);
    }
    #[test]
    fn multiply_long_() {
        let mut a = new_big(2);
        assert_eq!(a.length(), 2);
        let mut b = new_big(2);
        let mut p = new_big(4);
        a[0] = 0xFFFFFFFFu64;
        b[0] = 0xFFFFFFFFu64;
        multiply_long(&mut p, &a, &b);
        assert_eq!(p[0], 0xFFFFFFFE00000001);
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[0] = 0xFFFFFFFFFFFFFFFFu64;
        multiply_long(&mut p, &a, &b);
        println!("{:X} {:X}", p[1], p[0]);
        assert_eq!(p[1], 0xFFFFFFFFFFFFFFFE);
        assert_eq!(p[0], 0x0000000000000001);
        
        a[1] = 0x00FFFFFFFFFFFFFFu64;
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[1] = 0x0u64;
        b[0] = 0x10u64;        
        multiply_long(&mut p, &a, &b);
        println!("{:X} {:X}", p[1], p[0]);
        assert_eq!(p[1], 0x0FFFFFFFFFFFFFFFu64);
        assert_eq!(p[0], 0xFFFFFFFFFFFFFFF0u64);
    }
}
