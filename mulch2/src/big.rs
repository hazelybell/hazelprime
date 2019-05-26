#![warn(rust_2018_idioms)]

use crate::limb::*;
use crate::pod::*;
use mulch2_macro::make_big;

make_big!(4096);


// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use super::{*};
    #[test]
    fn new_() {
        let _b = Big4096::new();
    }
    #[test]
    fn get_() {
        let b = Big4096::new();
        for i in b.as_slice() {
            assert_eq!(i, &0);
        }
    }
}

