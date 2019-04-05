#![feature(unsized_locals)]

use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;
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
    let mut new_v : Vec<Limb> = std::vec::from_elem(0, sz as usize);
    return Big { v: new_v.into_boxed_slice() };
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

pub fn ss1_get_k(x_sz : BigSize) -> (BigSize, BigSize, BigSize) {
    let mut k : BigSize = 0;
    let N : BigSize = x_sz * LIMB_SIZE * 2;
    println!("N: {}", N);
    for i in (0..SIZE_SHIFT).rev() {
        if (x_sz >> i) == 1 {
            println!("x size = 2^{}", i);
            k = i as BigSize;
            for j in 0..i {
                let x = (1 << j) & x_sz;
                if x > 0 {
                    println!("x size: {}", x_sz);
                    panic!("x size not a power of two!");
                }
            }
            break;
        }
    }
    assert_ne!(k, 0);
    while true {
        let twok = 1 << (k as usize);
        println!("k: {}; 2^k: {}", k, twok);
        if (twok > N) {
            panic!("Couldn't satisfy constraints! Giving up.");
        }
        
        // find n
        let mut n = 1;
        while (n < 2 * N / twok + k) {
            n = n << 1;
        }
        while (n % twok != 0) {
            n = n << 1;
        }
        println!("2N/2k+k = {};", 2 * N / twok + k);
        println!("n = {}", n);
        if n <= 64-1 {
            return (k, twok, n);
        }
        k = k + 1;
    }
    unreachable!();
}

pub fn multiply_ss1(p : &mut Big, x : &Big, y : &Big) {
    assert_ne!(p as *const _, x as *const _);
    assert_ne!(p as *const _, y as *const _);
    let x_sz = x.length();
    let y_sz = y.length();
    let p_sz = p.length();
    p.zero();
    assert_eq!(p_sz, x_sz);
    assert_eq!(x_sz, y_sz);

    let (k, twok, n) = ss1_get_k(x_sz);
    println!("twok: {}", twok);
    let N : BigSize = p_sz * (LIMB_SHIFT as BigSize);
    println!("doing multiplication modulo 2^{}+1", N);
    println!("inner multiplication modulo 2^{}+1", n);
    
    println!("n = {}", n);
    assert_eq!(n % twok, 0); // n is divisible by 2^k
    let p_bits = LIMB_SIZE * p_sz;
    let x_bits = LIMB_SIZE * x_sz;
    println!("p_bits: {} x_bits: {}", p_bits, x_bits);
    let x_elt_bits = x_bits / twok;
    let x_elt_per_limb = x_bits / x_elt_bits;
    println!("x_elt_bits: {} x_elt_per_limb: {}", x_elt_bits, x_elt_per_limb);
    let x_elt_mask = LIMB_MASK >> (LIMB_SHIFT - (x_elt_bits as usize));
    println!("x_elt_mask: {:x?}", x_elt_mask);

    let n_over_2k = n / twok;
    let two_n_over_2k = n_over_2k * 2;
    println!("n_over_2k: {}, two_n_over_2k: {}", n_over_2k, two_n_over_2k);
    let x_elts = twok;
    let y_elts = twok;
    assert!(n_over_2k > 0);
    println!("x_elts: {}", x_elts);
    let bits = x.bits() + y.bits();
    println!("twok: {} = 2^{}, n = {}, bits = {}, n_over_2k = {}",
                twok, k, n, bits, n_over_2k);
    let max_x_shift = n_over_2k * (x_elts - 1);
    println!("max_x_shift: {}", n_over_2k * (x_elts - 1));
    println!("two_n_over_2k: {} prou: 2^{}", two_n_over_2k, two_n_over_2k);
    assert!(n > 2 * N / twok + k);
    let mut weighted_x = new_big(twok);
    let mut weighted_y = new_big(twok);
    let prou : Limb = 1 << two_n_over_2k; // principle (2^k)th root of unity
    let mut a = new_big(twok * twok); // DFT matrix
    let mut a_inv = new_big(twok * twok); // Inverse DFT matrix
    {
        // compute the DFT matrix according to the Fermat Number Transform
        let mut aa = 1;
        for i in 0..twok {
            a[i] = 1;
            print!("{} ", 1);
            let mut aaa = aa;
            for j in 1..twok {
                let ij = i + j * twok;
                a[ij] = aaa;
                print!("{} ", aaa);
                aaa = (((aaa as u128) * (aa as u128)) % ((1u128 << n) + 1u128)) as u64; // TODO: replace with shifts and adds;
            }
            aa = (aa * prou) % ((1 << n) + 1); // TODO: replace with shifts and adds;
            println!("");
        }
    }
    // compute the inverse DFT matrix
    // since prou is the 2^k-th root of unity, that means that its multiplicative inverse is prou^(2^k-1)
    
    // Multiply A and B by the weight vector
    for j in 0..twok {
        {
            let xj = j/x_elt_per_limb;
            let subj = j % x_elt_per_limb;
            let e = ((x[xj] as u128) >> (x_elt_bits * subj)) & x_elt_mask;
            let ae = e << j * n_over_2k; // multiply by the weight
            // reduce mod 2^n+1
            let aer = ae % ((1 << n) + 1); // TODO: replace with shifts and adds
            weighted_x[j] = aer as Limb;
        }
        {
            let yj = j/x_elt_per_limb;
            let subj = j % x_elt_per_limb;
            let e = ((y[yj] as u128) >> (x_elt_bits * subj)) & x_elt_mask;
            let ae = e << j * n_over_2k; // multiply by the weight
            // reduce mod 2^n+1
            let aer = ae % ((1 << n) + 1); // TODO: replace with shifts and adds
            weighted_y[j] = aer as Limb;
        }
    }
    // Compute the DFT using Fermat Number Transform
    let mut dfted_x = new_big(twok);
    let mut dfted_y = new_big(twok);
    for i in 0..twok {
        {
            let mut xfi = 0;
            for j in 0..twok {
                xfi += (weighted_x[j] * a[i + j * twok])
                    % ((1 << n) + 1); // TODO: replace with shifts and adds
            }
            dfted_x[i] = xfi;
        }
        {
            let mut yfi = 0;
            for j in 0..twok {
                yfi += (weighted_y[j] * a[i + j * twok])
                    % ((1 << n) + 1); // TODO: replace with shifts and adds
            }
            dfted_y[i] = yfi;
        }
    }
    // Take the dot product P = X · Y
    let mut dfted_p = new_big(twok);
    for i in 0..twok {
        dfted_p[i] = (((dfted_x[i] as u128) * (dfted_y[i] as u128)) % ((1u128 << n) + 1u128)) as u64;
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
    #[test]
    fn multiply_ss1_() {
        let mut a = new_big(2);
        assert_eq!(a.length(), 2);
        let mut b = new_big(2);
        let mut p = new_big(2);
        a[0] = 0xFFFFFFFFu64;
        b[0] = 0xFFFFFFFFu64;
        multiply_ss1(&mut p, &a, &b);
        assert_eq!(p[0], 0xFFFFFFFE00000001);
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[0] = 0xFFFFFFFFFFFFFFFFu64;
        multiply_ss1(&mut p, &a, &b);
        println!("{:X} {:X}", p[1], p[0]);
        assert_eq!(p[1], 0xFFFFFFFFFFFFFFFE);
        assert_eq!(p[0], 0x0000000000000001);
    }
}
