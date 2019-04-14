use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::svast::{*};
use crate::chopped::{*};

#[derive(Clone,Copy,Debug)]
pub struct Fermat {
    pub n: BigSize
}

impl Pod for Fermat {
    fn limbs(&self) -> BigSize {
        let sz = div_up(self.n+1, LIMB_SIZE);
        return sz;
    }
    fn get_limb(&self, i: BigSize) -> Limb {
        let n = self.n;
        let sz = div_up(n+1, LIMB_SIZE);
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
    fn min_limbs(&self) -> BigSize {
        div_up(self.n+1, LIMB_SIZE)
    }
}

pub trait FermatOps {
    fn fermat(&mut self, n: BigSize);
//     fn add_fermat(&mut self, n: BigSize);
    fn mod_fermat(self, f: Fermat, temp: VastMut);
}

impl<'a> FermatOps for VastMut<'a> {
    fn fermat(&mut self, n: BigSize) {
        let sz = div_up(n+1, LIMB_SIZE);
        let bit = n % LIMB_SIZE;
        let limb = n / LIMB_SIZE;
        assert!(self.limbs() >= sz);
        for i in 0..self.limbs() {
            self[i] = 0;
        }
        self[limb] = 1 << bit;
        self[0] |= 1;
    }
    
    fn mod_fermat(self, f: Fermat, mut temp: VastMut) {
        temp.zero();
        let sz = f.limbs();
        let mut mod_f = SVastMut::from(temp);
        let src_bits = self.bits();
        let iters = div_up(src_bits, f.n);
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
            let piece = Chopped::chop(Vast::from(&self), f.n*i, chunk);
            if i % 2 == 0 {
                add_assign_svast_pod(&mut mod_f, &piece);
            } else {
                sub_assign_svast_pod(&mut mod_f, &piece);
            }
            if mod_f.negative {
                add_assign_svast_pod(&mut mod_f, &f);
            }
            if mod_f.negative {
                panic!("Still negative!");
            }
            panic!("TODO: implement")
        }
    }
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big::{*};
    use crate::vast_mod_f::{*};
    #[test]
    fn fermat_1() {
        let mut fb = Big::new(2);
        let mut f = VastMut::from(&mut fb);
        f.fermat(64);
        assert_eq!(fb.hex_str(),"10000000000000001");
        let mut f = VastMut::from(&mut fb);
        f.fermat(32);
        assert_eq!(fb.hex_str(),"100000001");
    }
}
