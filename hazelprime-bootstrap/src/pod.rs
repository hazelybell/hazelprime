#![warn(rust_2018_idioms)]

use rug::integer::UnsignedPrimitive;

pub type AWord = usize;
pub const AWORD_BITS = mem::size_of::<AWord>;

pub trait APhrase {
    fn words(&self) -> usize;
    fn get_word(&self, i: usize) -> usize;
}


