#![warn(rust_2018_idioms)]
#![allow(unused)]

use std::ops::Deref;
use std::ops::DerefMut;

use crate::limb::*;
use crate::pod::*;
use mulch2_macro::make_big;

make_big!(512);

        impl<'a> IntoIterator for &'a Big512 {
            type Item = &'a Limb;
            type IntoIter = ::std::slice::Iter<'a, Limb>;
            fn into_iter(self) -> Self::IntoIter {
                self.v.iter()
            }
        }


// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    #[test]
    fn new_() {
        let _b = Big512::new();
    }
    #[test]
    fn get_() {
        let b = Big512::new();
        for i in b.as_slice() {
            assert_eq!(i, &0);
        }
    }
}

