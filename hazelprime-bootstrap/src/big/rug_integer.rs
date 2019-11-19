#![warn(rust_2018_idioms)]

use super::parse::{*};
use super::sign::{*};
use super::natural::{*};

use rug::Integer as RugInteger;

use_arith!();

impl Signed for RugInteger {}

impl IsNegative for RugInteger {
    fn is_negative(&self) -> bool {
        self < &0
    }
}

impl FromStrRadix for RugInteger {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseBigError> {
        match RugInteger::from_str_radix(s, radix as i32) {
            Ok(v) => Ok(v),
            Err(e) => Err(ParseBigError {kind: LibError(e.to_string())}),
        }
    }
}

impl NotWrapped for RugInteger {}

natural_from_unsigned!(RugInteger, u8);
natural_from_unsigned!(RugInteger, u16);
natural_from_unsigned!(RugInteger, u32);
natural_from_unsigned!(RugInteger, u64);
natural_from_unsigned!(RugInteger, u128);
natural_from_unsigned!(RugInteger, usize);
natural_from!(RugInteger, RugInteger);

arithmetic_with_unsigned!(RugInteger, u8);
arithmetic_with_unsigned!(RugInteger, u16);
arithmetic_with_unsigned!(RugInteger, u32);
arithmetic_with_unsigned!(RugInteger, u64);
arithmetic_with_unsigned!(RugInteger, u128);
arithmetic_with_signed!(RugInteger, RugInteger);
arithmetic_with_self!(RugInteger);

macro_rules! size_plain_op {
    ($T:ident, $f:tt) => {
        impl $T<usize> for WrappedNatural<RugInteger> where
        {
            type Output = Self;
            
            fn $f(self, other: usize) -> Self::Output {
                #[cfg(target_pointer_width="32")]
                let o: u32 = other as u32;
                #[cfg(target_pointer_width="64")]
                let o: u64 = other as u64;
                return Self((self.0).$f(o));
            }
        }
    }
}

macro_rules! size_assign_op {
    ($T:ident, $f:tt) => {
        impl $T<usize> for WrappedNatural<RugInteger> where
        {
            fn $f(&mut self, other: usize) {
                #[cfg(target_pointer_width="32")]
                let o: u32 = other as u32;
                #[cfg(target_pointer_width="64")]
                let o: u64 = other as u64;
                (self.0).$f(o);
            }
        }
    }
}

size_plain_op!(Add, add);
size_assign_op!(AddAssign, add_assign);
size_plain_op!(Sub, sub);
size_assign_op!(SubAssign, sub_assign);

impl Natural for WrappedNatural<RugInteger> {}

