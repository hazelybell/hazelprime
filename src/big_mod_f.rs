#![warn(rust_2018_idioms)]

use crate::limb::{*};
use crate::big::{*};
use crate::sbig::{*};
use crate::vast::{*};
use crate::fermat::{*};

pub fn fermat(n : BigSize) -> Big {
    let sz = div_up(n+1, LIMB_SIZE);
    let mut f = Big::new_one(sz);
    f <<= n;
    f += 1;
    return f;
}

pub fn mod_fermat(x : &Big, n : BigSize) -> Big {
    let sz = div_up(n+1, LIMB_SIZE);
    let mut plus = Big::new(sz);
//     let mut sv = SVastMut{v: VastMut{v: plus.as_mut_slice()}, negative: false};
//     sv.assign_mod_fermat(&Vast::from(x), Fermat::new(n));
    let v = VastMut::from(&mut plus);
    Fermat::mod_fermat(v, &Vast::from(x), Fermat::new(n));
    return plus;
}


pub fn mul_mod_fermat(a : &Big, b : &Big, n : BigSize) -> Big {
    let p_big = a * b;
    let p = mod_fermat(&p_big, n);
    return p;
}

pub fn inv_mod_fermat(a: &Big, n: BigSize) -> Big {
    // extended euclidean algorithm
    // https://en.wikipedia.org/w/index.php?title=Extended_Euclidean_algorithm&oldid=890036949#Pseudocode
    let b = fermat(n);
    assert_eq!(a.length(), b.length());
    let sz = b.length();
    let mut t = SBig::new(b.length());
    let mut new_t = SBig::new_one(b.length());
    let mut r = b.clone();
    let mut new_r = a.clone();
    while new_r != 0 {
        let q = &r / &new_r;
        
        let qt = (&q * &new_t).downsized(sz);
        let new_new_t = &t - &qt;
        t = new_t;
        new_t = new_new_t;
        
        let qr = (&q * &new_r).downsized(sz);
        let mut new_new_r = r.clone();
        new_new_r -= &qr;
        r = new_r;
        new_r = new_new_r;
    }
    if r > 1 {
        panic!("a is not invertible")
    }
    if t.is_negative() {
        t = &t + &b;
    }
    return t.into_big();
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big_mod_f::{*};
    #[test]
    fn mod_fermat_1() {
        let mut a = Big::new(1);
        a[0] = 656;
        let r = mod_fermat(&a, 3);
        assert_eq!(r[0], 8);
        assert_eq!(r.length(), 1);
    }
    #[test]
    fn mod_fermat_2() {
        let mut a = fermat(100);
        assert_eq!(a[0], 1);
        assert_eq!(a[1], 1<<36);
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 0);
        assert_eq!(r[1], 0);
        a[0] = 2;
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 0);
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        a[1] = 0xFFFFFFFFFFFFFFFFu64;
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 0xFFFFFFFFF0000000u64);
        assert_eq!(r[1], 0x0000000FFFFFFFFFu64);
        let r = mod_fermat(&a, 99);
        assert_eq!(r[0], 0xFFFFFFFFE0000000u64);
        assert_eq!(r[1], 0x00000007FFFFFFFFu64);
    }
    #[test]
    fn mul_mod_fermat_1() {
        let mut a = Big::new(1);
        a[0] = 41;
        let mut b = Big::new(1);
        b[0] = 16;
        let r = mul_mod_fermat(&a, &b, 3);
        assert_eq!(r[0], 8);
        assert_eq!(r.length(), 1);
        let r = mul_mod_fermat(&a, &b, 16);
        assert_eq!(r[0], 656);
        assert_eq!(r.length(), 1);
    }
    #[test]
    fn mul_mod_fermat_2() {
        let mut a = Big::new(1);
        a[0] = 0x10000000;
        let mut b = Big::new(1);
        b[0] = 0x10;
        let r = mul_mod_fermat(&a, &b, 32);
        assert_eq!(r[0], 0x100000000);
    }
    #[test]
    fn mod_fermat_3() {
        let mut a = Big::new(1);
        a[0] = 0x100000000;
        let r = mod_fermat(&a, 32);
        assert_eq!(r[0], 0x100000000);
    }
    #[test]
    fn inv_mod_fermat_1() {
        // Example from Emily Smith
        let mut a = Big::new(3);
        a[0] = 8;
        let i = inv_mod_fermat(&a, 136);
        let mut b = Big::new(3);
        b[0] = 7;
        b <<= 133;
        b += 1;
        assert_eq!(i, b);
        let p = mul_mod_fermat(&a, &i, 136);
        assert_eq!(p, 1);
    }
    #[test]
    fn mod_fermat_4() {
        let mut a = Big::new(3);
        a[0] = 0x0000000000000040;
        a[1] = 0x0000000000000000;
        a[2] = 0x0000000000003800;
        let r = mod_fermat(&a, 136);
        println!("{:?}", r);
        assert_eq!(r, 8);
    }
    #[test]
    fn mul_mod_fermat_3() {
        let mut a = Big::new(3);
        a[0] = 0xFFFFFFFC00000001;
        a[1] = 0xFFFFFFFFFFFFFFFF;
        a[2] = 0x00000000000000FF;
        let b = a.clone();
        let r = &a * &b;
        println!("a^2: {:?}", &a * &b);
        assert_eq!(r[0], 0xFFFFFFF800000001);
        assert_eq!(r[1], 0x000000000000000F);
        assert_eq!(r[2], 0xFFFFF80000000200);
        assert_eq!(r[3], 0xFFFFFFFFFFFFFFFF);
        assert_eq!(r[4], 0x000000000000FFFF);
        let r = mul_mod_fermat(&a, &b, 136);
        println!("{:?}", r);
        assert_eq!(r[0], 0x0000000000000000);
        assert_eq!(r[1], 0x0000000000000010);
        assert_eq!(r[2], 0x0000000000000000);
    }
}
