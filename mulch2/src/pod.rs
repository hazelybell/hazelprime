#![warn(rust_2018_idioms)]

use crate::limb::{*};

pub trait Pod {
    fn limbs(&self) -> usize;
    fn get_limb(&self, i: usize) -> &Limb;
}


