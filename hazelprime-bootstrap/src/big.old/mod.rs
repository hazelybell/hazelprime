#![warn(rust_2018_idioms)]

mod appendage;
mod errors;

#[macro_use] mod wrapped;

#[cfg(feature="rug_integer")]
mod rug_integer;

// use pod::PodN;
// use pod::FromStrRadix;
// use_ops!();

#[cfg(feature="rug_integer")]
use rug_integer::RugInteger;

// #[cfg(feature="rug_integer")]
// make_interpod_trait_types!(
//     RugInteger
// );


