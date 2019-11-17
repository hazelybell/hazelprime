#![warn(rust_2018_idioms)]

macro_rules! use_ops {
    () => {
        use std::str::FromStr;
        use std::ops::Add;
        use std::ops::AddAssign;
        use std::ops::Sub;
        use std::ops::SubAssign;
    }
}

mod appendage;
mod parse;
// mod bign;
#[macro_use] mod pod;


