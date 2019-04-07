use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;

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
    pub fn lt(&self, other: &Big) -> bool {
        assert_eq!(self.v.len(), other.v.len());
        for i in (0..self.v.len()).rev() {
            if self.v[i] < other.v[i] {
                return true;
            }
        }
        return false;
    }
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
    pub fn shift_left(&mut self, n : BigSize) {
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
            if src_lower < 0 {
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
    pub fn increase(&mut self, a : Limb) {
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
    pub fn increase_big(&mut self, a : Big) {
        let mut carry : Limb = 0;
        let sz = self.length();
        for i in 0..sz {
            let s : Limb2 = 
                (self[i] as Limb2) 
                + (carry as Limb2)
                + (a[i] as Limb2);
            self[i] = (s & LIMB_MASK) as Limb;
            carry = (s >> LIMB_SHIFT) as Limb;
        }
        if carry > 0 {
            panic!("Big overflow in increase_big()");
        }
    }
    pub fn decrease_big(&mut self, a : Big) {
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
            panic!("Big underflow in decrease_big()");
        }
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
        let last_r_bits = (start + l) % LIMB_SIZE;
        if l % LIMB_SIZE > 0 && (!over) && last_r_bits > 0 {
            let shake_l = LIMB_SIZE - last_r_bits;
            let shake_r = LIMB_SIZE - l % LIMB_SIZE;
            let last_r = (self[last] << shake_l) >> shake_r
            ;
            r[sz-1] = last_r;
        }
        return r;
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
    let mut p = Big::new(a_sz + b_sz);
    multiply_long(&mut p, &a, &b);
    return p;
}

pub fn div_up(n : BigSize, d : BigSize) -> BigSize {
    let mut r = n / d;
    if r * d < n {
        r = r + 1;
    }
    return r;
}

pub fn fermat(n : BigSize) -> Big {
    let sz = div_up(n, LIMB_SIZE);
    let mut f = Big::new_one(sz);
    f.shift_left(n);
    f.increase(1);
    return f;
}

pub fn mod_fermat(x : Big, n : BigSize) -> Big {
    let sz = div_up(n, LIMB_SIZE);
    let mut plus = Big::new(sz);
    let mut minus = Big::new(sz);
    let src_bits = x.bitlen();
    let iters = div_up(src_bits, n);
    for i in 0..iters {
        let piece = x.slice_bits(n*i, n);
        println!("piece: {}", piece[0]);
        if i % 2 == 0 { // even
            plus.increase_big(piece);
            println!("plus: {}", plus[0])
        } else { // odd
            minus.increase_big(piece);
            println!("minus: {}", minus[0])
        }
    }
    let f = fermat(n);
    if plus.lt(&minus) {
        plus.increase_big(f);
    }
    plus.decrease_big(minus);
    return plus;
}

pub fn mul_mod_fermat(a : Big, b : Big, n : BigSize) -> Big {
    let p_big = multiply(a, b);
    let p = mod_fermat(p_big, n);
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
        let a = Big::new(2);
        assert_eq!(a.length(), 2);
        let b = Big::new(2);
        let p = multiply(a, b);
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
    fn shift_() {
        let mut a = Big::new(2);
        a[0] = 0x00000000000000FFu64;
        a[1] = 0x0000000000000000u64;
        a.shift_left(8);
        assert_eq!(a[0], 0x000000000000FF00u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a.shift_left(48);
        assert_eq!(a[0], 0xFF00000000000000u64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a.shift_left(4);
        assert_eq!(a[0], 0xF000000000000000u64);
        assert_eq!(a[1], 0x000000000000000Fu64);
        a.shift_left(4);
        assert_eq!(a[0], 0x0000000000000000u64);
        assert_eq!(a[1], 0x00000000000000FFu64);
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
        a.increase(0xF000000000000000u64);
        assert_eq!(a[0], 0xFFFFFFFFFFFFFFFFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
        a.increase(0x0000000000000001u64);
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
    fn decrease_big_() {
        let mut a = Big::new(2);
        a[0] = 0x0000000000000000u64;
        a[1] = 0x0000000000000001u64;
        let b = Big::new_one(2);
        a.decrease_big(b);
        assert_eq!(a[0], 0xFFFFFFFFFFFFFFFFu64);
        assert_eq!(a[1], 0x0000000000000000u64);
    }
    #[test]
    fn mod_fermat_() {
        let mut a = Big::new(1);
        a[0] = 656;
        let r = mod_fermat(a, 3);
        assert_eq!(r[0], 8);
        assert_eq!(r.length(), 1);
    }
}
