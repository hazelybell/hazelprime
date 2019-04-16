#![warn(rust_2018_idioms)]

use std::cmp::Ordering;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::svast::{*};
use crate::chopped::{*};

#[derive(Clone,Copy,Debug)]
pub struct Fermat {
    pub n: BigSize
}

impl Fermat {
    pub fn new(n: BigSize) -> Fermat {
        Fermat {n: n}
    }
    pub fn mod_fermat<'a>(mut dest: VastMut<'a>, src: &Vast<'_>, f: Fermat) -> VastMut<'a> {
        dest.zero();
        let mut mod_f = SVastMut::from_vastmut(dest);
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
        let r = mod_f.into_vastmut();
//         println!("Res: {}", r.to_hex());
        return r;
    }
    pub fn mul_mod_fermat<'a>(
        dest: VastMut<'a>,
        a: &Vast<'_>,
        b: &Vast<'_>,
        f: Fermat,
        mut work: VastMut<'a>
    ) -> (VastMut<'a>, VastMut<'a>) {
        work.pod_assign_mul(a, b);
        let dest = Fermat::mod_fermat(dest, &Vast::from(&work), f);
        return (dest, work);
    }
    pub fn mul_mod_fermat_assign<'a>(
        a: VastMut<'a>,
        b: &Vast<'_>,
        f: Fermat,
        mut work: VastMut<'a>
    ) -> (VastMut<'a>, VastMut<'a>) {
        work.pod_assign_mul(&a, b);
        let a = Fermat::mod_fermat(a, &Vast::from(&work), f);
        return (a, work);
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
            VastMut::from(&mut big_p),
            &Vast::from(&big_a),
            &Vast::from(&big_b),
            f,
            VastMut::from(&mut big_work)
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
            VastMut::from(&mut big_a),
            &Vast::from(&big_b),
            f,
            VastMut::from(&mut big_work)
        );
        assert_eq!(big_b.to_hex(), "B6C0259E16F63F000194C4D5BBE3BB3908");
        assert_eq!(big_work.to_hex(), "705EB5C093303072599253B8A1DAC0361ED48C086047B568C2521C9B7220F38DE2C8");
        assert_eq!(big_a.to_hex(), "642D529FB485384FF88A47B97F18CDACAA");
    }
}
