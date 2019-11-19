#![warn(rust_2018_idioms)]

use super::pod::{*};
use crate::big::errors::{*};
use super::{*};

pub use rug::Integer as RugInteger;

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

impl Lower for RugInteger {}

impl Signed for RugInteger {}

use_add_signed!();
add_signed!(RugInteger);

use_podn_with!();
podn_with!(RugInteger);

impl SubAssign<usize> for PodN<RugInteger> where
{
    fn sub_assign(&mut self, other: usize) {
        #[cfg(target_pointer_width="32")]
        let o: u32 = other as u32;
        #[cfg(target_pointer_width="64")]
        let o: u64 = other as u64;
        self.0 -= o;
        self.0.assert_not_negative();
    }
}

wrapped_arithmetic_unsigned!(RugInteger, u8);
wrapped_arithmetic_unsigned!(RugInteger, u16);
wrapped_arithmetic_unsigned!(RugInteger, u32);
wrapped_arithmetic_unsigned!(RugInteger, u64);
wrapped_arithmetic_unsigned!(RugInteger, u128);
wrapped_arithmetic_signed!(RugInteger, RugInteger);


impl Podly for PodN<RugInteger> {}
