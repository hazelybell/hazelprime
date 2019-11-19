#![warn(rust_2018_idioms)]

macro_rules! use_arith {
    () => {
        use std::ops::Add;
        use std::ops::AddAssign;
        use std::ops::Sub;
        use std::ops::SubAssign;
    }
}

use std::str::FromStr;
use_arith!();

use super::parse::{*};
use super::sign::{*};

pub struct WrappedNatural<T>(pub(super) T);

pub trait NotWrapped {}

impl NotWrapped for u8 {}
impl NotWrapped for u16 {}
impl NotWrapped for u32 {}
impl NotWrapped for u64 {}
impl NotWrapped for u128 {}
impl NotWrapped for usize {}

pub trait HasNoop {
    fn noop(&self) -> ();
}

impl<T> HasNoop for T where T: NotWrapped {
    fn noop(&self) -> () {}
}

impl <T: FromStrRadix> FromStrRadix for WrappedNatural<T>
    where T: IsNegative
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseBigError> { 
        match T::from_str_radix(src, radix) {
            Ok(x) => match x.is_negative() {
                true => Err(ParseBigError {kind: Underflow}),
                false => Ok(Self(x))
            },
            Err(e) => Err(e),
        }
    }
}

impl<T: FromStrRadix> FromStr for WrappedNatural<T> 
    where T: IsNegative
{
    type Err = ParseBigError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

macro_rules! natural_from {
    ($Y:ident, $U:ty) => {
        impl From<$U> for WrappedNatural<$Y> where
            $Y: From<$U>,
            $Y: AssertNotNegative,
        {
            fn from(src: $U) -> Self {
                let newv: $Y = $Y::from(src);
                newv.assert_not_negative();
                return Self(newv);
            }
        }
    }
}

macro_rules! natural_from_unsigned {
    ($Y:ident, $U:ty) => {
        impl From<$U> for WrappedNatural<$Y> where
            $Y: From<$U>
        {
            fn from(src: $U) -> Self {
                let newv: $Y = $Y::from(src);
                return Self(newv);
            }
        }
    }
}

macro_rules! plain_op {
    ($T:ident, $f:ident, $Y:ty, $U:ty, $UT:path, $VT:path, $Vf:ident) => {
        impl $T<$U> for WrappedNatural<$Y> where
            $U: $UT,
            $Y: $T<$U>,
            $Y: $VT,
        {
            type Output = Self;
            
            fn $f(self, other: $U) -> Self::Output {
                let newv : $Y = (self.0).$f(other);
                newv.$Vf();
                Self(newv)
            }
        }
    }
}

macro_rules! assign_op {
    ($T:ident, $f:ident, $Y:ty, $U:ty, $UT:path, $VT:path, $Vf:ident) => {
        impl $T<$U> for WrappedNatural<$Y> where 
            $U: $UT,
            $Y: $T<$U>,
            $Y: $VT,
        {
            fn $f(&mut self, other: $U) {
                (self.0).$f(other);
                (self.0).$Vf();
            }
        }
    }
}

macro_rules! plain_op_unchecked {
    ($T:ident, $f:ident, $Y:ty, $U:ty) => {
        plain_op!($T, $f, $Y, $U, Unsigned, NotWrapped, noop);
    }
}

macro_rules! plain_op_checked {
    ($T:ident, $f:ident, $Y:ty, $U:ty) => {
        plain_op!($T, $f, $Y, $U, NotWrapped, AssertNotNegative, assert_not_negative);
    }
}

macro_rules! assign_op_unchecked {
    ($T:ident, $f:ident, $Y:ty, $U:ty) => {
        assign_op!($T, $f, $Y, $U, Unsigned, NotWrapped, noop);
    }
}


macro_rules! assign_op_checked {
    ($T:ident, $f:ident, $Y:ty, $U:ty) => {
        assign_op!($T, $f, $Y, $U, NotWrapped, AssertNotNegative, assert_not_negative);
    }
}

macro_rules! sub_with_whatever {
    ($Y:ty, $U:ty) => {
        plain_op_checked!(Sub, sub, $Y, $U);
        assign_op_checked!(SubAssign, sub_assign, $Y, $U);
    }
}

macro_rules! arithmetic_with_unsigned {
    ($Y:ty, $U:ty) => {
        plain_op_unchecked!(Add, add, $Y, $U);
        assign_op_unchecked!(AddAssign, add_assign, $Y, $U);
        sub_with_whatever!($Y, $U);
    }
}

macro_rules! arithmetic_with_signed {
    ($Y:ident, $U:ty) => {
        plain_op_checked!(Add, add, $Y, $U);
        assign_op_checked!(AddAssign, add_assign, $Y, $U);
        sub_with_whatever!($Y, $U);
    }
}

macro_rules! arithmetic_with_self {
    ($Y:ty) => {
        impl Add for WrappedNatural<$Y> where
        {
            type Output = Self;
            
            fn add(self, other: Self) -> Self::Output {
                self + other.0
            }
        }

        impl AddAssign for WrappedNatural<$Y> where
            WrappedNatural<$Y>: AddAssign<$Y>
        {
            fn add_assign(&mut self, other: Self) {
                self.add_assign(other.0);
            }
        }
        
        impl Sub for WrappedNatural<$Y> where
        {
            type Output = Self;
            
            fn sub(self, other: Self) -> Self::Output {
                self - other.0
            }
        }

        impl SubAssign for WrappedNatural<$Y> where
            WrappedNatural<$Y>: AddAssign<$Y>
        {
            fn sub_assign(&mut self, other: Self) {
                self.sub_assign(other.0);
            }
        }
    }
}

// impl<T, U> Add<WrappedNatural<U>> for WrappedNatural<T> where
//     T: NotWrapped,
//     U: NotWrapped,
//     WrappedNatural<T>: Add<U, Output = WrappedNatural<T>>,
// {
//     type Output = WrappedNatural<T>;
//     
//     fn add(self, other: WrappedNatural<U>) -> Self::Output {
//         let newv: WrappedNatural<T> = self.add(other.0);
//         return newv;
//     }
// }

macro_rules! make_natural_trait_ops {
    ($d:tt $($R:ident),*) => {
        macro_rules! make_natural_trait_types {
            ($d($d Y:ty),*) => {
                pub trait Natural: FromStrRadix + FromStr
                    $(+ $R)*
                    $d(
                        + From<$Y>
                        $(+ $R<$Y>)*
                    )*
                {}
            }
        }
    }
}

make_natural_trait_ops! {$ Add, AddAssign, Sub, SubAssign}
make_natural_trait_types! {u8, u16, u32, u64, u128, usize}



