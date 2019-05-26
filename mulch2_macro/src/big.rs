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
    let size = bits / LIMB_BITS;
    let name = &format!("Big{}", bits);
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
        impl Pod for #ident {
            fn limbs(&self) -> usize {
                #size
            }
            #[cfg(feature="no-bounds-checks")]
            fn get_limb(&self, i: usize) -> &Limb {
                let r: &Limb;
                unsafe {
                    r = self.v.get_unchecked(i)
                }
                return r;
            }
            #[cfg(not(feature="no-bounds-checks"))]
            fn get_limb(&self, i: usize) -> &Limb {
                self.v.get(i).unwrap()
            }
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
