#![warn(rust_2018_idioms)]

use std::ops::Add;
use std::ops::Sub;
use std::ops::AddAssign;

trace_macros!(true);

macro_rules! make_big_trait_ops {
    ($d:tt $R1:ident $(, $R:ident)*) => {
        macro_rules! make_big_trait_types {
            ($d($d T:ty),*) => {
                pub trait PodN: 
                    $R1 $(+ $R)*
                    $d(+ $R1<$d T> $(+ $R<$d T>)*)*
                    where Self: std::marker::Sized
                {}
            }
        }
    }
}

make_big_trait_ops! {$ Add, AddAssign, Sub}
make_big_trait_types! {u8, u16}

