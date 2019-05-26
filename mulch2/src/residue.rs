#![warn(rust_2018_idioms)]

use std::fmt::Write;

use crate::limb::*;
use mulch2_macro::make_residue;

make_residue!(4096);
make_residue!(2048);

pub trait ResBasics {
    fn as_slice(&self) -> &[Limb];
    fn as_mut_slice(&mut self) -> &mut [Limb];
}

pub trait ResOps {
    fn from_hex(&mut self, src: &str);
    fn to_hex(&self) -> String;
}

impl<T> ResOps for T where T: ResBasics {
    fn from_hex(&mut self, src: &str) {
        let chunk_size = (LIMB_BITS / 4) as usize;
        let len = src.len();
        let chunks = len / chunk_size;
        let remaining = len % chunk_size;
        let mut limbs = self.as_mut_slice().iter_mut();
        for i in 0..chunks {
            let end = len - i * chunk_size;
            let start = len - (i+1) * chunk_size;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            let limb = limbs.next();
            match limb {
                Some(v) => { *v = chunk; }
                None => { panic!("String too big!"); }
            }
        }
        if remaining > 0 {
            let end = len - chunks * chunk_size;
            let start = 0;
            let chunk: Limb = Limb::from_str_radix(&src[start..end], 16)
                .unwrap();
            let limb = limbs.next();
            match limb {
                Some(v) => { *v = chunk; }
                None => { panic!("String too big!"); }
            }
        }
        for limb in limbs {
            *limb = 0;
        }
    }
    fn to_hex(&self) -> String {
        let mut s = String::new();
        let mut limbs = self.as_slice().iter().rev();
        write!(s, "{:X}", limbs.next().unwrap()).unwrap();
        for limb in limbs {
            write!(s, "{:016X}", limb).unwrap();
        }
        return s;
    }
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    #[test]
    fn new_from_hex() {
        let mut b = Residue2048::new();
        b.from_hex(
              "327B00242CFAEE8DF0C4F7486CADB351CEABFBDCF340A119E34DC3BEFD209D\
             6408553EA56FEC93DED68F3FFB9BABB60E3E0C03FF652DB955AACE4F05576796\
             3E8DA37B7C7FD5C35A29AE814656217397F562B3E5527F49DFAC585F32E8B905\
             ADCB3C58F3C0F4D3511A1E02A357EBE095371FAEC2F1616595CBA68029323FF8\
             916FB9E7792750B8309B1322E8A1B8038881CE87B99F241A1C475629ACF29077\
             A8A06FED983FF02114C3E7D57CFF99EAB76323E2B356E24A0CC49618BE216A2A\
             C97DB6185B92275311C91B2B337B38F6839960047A9971BFE776668CEB0802DC\
             3E1F7310289C6E4AF589914E6FCAC46673D036908906B308CB301134B6F47432"
        );
        assert_eq!(b.to_hex(),
              "0\
             00327B00242CFAEE8DF0C4F7486CADB351CEABFBDCF340A119E34DC3BEFD209D\
             6408553EA56FEC93DED68F3FFB9BABB60E3E0C03FF652DB955AACE4F05576796\
             3E8DA37B7C7FD5C35A29AE814656217397F562B3E5527F49DFAC585F32E8B905\
             ADCB3C58F3C0F4D3511A1E02A357EBE095371FAEC2F1616595CBA68029323FF8\
             916FB9E7792750B8309B1322E8A1B8038881CE87B99F241A1C475629ACF29077\
             A8A06FED983FF02114C3E7D57CFF99EAB76323E2B356E24A0CC49618BE216A2A\
             C97DB6185B92275311C91B2B337B38F6839960047A9971BFE776668CEB0802DC\
             3E1F7310289C6E4AF589914E6FCAC46673D036908906B308CB301134B6F47432"
        );
    }
}
