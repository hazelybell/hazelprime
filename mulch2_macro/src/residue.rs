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
            v: [Limb; #size],
            negative: bool,
        }
    };
    let imp = quote! {
        impl #ident {
            pub fn new() -> #ident {
                #ident { 
                    v: [0; #size ],
                    negative: false,
                }
            }
            pub fn normalize_up(&mut self) {
                let mut borrow: Limb = 0;
                for i in 0..#size {
                    let x: Limb;
                    if i == 0 || i == #size - 1 {
                        x = 1;
                    } else {
                        x = 0;
                    }
                    let y = self.v[i];
                    let r = x.wrapping_sub(borrow);
                    if x >= borrow {
                        borrow = 0;
                    } else {
                        borrow = 1;
                    }
                    let s = r.wrapping_sub(y);
                    if r >= y {
                        borrow += 0;
                    } else {
                        borrow += 1;
                    }
                    self.v[i] = s;
                }
            }
            pub fn normalize(&mut self) {
                if self.negative {
                    self.normalize_up();
                }
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


