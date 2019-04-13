use crate::limb::{*};
use crate::vast::{*};

pub trait Fermat {
    fn fermat(&mut self, n: BigSize);
//     fn add_fermat(&mut self, n: BigSize);
//     fn mod_fermat(&mut self, n: BigSize);
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
//     
//     fn mod_fermat(&mut self, n: BigSize) {
//         
//     }
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
