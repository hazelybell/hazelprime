#![warn(rust_2018_idioms)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature="trace_macros")]
trace_macros!(true);


use super::appendage::{*};
use rug::Integer;
use rug::Assign;
use std::ops::Add;
// use std::ops::AddAssign;

pub struct BigN {
    v: Integer
}

macro_rules! use_with {
    ($T:ty, $X:ident, $conversion:expr) => {
        impl From<$T> for BigN {
            fn from(src: $T) -> Self {
                let $X = src;
                return Self { v: Integer::from($conversion) };
            }
        }
        
//         impl Add<$T> for BigN {
//             type Output = Self;
//             
//             fn add(self, other: $T) -> Self {
//                 let x = other;
//                 return Self { v: self.v.add($X) };
//             }
//         }
        
//         impl AddAssign<$T> for ApBitIndex {
//             fn add_assign(&mut self, other: $T) {
//                 let newv : Smol = self.v + Smol::from(other as Smol);
//                 assert!(newv < AP_BITS);
//                 self.v = newv;
//             }
//         }
        
        
    }
}

use_with! {u8, x, x}
use_with! { u16, x, x }
use_with! { u32, x, x }
use_with! { u64, x, x }
use_with! { u128, x, x }
#[cfg(target_ptr_width = "32")]
use_with! { usize, x, x as u32 }
#[cfg(target_ptr_width = "64")]
use_with! { usize, x, x as u64 }



