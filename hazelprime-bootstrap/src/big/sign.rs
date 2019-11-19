#![warn(rust_2018_idioms)]

pub trait Unsigned {}

impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}
impl Unsigned for usize {}

pub trait Signed {}

impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}
impl Signed for i128 {}
impl Signed for isize {}

pub trait IsNegative {
    fn is_negative(&self) -> bool;
}

pub trait AssertNotNegative {
    fn assert_not_negative(&self);
}

impl<T> AssertNotNegative for T where T: IsNegative {
    fn assert_not_negative(&self) {
        if self.is_negative() {
            panic!("Underflow");
        }
    }
}

impl<T> IsNegative for T where T: Unsigned {
    fn is_negative(&self) -> bool {false}
}

