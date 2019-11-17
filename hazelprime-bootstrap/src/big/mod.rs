#![warn(rust_2018_idioms)]

mod appendage;
mod parse;
// mod bign;

#[macro_use] mod pod;

#[cfg(feature="rug_integer")]
mod rug_integer;

use pod::PodN;
use pod::FromStrRadix;
use_ops!();

#[cfg(feature="rug_integer")]
use rug_integer::RugInteger;

#[cfg(feature="rug_integer")]
make_interpod_trait_types!(
    RugInteger
);


