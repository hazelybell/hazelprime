#![warn(rust_2018_idioms)]
#![allow(unused)]

use std::ops::Deref;
use std::ops::DerefMut;

use crate::limb::*;
use crate::pod::*;
use mulch2_macro::make_big;

make_big!(512);

//         impl<'a> IntoIterator for &'a Big512 {
//             type Item = &'a Limb;
//             type IntoIter = ::std::slice::Iter<'a, Limb>;
//             fn into_iter(self) -> Self::IntoIter {
//                 self.v.iter()
//             }
//         }
        impl Deref for Big512 {
            type Target = [Limb];
            fn deref(&self) -> &[Limb] {
                &self.v
            }
        }
        impl DerefMut for Big512 {
            fn deref_mut(&mut self) -> &mut [Limb] {
                &mut self.v
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
        for i in b.into_iter() {
            assert_eq!(i, &0);
        }
    }
}

