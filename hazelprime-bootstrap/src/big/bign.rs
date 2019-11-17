#![warn(rust_2018_idioms)]
#![allow(dead_code)]

#[cfg(feature="trace_macros")]
trace_macros!(true);

// use super::appendage::{*};
use super::pod::{*};
use super::parse::{*};
use crate::make_fromstr;


use rug::Integer;
// use rug::Assign;
use_ops!();

pub struct BigN {
    v: Integer
}

impl Add for BigN {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        return Self { v: self.v + other.v };
    }
}

impl AddAssign for BigN {
    fn add_assign(&mut self, other: Self) {
        self.v += other.v;
    }
}

impl Sub for BigN {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        let newv = self.v - other.v;
        assert!(newv >= 0);
        return Self { v: newv };
    }
}

impl SubAssign for BigN {
    fn sub_assign(&mut self, other: Self) {
        self.v -= other.v;
        assert!(self.v > 0);
    }
}

impl FromStrRadix for BigN {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseBigError> {
        match Integer::from_str_radix(s, radix as i32) {
            Ok(v) => match v < 0 {
                true => Err(ParseBigError {kind: Underflow}),
                false => Ok(Self {v:v})
            },
            Err(e) => Err(ParseBigError {kind: LibError(e.to_string())})
        }
    }
}

make_fromstr!(BigN);

macro_rules! use_with {
    ($T:ty, $X:ident, $conversion:expr) => {
        impl From<$T> for BigN {
            fn from(src: $T) -> Self {
                let $X = src;
                return Self { v: Integer::from($conversion) };
            }
        }
        
        impl Add<$T> for BigN {
            type Output = Self;
            
            fn add(self, other: $T) -> Self {
                let $X = other;
                return Self { v: self.v.add($conversion) };
            }
        }
        
        impl AddAssign<$T> for BigN {
            fn add_assign(&mut self, other: $T) {
                let $X = other;
                self.v += $conversion;
            }
        }
        
        impl Sub<$T> for BigN {
            type Output = Self;
            
            fn sub(self, other: $T) -> Self {
                let $X = other;
                let newv = self.v - $conversion;
                assert!(newv >= 0);
                return Self { v: newv };
            }
        }
        
        impl SubAssign<$T> for BigN {
            fn sub_assign(&mut self, other: $T) {
                let $X = other;
                self.v -= $conversion;
                assert!(self.v >= 0);
            }
        }
    }
}

use_with! { u8, x, x }
use_with! { u16, x, x }
use_with! { u32, x, x }
use_with! { u64, x, x }
use_with! { u128, x, x }
#[cfg(target_pointer_width = "32")]
use_with! { usize, x, x as u32 }
#[cfg(target_pointer_width = "64")]
use_with! { usize, x, x as u64 }

impl Podly for BigN {}

