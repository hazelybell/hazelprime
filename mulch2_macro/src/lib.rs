#![warn(rust_2018_idioms)]

extern crate proc_macro;
#[macro_use]
extern crate quote;

mod residue;
mod big;

use proc_macro::TokenStream;

const LIMB_BITS: usize = 64;

#[proc_macro]
pub fn make_big(item: TokenStream) -> TokenStream {
    big::make(item)
}

#[proc_macro]
pub fn make_residue(item: TokenStream) -> TokenStream {
    residue::make(item)
}
