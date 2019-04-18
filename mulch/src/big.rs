#![warn(rust_2018_idioms)]

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

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::chopped::{*};

pub struct Big {
    pub v: Box<[Limb]>
}

impl Pod for Big {
    fn limbs(&self) -> BigSize {
        self.v.len() as BigSize 
    }
    fn get_limb(&self, i: BigSize) -> Limb { 
        self.v[i as usize] 
    }
}

pod_eq! {
    lifetime 'a;
    Big;
}

impl PodMut for Big {
    fn set_limb(&mut self, i: BigSize, l: Limb) {
        self.v[i as usize] = l;
    }
}

impl Big {
    pub fn length(&self) -> BigSize { self.v.len() as BigSize }
    pub fn least_sig(&self) -> Limb { self.v[0] }
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
        let c = Chopped::chop(Vast::from(self), start, l);
        for i in 0..sz {
            r[i] = c.get_limb(i);
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
        r.pod_assign_hex(src);
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

impl PartialEq<Limb> for &Big {
    fn eq (&self, other: &Limb) -> bool {
        self.pod_eq(other)
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Big) -> Ordering {
        self.pod_cmp(other)
    }
}
impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Big) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Limb> for Big {
    fn partial_cmp(&self, other: &Limb) -> Option<Ordering> {
        Some(self.pod_cmp(other))
    }
}


impl fmt::UpperHex for Big {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Display for Big {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        return fmt::UpperHex::fmt(self, f);
    }
}

impl fmt::Debug for Big {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
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
        self.pod_shl_assign(n);
    }
}

impl ShrAssign<BigSize> for Big {
    fn shr_assign(&mut self, n: BigSize) {
        self.pod_shr_assign(n);
    }
}

impl AddAssign<&Big> for Big {
    fn add_assign(&mut self, a : &Big) {
        self.pod_add_assign(a);
    }
}

impl AddAssign<Limb> for Big {
    fn add_assign(&mut self, a: Limb) {
        self.pod_add_assign(&a);
    }
}

impl SubAssign<&Big> for Big {
    fn sub_assign(&mut self, a : &Big) {
        self.pod_sub_assign(a);
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

impl Mul for Big {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self {
        let self_sz = self.v.len();
        let rhs_sz = rhs.v.len();
        let mut pb = Big::new((self_sz + rhs_sz) as BigSize);
        pb.pod_assign_mul(&self, &rhs);
        return pb;
    }
}

impl Mul for &Big {
    type Output = Big;
    
    fn mul(self, rhs: Self) -> Big {
        let self_sz = self.v.len();
        let rhs_sz = rhs.v.len();
        let mut pb = Big::new((self_sz + rhs_sz) as BigSize);
        pb.pod_assign_mul(self, rhs);
        return pb;
    }
}

impl Div for &Big {
    type Output = Big;
    fn div(self, rhs: Self) -> Big {
        let n = self;
        let d = rhs;
        let sz = n.length();
        let mut q = Big::new(sz);
        let mut r = Big::new(sz);
        q.pod_assign_div_qr(&mut r, n, d);
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
        let a = Big::from_hex("8123456789ABCDEF00112233445566778899AABBCCDDEEFF");
        let b = a.slice_bits(64, 64);
        assert_eq!(b[0], 0x0011223344556677u64);
        assert_eq!(b.length(), 1);
        let b = a.slice_bits(64, 128);
        assert_eq!(b.to_hex(), "8123456789ABCDEF0011223344556677");
        assert_eq!(b.length(), 2);
        let b = a.slice_bits(32, 64);
        assert_eq!(b.to_hex(), "445566778899AABB");
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
    fn slice_bits_4() {
        let a = Big::from_hex("4213D45F73B243B68546D5A3716796C4004B84606D1682968773BEAB03EF51D0E0");
        let b = a.slice_bits(200, 63);
        assert_eq!(b.to_hex(), "4213D45F73B243B6");
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
    #[test]
    fn bits_() {
        let a = Big::from_hex("3");
        assert_eq!(a.bits(), 2);
        let a = Big::from_hex("FFFFFFFF");
        assert_eq!(a.bits(), 32);
        let a = Big::from_hex("FFFFFFFFFFFFFFFF");
        assert_eq!(a.bits(), 64);
        let a = Big::from_hex("1FFFFFFFFFFFFFFFF");
        assert_eq!(a.bits(), 65);
    }
}