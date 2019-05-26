#![warn(rust_2018_idioms)]

extern crate proc_macro;
use syn;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

use super::LIMB_BITS;

pub fn make(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::LitInt);
    let bits = input.value() as usize;
    assert_eq!(bits % LIMB_BITS, 0);
    let size = bits / LIMB_BITS + 1;
    let name = &format!("Residue{}", bits);
    let ident = Ident::new(name, Span::call_site());
    let struc = quote! {
        struct #ident {
            v: [Limb; #size]
        }
    };
    let imp = quote! {
        impl #ident {
            pub fn new() -> #ident {
                #ident { v: [0; #size ] }
            }
        }
        impl ResBasics for #ident {
            fn as_slice(&self) -> &[Limb] {
                &self.v
            }
            fn as_mut_slice(&mut self) -> &mut [Limb] {
                &mut self.v
            }
        }
    };
    let result = quote! {
        #struc
        #imp
    };
    return result.into();
}


