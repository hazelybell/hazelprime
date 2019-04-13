use crate::limb::{*};
use crate::vast::{*};
use crate::svast::{*};
use crate::chopped::{*};

pub trait Fermat {
    fn fermat(&mut self, n: BigSize);
//     fn add_fermat(&mut self, n: BigSize);
    fn mod_fermat(self, n: BigSize, temp: VastMut);
}

impl<'a> Fermat for VastMut<'a> {
    fn fermat(&mut self, n: BigSize) {
        let sz = div_up(n+1, LIMB_SIZE);
        let bit = n % LIMB_SIZE;
        let limb = n / LIMB_SIZE;
        assert!(self.length() >= sz);
        for i in 0..self.length() {
            self[i] = 0;
        }
        self[limb] = 1 << bit;
        self[0] |= 1;
    }
    
//     fn add_fermat(&mut self, n: BigSize) {
//         assert_ne!(n, 0)
//         let mut carry : Limb = 1;
//         let sz = self.length();
//         let bit = n % LIMB_SIZE;
//         let limb = n / LIMB_SIZE;
//         for i in 0..sz {
//             if i == limb {
//                 carry += 1 << bit;
//                 // TODO: Finish me
//             }
//         }
//     }
    
    fn mod_fermat(self, n: BigSize, mut temp: VastMut) {
        temp.zero();
        let sz = div_up(n+1, LIMB_SIZE);
        let f = SVastMut::from(temp);
        let src_bits = self.bits();
        let iters = div_up(src_bits, n);
        for i in 0..iters {
            let chunk: BigSize;
            if (n*i + n) > src_bits {
                chunk = src_bits - n*i;
            } else {
                chunk = n;
            }
            if chunk == 0 {
                break;
            }
            let piece = Chopped::chop(Vast::from(&self), n*i, chunk);
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
