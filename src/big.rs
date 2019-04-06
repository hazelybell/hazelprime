#![allow(non_snake_case)]
#![allow(unused)]

use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;
use std::cmp::{min, max};
// use std::boxed::Box;

pub type BigSize = isize;
const SIZE_SHIFT : usize = 63;
pub type Limb = u64;
pub type Limb2 = u128;
const LIMB_SHIFT : usize = 64;
const LIMB_SIZE : BigSize = LIMB_SHIFT as BigSize;
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

pub fn divides(n : BigSize, d : BigSize) -> bool {
    return (d % n) == 0;
}

#[derive(Debug)]
pub struct Nkn {
    N: BigSize,
    k: BigSize,
    n: BigSize
}

pub fn ss_simple_get_Nkn(p_bits: BigSize, sz: BigSize) -> Option<Nkn> {
    // find a suitable N, k and n
    let N_min = p_bits + 1;
    let N_max = sz * LIMB_SIZE - 1;
    let k_max : BigSize = 16;
    let k_min : BigSize = 1;
    for N in N_min..(N_max+1) {
        println!("Trying N={}", N);
        for k in k_min..(k_max+1) {
            let twok = 1 << k;
            if (twok > p_bits) {
                break;
            }
            let n_min = 2 * N / twok + k;
            let n_max = twok * 4;
            println!("Trying k={} twok={} n_min=2N/2^k+k={} n_max={}", k, twok, n_min, n_max);
            if divides(twok, N) {
                for n in n_min..(n_max+1) {
                    println!("Trying n={}", n);
                    if divides(twok, n) {
                        println!("Satisfied: N={}, k={}, twok={}, n={}", N, k, twok, n);
                        let optimal_twok = (N as f64).sqrt();
                        println!("Optimal twok={}", optimal_twok);
                        let r = Nkn {
                            N: N,
                            k: k,
                            n: n
                        };
                        return Some(r);
                    }
                }
            }
        }
    }
    return None;
}

#[derive(Debug)]
pub struct NknSize {
    Nkn: Nkn,
    sz: BigSize
}

pub fn ss_simple_get_size(p_bits: BigSize) -> NknSize {
     /* room for the modulo which is 2^N+1, and bigger than the biggest
      * value stored in p_bits, so if p_bits is 32, the max valued would
      * be 2^32-1 and we need to use 2^32+1 for the modulo at a minimum,
      * however, 2^32+1 takes 33 bits!
      */
    let min_bits = p_bits + 1;
    
    let mut min_sz : BigSize = min_bits / LIMB_SIZE;
    if (min_sz * LIMB_SIZE < min_bits) {
        // we got bit by integer division rounding down
        min_sz = min_sz + 1;
    }
    for sz in min_sz..(min_sz*2) {
        println!("Trying size {}: {} bits", sz, sz * LIMB_SIZE);
        let o_Nkn = ss_simple_get_Nkn(p_bits, sz);
        match o_Nkn {
            Some(an_Nkn) => {
                println!("Found size {}: {} bits", sz, sz * LIMB_SIZE);
                let r = NknSize {
                    Nkn: an_Nkn,
                    sz: sz
                };
                return r;
            },
            None => {
            }
        }
    }
    unreachable!();
}

// pub fn ss_big_enough(p_bits: BigSize, sz: BigSize) -> bool {
//     // Determine if a size sz is big enough to hold a product
//     // p_bits long without going over the size of the modulo
//     let max_bits = sz * LIMB_SIZE;
//     let mod_bits = max_bits - 1; // e.g. 2^63+1 takes 64 bits
//     if p_bits <= mod_bits {
//         let o_Nkn = ss_simple_get_Nkn(p_bits, sz);
//         match o_Nkn {
//             Some(_an_Nkn) => {
//                 return true;
//             },
//             None => {
//                 return false;
//             },
//         }
//     } else {
//         return false; // not possibly big enough
//     }
// }

// pub fn multiply_ss_simple(a: Big, b: Big) -> Big {
//     let mut target_sz : BigSize = 1;
//     let a_bits = a.bits();
//     let b_bits = b.bits();
//     let p_bits = a_bits + b_bits; // number of bits in the product
//     while !ss_big_enough(p_bits, target_sz) {
//         target_sz << 1;
//     }
//     // we need space for the product which can be a+b long
//     let a2 = big_extend(a, target_sz); 
//     let b2 = big_extend(b, target_sz);
// //     multiply_ss_simple2(a2, b2, p_bits);
//     return new_big(1); // shut up the compiler
// }

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
    #[test]
    fn divides_() {
        assert!(divides(64, 128));
        assert!(divides(16, 16));
    }
    #[test]
    fn ss_simple_get_Nkn_1() {
        let r = ss_simple_get_Nkn(64, 2);
        assert!(r.is_some())
    }

    #[test]
    fn ss_simple_get_Nkn_2() {
        let r = ss_simple_get_Nkn(32, 1);
        assert!(r.is_some())
    }
    #[test]
    fn ss_simple_get_Nkn_3() {
        let r = ss_simple_get_Nkn(64, 1);
        println!("{:?}", r);
        assert!(r.is_none())
    }
    #[test]
    fn ss_simple_get_Nkn_4() {
        let r = ss_simple_get_Nkn(133, 4);
        println!("{:?}", r);
        assert!(r.is_some())
    }
    #[test]
    fn ss_simple_get_Nkn_5() {
        let r = ss_simple_get_Nkn(256, 4);
        println!("{:?}", r);
        assert!(r.is_none())
    }
    #[test]
    fn ss_simple_get_Nkn_6() {
        let r = ss_simple_get_Nkn(1024, 32);
        println!("{:?}", r);
        assert!(r.is_some())
    }
    #[test]
    fn ss_simple_get_size_() {
        let r = ss_simple_get_size(1024);
        println!("{:?}", r);
        assert!(false)
    }
}
