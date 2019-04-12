use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;
use std::fmt;
use std::cmp::Ordering;
use std::ops::Mul;
use std::ops::Div;
use std::ops::ShlAssign;
use std::ops::ShrAssign;
use std::ops::AddAssign;
use std::ops::SubAssign;

use crate::slice_ops::{*};

pub type BigSize = isize;
// pub const SIZE_SHIFT : usize = 63;
pub type Limb = u64;
pub type Limb2 = u128;
pub const LIMB_SHIFT : usize = 64;
pub const LIMB_SIZE : BigSize = LIMB_SHIFT as BigSize;
pub const LIMB_MASK : Limb2 = 0xFFFFFFFFFFFFFFFFu128;

pub struct Big {
    v: Box<[Limb]>
}

impl Big {
    pub fn length(&self) -> BigSize { self.v.len() as BigSize }
    pub fn least_sig(&self) -> Limb { self.v[0] }
    pub fn zero(&mut self) { for i in 0..self.v.len() { self.v[i] = 0 } }
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
    pub fn bitlen(&self) -> BigSize {
        (self.v.len() as BigSize) * LIMB_SIZE
    }
    pub fn new(sz : BigSize) -> Big {
        assert_ne!(sz, 0);
        let new_v : Vec<Limb> = std::vec::from_elem(0, sz as usize);
        return Big { v: new_v.into_boxed_slice() };
    }
    pub fn new_one(sz : BigSize) -> Big {
        assert_ne!(sz, 0);
        let mut new_v : Vec<Limb> = std::vec::from_elem(0, sz as usize);
        new_v[0] = 1;
        return Big { v: new_v.into_boxed_slice() };
    }
    pub fn slice_bits(&self, start : BigSize, l : BigSize) -> Big {
        let sz = div_up(l, LIMB_SIZE);
        let mut r = Big::new(sz);
        let src_limb_start = start / LIMB_SIZE;
        let src_bit_start = start % LIMB_SIZE;
        let src_lower_bits = LIMB_SIZE - src_bit_start;
        let src_upper_bits = src_bit_start;
        for i in 0..(l/LIMB_SIZE) {
            // we need a total of LIMB_SIZE bits for each limb
            // this is like a shift left
            // the lower destination limb bits come from 
            // the upper LIMB_SIZE - start source limb bits
            let dst_lower = self[src_limb_start + i] >> src_upper_bits;
            let dst_upper;
            let over = src_limb_start + i + 1 >= self.length();
            if src_lower_bits < 64 && !over {
                dst_upper = self[src_limb_start + i + 1]
                << src_lower_bits;
            } else {
                dst_upper = 0;
            }
            r[i] = dst_lower | dst_upper;
        }
        let last = sz - 1 + src_limb_start;
        let over = last >= self.length();
        let last_r_bits = (start + l - 1) % LIMB_SIZE + 1;
//         println!("last: {} over: {} last_r_bits: {}", last, over, last_r_bits);
        if l % LIMB_SIZE > 0 && (!over) && last_r_bits > 0 {
            let shake_l = LIMB_SIZE - last_r_bits;
            let shake_r = LIMB_SIZE - l % LIMB_SIZE;
//             println!("shake_l: {} shake_r: {}", shake_l, shake_r);
            let last_r = (self[last] << shake_l) >> shake_r;
            r[sz-1] = last_r;
        }
        return r;
    }
    pub fn hex_str(&self) -> String {
        format!("{:X}", self)
    }
    pub fn downsized(&self, sz: BigSize) -> Big {
        for i in (sz as usize)..self.v.len() {
            assert_eq!(self.v[i], 0);
        }
        let mut n = Big::new(sz);
        for i in 0..(sz as usize) {
            n.v[i] = self.v[i];
        }
        return n;
    }
    pub fn from_hex(src: &str) -> Big {
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
        let mut r = Big::new(sz);
        for i in 0..chunks {
            let end = len - i * chunk_size;
            let start = len - (i+1) * chunk_size;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            r[i as BigSize] = chunk;
        }
        if remaining > 0 {
            let end = len - chunks * chunk_size;
            let start = 0;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            r[chunks as BigSize] = chunk;
        }
        return r;
    }
    pub fn as_mut_slice(&mut self) -> &mut[Limb] {
        let x: &mut[Limb] = &mut self.v[..];
        return x;
    }
    pub fn as_slice(&self) -> &[Limb] {
        let x: &[Limb] = &self.v[..];
        return x;
    }
}

impl Index<BigSize> for Big {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i as usize] }
}

impl IndexMut<BigSize> for Big {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb { &mut self.v[i as usize] }
}

impl Clone for Big {
    fn clone(&self) -> Big {
        let c : Box<[Limb]> = self.v.clone();
        return Big { v: c };
    }
}

impl PartialEq for Big {
    fn eq (&self, other: &Big) -> bool {
        assert_eq!(self.v.len(), other.v.len());
        for i in (0..self.v.len()).rev() {
            if self.v[i] != other.v[i] {
                return false;
            }
        }
        return true;
    }
}
impl Eq for Big {}

impl PartialEq<Limb> for &Big {
    fn eq (&self, other: &Limb) -> bool {
        for i in 1..self.v.len() {
            if self.v[i] != 0 {
                return false;
            }
        }
        return self.v[0] == *other;
    }
}

impl PartialEq<Limb> for Big {
    fn eq (&self, other: &Limb) -> bool {
        self == *other
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Big) -> Ordering {
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
impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Big) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Limb> for Big {
    fn partial_cmp(&self, other: &Limb) -> Option<Ordering> {
        for i in 1..self.v.len() {
            if self.v[i] > 0 {
                return Some(Ordering::Greater);
            }
        }
        return Some(self.v[0].cmp(other));
    }
}


impl fmt::UpperHex for Big {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut z = true;
        let mut r = Ok(());
        for i in (0..self.v.len()).rev() {
            if z {
                if self.v[i] == 0 && i > 0 {
                } else {
                    z = false;
                    r = write!(f, "{:X}", self.v[i]);
                }
            } else {
                r = write!(f, "{:016X}", self.v[i]);
            }
            match r {
                Ok(_) => {},
                Err(_) => {return r;}
            }
        }
        return r;
    }
}

impl fmt::Display for Big {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return fmt::UpperHex::fmt(self, f);
    }
}

impl fmt::Debug for Big {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut r : fmt::Result;
        r = write!(f, "Big {:016X}", self.v[self.v.len()-1]);
        match r {
            Ok(_) => {},
            Err(_) => {return r;}
        }
        for i in (0..(self.v.len()-1)).rev() {
            r = write!(f, ",{:016X}", self.v[i]);
            match r {
                Ok(_) => {},
                Err(_) => {return r;}
            }
        }
        return r;
    }
}

impl ShlAssign<BigSize> for Big {
    fn shl_assign(&mut self, n: BigSize) {
        assert!(self.bits() + n <= self.bitlen());
        // rely on integer rounding down here
        let n_limbs = n / LIMB_SIZE;
        let n_bits = n - (n_limbs * LIMB_SIZE);
        let sz = self.length();
        assert!(n_limbs < sz);
        for i in (n_limbs..sz).rev() {
            let src_lower = i-n_limbs-1;
            let src_upper = i-n_limbs;
            // we need a total of LIMB_SIZE bits for each limb
            // the upper LIMB_SIZE - n_bits of the destination comes
            // from the lower LIMB_SIZE - n_bits of the upper source
            let upper : Limb = self[src_upper] << n_bits;
            let lower : Limb;
            if src_lower < 0 || n_bits == 0 {
                lower = 0;
            } else {
                // the lower n_bits of the destination comes
                // from the upper n_bits of the source
                // so we discard LIMB_SIZE - n_bits of the lower source
                lower = self[src_lower] >> (LIMB_SIZE - n_bits);
            }
            self[i] = upper | lower;
        }
        for i in 0..n_limbs {
            // zero the least significant bits
            self[i] = 0;
        }
    }
}

impl ShrAssign<BigSize> for Big {
    fn shr_assign(&mut self, n: BigSize) {
        // rely on integer rounding down here
        let n_limbs = n / LIMB_SIZE;
        let n_bits = n - (n_limbs * LIMB_SIZE);
        let sz = self.length();
        let limbs_remaining = sz - n_limbs;
        assert!(n_limbs < sz);
        for i in 0..limbs_remaining {
            let src_lower = i + n_limbs;
            let src_upper = i + n_limbs + 1;
            let lower = self[src_lower] >> n_bits;
            let upper : Limb;
            if src_upper >= sz || n_bits == 0 {
                upper = 0;
            } else {
                upper = self[src_upper] << (LIMB_SIZE - n_bits);
            }
            self[i] = upper | lower;
        }
        for i in limbs_remaining..sz {
            self[i] = 0; // zero the most significant bits
        }
    }
}

impl AddAssign<&Big> for Big {
    fn add_assign(&mut self, a : &Big) {
        let mut carry : Limb = 0;
        let sz = self.length();
        for i in 0..sz {
            let ai: Limb;
            if i < a.length() {
                ai = a[i];
            } else {
                ai = 0;
            }
            let s : Limb2 = 
                (self[i] as Limb2) 
                + (carry as Limb2)
                + (ai as Limb2);
            self[i] = (s & LIMB_MASK) as Limb;
            carry = (s >> LIMB_SHIFT) as Limb;
        }
        if carry > 0 {
            panic!("Big overflow in add_assign(Big)");
        }
    }
    
}

impl AddAssign<Limb> for Big {
    fn add_assign(&mut self, a: Limb) {
        let mut carry : Limb = a;
        let sz = self.length();
        for i in 0..sz {
            let s : Limb2 = (self[i] as Limb2) + (carry as Limb2);
            self[i] = (s & LIMB_MASK) as Limb;
            carry = (s >> LIMB_SHIFT) as Limb;
        }
        if carry > 0 {
            panic!("Big overflow in increase()");
        }
    }
    
}

impl SubAssign<&Big> for Big {
    fn sub_assign(&mut self, a : &Big) {
//         println!("{:?}-{:?}", self, a);
        let mut borrow : Limb = 0;
        let sz = self.length();
        for i in 0..sz {
            let s : Limb;
            s = self[i].wrapping_sub(borrow);
            if self[i] >= borrow {
                borrow = 0;
            } else {
                borrow = 1;
            }
            let s2 = s.wrapping_sub(a[i]);
            if s < a[i] {
                borrow = borrow + 1;
            }
            self[i] = s2;
        }
        if borrow > 0 {
            panic!("Big underflow in sub_assign(Big)");
        }
    }
}

pub fn big_extend(x: Big, sz: BigSize) -> Big {
    let x_sz = x.length();
    assert!(sz >= x_sz);
    let mut r : Big = Big::new(sz);
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
//     println!("a: {}, b: {}", a, b);
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

impl Mul for Big {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self {
        let self_sz = self.v.len();
        let a = self.as_slice();
        let rhs_sz = rhs.v.len();
        let b = rhs.as_slice();
        let mut pb = Big::new((self_sz + rhs_sz) as BigSize);
        let p = pb.as_mut_slice();
        multiply_slice(p, a, b);
        return pb;
    }
}

impl Mul for &Big {
    type Output = Big;
    
    fn mul(self, rhs: Self) -> Big {
        let self_sz = self.v.len();
        let rhs_sz = rhs.v.len();
        let mut p = Big::new((self_sz + rhs_sz) as BigSize);
        multiply_long(&mut p, self, rhs);
        return p;
    }
}

pub fn div_up(n : BigSize, d : BigSize) -> BigSize {
    let mut r = n / d;
    if r * d < n {
        r = r + 1;
    }
    return r;
}




impl Div for &Big {
    type Output = Big;
    fn div(self, rhs: Self) -> Big {
        let n = self;
        let d = rhs;
        if d == 0 {
            panic!("Trying to divide by zero-valued `Big`!");
        }
        // do long division
        // TODO: fix this to use u64 division instead of binary
        // https://en.wikipedia.org/w/index.php?title=Division_algorithm&oldid=891240037#Integer_division_(unsigned)_with_remainder
        let sz = n.length();
        if d >= n {
            return Big::new(sz);
        }
        let bits = n.bits();
        let mut q = Big::new(sz);
        let mut r = Big::new(sz);
        for i in (0..bits).rev() {
            r <<= 1;
            let limb_i = i/LIMB_SIZE;
            let bit_i = i%LIMB_SIZE;
            let mask_i : Limb = (1 as Limb) << bit_i;
            let n_i = (n[limb_i] & mask_i) >> bit_i;
            r[0] = r[0] | n_i;
            if r.ge(d) {
                r -= &d;
                q[limb_i] = q[limb_i] | mask_i;
            }
        }
        return q;
    }
}



// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big::{*};
    #[test]
    fn smoke() {
        let a = Big::new(2);
        assert_eq!(a.length(), 2);
        let b = Big::new(2);
        let p = a * b;
        assert_eq!(p.length(), 4);
        assert_eq!(p.least_sig(), 0);
    }
    #[test]
    fn multiply_long_() {
        let mut a = Big::new(2);
        assert_eq!(a.length(), 2);
        let mut b = Big::new(2);
        let mut p = Big::new(4);
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
    fn mul_() {
        let mut a = Big::new(2);
        assert_eq!(a.length(), 2);
        let mut b = Big::new(2);
        a[0] = 0xFFFFFFFFu64;
        b[0] = 0xFFFFFFFFu64;
        let p = &a * &b;
        assert_eq!(p[0], 0xFFFFFFFE00000001);
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[0] = 0xFFFFFFFFFFFFFFFFu64;
        let p = &a * &b;
        println!("{:?}x{:?}={:?}", a, b, p);
        assert_eq!(p[1], 0xFFFFFFFFFFFFFFFE);
        assert_eq!(p[0], 0x0000000000000001);
        a[1] = 0x00FFFFFFFFFFFFFFu64;
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[1] = 0x0u64;
        b[0] = 0x10u64;
        let p = a * b;
        println!("{:X} {:X}", p[1], p[0]);
        assert_eq!(p[1], 0x0FFFFFFFFFFFFFFFu64);
        assert_eq!(p[0], 0xFFFFFFFFFFFFFFF0u64);
    }
    #[test]
    fn mul_2() {
        let mut a = Big::new(1);
        a[0] = 0xFFFFFFFC00000001;
        let p = &a * &a;
        assert_eq!(p[0], 0xFFFFFFF800000001);
    }
    #[test]
    fn shift_right() {
        let mut a = Big::new(2);
        a[0] = 0x00000000000000FFu64;
        a[1] = 0x0000000000000000u64;
        a <<= 8;
        assert_eq!(a[0], 0x000000000000FF00u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a <<= 48;
        assert_eq!(a[0], 0xFF00000000000000u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a <<= 4;
        assert_eq!(a[0], 0xF000000000000000u64);
        assert_eq!(a[1], 0x000000000000000Fu64);
        a <<= 4;
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x00000000000000FFu64);
        a <<= 0;
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x00000000000000FFu64);
        a[0] = 0x00000000000000FFu64;
        a[1] = 0x0000000000000000u64;
        a <<= 64;
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x00000000000000FFu64);
    }
    #[test]
    fn shift_left() {
        let mut a = Big::new(2);
        a[0] = 0x0000000000000000u64;
        a[1] = 0x00000000000000FFu64;
        a >>= 0;
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x00000000000000FFu64);
        a >>= 4;
        assert_eq!(a[0], 0xF000000000000000u64);
        assert_eq!(a[1], 0x000000000000000Fu64);
        a >>= 4;
        assert_eq!(a[0], 0xFF00000000000000u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a >>= 48;
        assert_eq!(a[0], 0x000000000000FF00u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a >>= 8;
        assert_eq!(a[0], 0x00000000000000FFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a[0] = 0x0000000000000000u64;
        a[1] = 0x00000000000000FFu64;
        a >>= 64;
        assert_eq!(a[0], 0x00000000000000FFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
    }
    #[test]
    fn clone_() {
        let mut a = Big::new(1);
        a[0] = 0x00000000000000AAu64;
        let b = a.clone();
        a[0] = 0x00000000000000FFu64;
        assert_eq!(a[0], 0x00000000000000FFu64);
        assert_eq!(b[0], 0x00000000000000AAu64);
    }
    #[test]
    fn increase_() {
        let mut a = Big::new(2);
        a[0] = 0x0FFFFFFFFFFFFFFFu64;
        a[1] = 0x0000000000000000u64;
        a += 0xF000000000000000u64;
        assert_eq!(a[0], 0xFFFFFFFFFFFFFFFFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a += 0x0000000000000001u64;
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x0000000000000001u64);
    }
    #[test]
    fn slice_bits_1() {
        let mut a = Big::new(3);
        a[0] = 0x8899AABBCCDDEEFFu64;
        a[1] = 0x0011223344556677u64;
        a[2] = 0x0123456789ABCDEFu64;
        let b = a.slice_bits(64, 64);
        assert_eq!(b[0], 0x0011223344556677u64);
        assert_eq!(b.length(), 1);
        let b = a.slice_bits(64, 128);
        assert_eq!(b[0], 0x0011223344556677u64);
        assert_eq!(b[1], 0x0123456789ABCDEFu64);
        assert_eq!(b.length(), 2);
        let b = a.slice_bits(32, 64);
        assert_eq!(b[0], 0x445566778899AABBu64);
        let b = a.slice_bits(32, 128);
        assert_eq!(b[0], 0x445566778899AABBu64);
        assert_eq!(b[1], 0x89ABCDEF00112233u64);
    }
    #[test]
    fn slice_bits_2() {
        let mut a = Big::new(3);
        a[0] = 0x8899AABBCCDDEEFFu64;
        a[1] = 0x0011223344556677u64;
        a[2] = 0x0123456789ABCDEFu64;
        let b = a.slice_bits(8, 8);
        assert_eq!(b[0], 0x00000000000000EEu64);
        assert_eq!(b.length(), 1);
        let b = a.slice_bits(8+64, 8);
        assert_eq!(b[0], 0x0000000000000066u64);
        assert_eq!(b.length(), 1);
    }
    #[test]
    fn slice_bits_3() {
        let mut a = Big::new(1);
        a[0] = 0x100000000;
        let b = a.slice_bits(32, 32);
        assert_eq!(b[0], 0x0000000000000001u64);
    }
    #[test]
    fn decrease_big_() {
        let mut a = Big::new(2);
        a[0] = 0x0000000000000000u64;
        a[1] = 0x0000000000000001u64;
        let b = Big::new_one(2);
        a -= &b;
        assert_eq!(a[0], 0xFFFFFFFFFFFFFFFFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
    }
    #[test]
    fn is_zero_() {
        let mut a = Big::new(10);
        assert!(&a == 0u64);
        a[1] = 1;
        assert!(!(a == 0));
    }
    #[test]
    fn hex_str_() {
        let mut a = Big::new(1);
        assert_eq!(a.hex_str(), "0");
        a[0] = 1;
        assert_eq!(a.hex_str(), "1");
        let mut a = Big::new(2);
        assert_eq!(a.hex_str(), "0");
        a[0] = 1;
        assert_eq!(a.hex_str(), "1");
        a[1] = 1;
        assert_eq!(a.hex_str(), "10000000000000001");
        assert_eq!(format!("{:X}", a), "10000000000000001");
        assert_eq!(format!("{}", a), "10000000000000001");
    }
    #[test]
    fn div_() {
        let mut n = Big::new(1);
        let mut d = Big::new(1);
        n[0] = 0x68E100A50B479104u64;
        d[0] = 0x00000000D00B0638u64;
        let q = &n / &d;
        assert_eq!(q[0], 0x00000000810E1609u64);
    }
    #[test]
    fn from_hex_() {
        let a = Big::from_hex("810E1609");
        assert_eq!(a[0], 0x810E1609);
    }
}
