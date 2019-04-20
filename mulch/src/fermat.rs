#![warn(rust_2018_idioms)]

use std::cmp::Ordering;
use std::fmt;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::svast::{*};
use crate::chopped::{*};

#[derive(Clone,Copy)]
pub struct Fermat {
    pub n: BigSize
}

pub fn pmod(x: isize, n: BigSize) -> BigSize {
    assert!(n > 0);
    let n = n as isize;
    let mut r = x % n;
    if r < 0 {
        r += n;
    }
    return r as BigSize;
}

pub fn div_down(n: isize, d: BigSize) -> isize {
    if n < 0 {
        let mut r = n / d;
        if r * d > n {
            r = r - 1;
        }
        return r;
    } else {
        return n / d; // integer div rounds towards 0
    }
}

impl Fermat {
    pub fn new(n: BigSize) -> Fermat {
        Fermat {n: n}
    }
    pub fn mod_fermat<'a>(dest: &mut VastMut<'a>, src: &Vast<'_>, f: Fermat) {
        dest.zero();
        let mut mod_f = SVastMut {v: VastMut {v: dest.v}, negative: false};
        let src_bits = src.bits();
        let iters = div_up(src_bits, f.n);
//         println!("iters: {}", iters);
//         println!("src: {}", src.to_hex());
        for i in 0..iters {
            let chunk: BigSize;
            if (f.n*i + f.n) > src_bits {
                chunk = src_bits - f.n*i;
            } else {
                chunk = f.n;
            }
            if chunk == 0 {
                break;
            }
            let piece = Chopped::chop(src.clone(), f.n*i, chunk);
//             println!("start: {} chunk: {}: {}", f.n*i, chunk, piece.to_hex());
            if i % 2 == 0 {
//                 println!("i={} +{}", i, piece.to_hex());
                mod_f.pod_add_assign(&piece);
            } else {
//                 println!("i={} -{}", i, piece.to_hex());
                mod_f.pod_sub_assign(&piece);
            }
//             println!("mod_f: {}", mod_f);
        }
        if mod_f.negative && !mod_f.v.eq(&0) {
            mod_f.pod_add_assign(&f);
            if mod_f.negative {
                panic!("Still negative!");
            }
        } else {
            if mod_f.pod_cmp(&f) != Ordering::Less {
                mod_f.pod_sub_assign(&f);
                if mod_f.pod_cmp(&f) != Ordering::Less {
                    panic!("Still too big!")
                }
            }
        }
//         let r = mod_f.into_vastmut();
//         println!("Res: {}", r.to_hex());
    }
    pub fn mul_mod_fermat<'a>(
        dest: &mut VastMut<'a>,
        a: &Vast<'_>,
        b: &Vast<'_>,
        f: Fermat,
        work: &'a mut VastMut<'a>
    ) {
        work.pod_assign_mul(a, b);
        Fermat::mod_fermat(dest, &Vast::from(&*work), f);
    }
    pub fn mul_mod_fermat_assign<'a>(
        a: &mut VastMut<'a>,
        b: &Vast<'_>,
        f: Fermat,
        work: &'a mut VastMut<'a>
    ) {
        work.pod_assign_mul(a, b);
        Fermat::mod_fermat(a, &Vast::from(&*work), f);
    }
    pub fn mod_fermat_shifted<'a>(
        dest: &mut VastMut<'a>,
        src: &Vast<'_>,
        shift: isize,
        f: Fermat
    ) {
        dest.zero();
        let mut mod_f = SVastMut {v: VastMut {v: dest.v}, negative: false};
        let src_bits = src.bits();
        let iters = div_up(src_bits, f.n);
//         println!("iters: {}", iters);
//         println!("src: {}", src.to_hex());
        let real_start = div_down(shift, f.n);
        for i in 0..iters {
            let ri = i + real_start;
            let chunk: BigSize;
            if (f.n*i + f.n) > src_bits {
                chunk = src_bits - f.n*i;
            } else {
                chunk = f.n;
            }
            if chunk == 0 {
                break;
            }
            let piece = Chopped::chop(src.clone(), f.n*i, chunk);
//             println!("start: {} chunk: {}: {}", f.n*i, chunk, piece.to_hex());
            if pmod(ri, 2) == 0 {
//                 println!("i={} +{}", i, piece.to_hex());
                mod_f.pod_add_assign(&piece);
            } else {
//                 println!("i={} -{}", i, piece.to_hex());
                mod_f.pod_sub_assign(&piece);
            }
//             println!("mod_f: {}", mod_f);
        }
        if mod_f.negative && !mod_f.v.eq(&0) {
            mod_f.pod_add_assign(&f);
            if mod_f.negative {
                panic!("Still negative!");
            }
        } else {
            if mod_f.pod_cmp(&f) != Ordering::Less {
                mod_f.pod_sub_assign(&f);
                if mod_f.pod_cmp(&f) != Ordering::Less {
                    panic!("Still too big!")
                }
            }
        }
//         let r = mod_f.into_vastmut();
//         println!("Res: {}", r.to_hex());
    }
}

impl Pod for Fermat {
    fn limbs(&self) -> BigSize {
        let sz = div_up(self.n+1, LIMB_SIZE);
        return sz;
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        let n = self.n;
        let bit = n % LIMB_SIZE;
        let limb = n / LIMB_SIZE;
        let mut l: Limb = 0;
        if i == 0 {
            l |= 1;
        }
        if i == limb {
            l |= 1 << bit;
        }
        return l;
    }
}

pod_eq! {
    lifetime 'a;
    Fermat;
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    use crate::big::{*};
    use crate::big_mod_f::mod_fermat;
    #[test]
    fn fermat_1() {
        let f = Fermat::new(64);
        assert_eq!(f.to_hex(),"10000000000000001");
        let f = Fermat::new(32);
        assert_eq!(f.to_hex(),"100000001");
    }
    #[test]
    fn mul_mod_fermat() {
        let n = 136;
        let f = Fermat::new(n);
        let big_a = Big::from_hex("9D68E100A50B479104B85497A9BA510639");
        let big_b = Big::from_hex("B6C0259E16F63F000194C4D5BBE3BB3908");
        let mut big_p = Big::new(div_up(n+1, LIMB_SIZE));
        let mut big_work = Big::new(div_up(n+1, LIMB_SIZE)*2);
        Fermat::mul_mod_fermat(
            &mut VastMut::from(&mut big_p),
            &Vast::from(&big_a),
            &Vast::from(&big_b),
            f,
            &mut VastMut::from(&mut big_work)
        );
        assert_eq!(big_a.to_hex(), "9D68E100A50B479104B85497A9BA510639");
        assert_eq!(big_b.to_hex(), "B6C0259E16F63F000194C4D5BBE3BB3908");
        assert_eq!(big_work.to_hex(), "705EB5C093303072599253B8A1DAC0361ED48C086047B568C2521C9B7220F38DE2C8");
        assert_eq!(big_p.to_hex(), "642D529FB485384FF88A47B97F18CDACAA");
    }
    #[test]
    fn mul_mod_fermat_assign() {
        let n = 136;
        let f = Fermat::new(n);
        let mut big_a = Big::from_hex("9D68E100A50B479104B85497A9BA510639");
        let big_b = Big::from_hex("B6C0259E16F63F000194C4D5BBE3BB3908");
        let mut big_work = Big::new(div_up(n+1, LIMB_SIZE)*2);
        Fermat::mul_mod_fermat_assign(
            &mut VastMut::from(&mut big_a),
            &Vast::from(&big_b),
            f,
            &mut VastMut::from(&mut big_work)
        );
        assert_eq!(big_b.to_hex(), "B6C0259E16F63F000194C4D5BBE3BB3908");
        assert_eq!(big_work.to_hex(), "705EB5C093303072599253B8A1DAC0361ED48C086047B568C2521C9B7220F38DE2C8");
        assert_eq!(big_a.to_hex(), "642D529FB485384FF88A47B97F18CDACAA");
    }
    #[test]
    fn pmod_() {
        assert_eq!(pmod(1,2), 1);
        assert_eq!(pmod(-1,2), 1);
        assert_eq!(pmod(-3,2), 1);
        assert_eq!(pmod(-2,2), 0);
    }
    #[test]
    fn div_down_() {
        assert_eq!(div_down(2,2), 1);
        assert_eq!(div_down(1,2), 0);
        assert_eq!(div_down(0,2), 0);
        assert_eq!(div_down(-1,2), -1);
        assert_eq!(div_down(-2,2), -1);
    }
    #[test]
    fn mod_fermat_shifted_0() {
        let n = 30;
        let ba = Big::from_hex("FFFFFFFF");
        let mut br = Big::new(1);
        let c = mod_fermat(&ba, n);
        let a = Vast::from(&ba);
        let mut r = VastMut::from(&mut br);
        Fermat::mod_fermat_shifted(&mut r, &a, 0, Fermat::new(n));
        assert_eq!(c.to_hex(), br.to_hex());
    }
    #[test]
    fn mod_fermat_shifted_1() {
        let n = 30;
        let x = "FFFFFFFF";
        let mut ba = Big::from_hex(x);
        ba <<= 1;
        let bb = Big::from_hex(x);
        let mut br = Big::new(1);
        let c = mod_fermat(&ba, n);
        let b = Vast::from(&bb);
        let mut r = VastMut::from(&mut br);
        Fermat::mod_fermat_shifted(&mut r, &b, 1, Fermat::new(n));
        assert_eq!(c.to_hex(), br.to_hex());
    }
}
