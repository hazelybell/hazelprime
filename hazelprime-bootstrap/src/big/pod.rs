#![warn(rust_2018_idioms)]

use_ops! {}
use super::parse::ParseBigError;
use rug::Integer;
use super::parse::{*};
use std::cmp::PartialOrd;
use std::cmp::Ordering;

pub trait IsNegative {
    fn is_negative(&self) -> bool;
}

impl IsNegative for Integer {
    fn is_negative(&self) -> bool {
        self < &0
    }
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

pub trait FromStrRadix where
    Self: std::marker::Sized,
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseBigError>;
}

impl FromStrRadix for Integer {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseBigError> {
        match Integer::from_str_radix(s, radix as i32) {
            Ok(v) => Ok(v),
            Err(e) => Err(ParseBigError {kind: LibError(e.to_string())}),
        }
    }
}

pub trait Lower {}
impl Lower for u8 {}
impl Lower for u16 {}
impl Lower for u32 {}
impl Lower for u64 {}
impl Lower for u128 {}
impl Lower for usize {}
impl Lower for Integer {}

pub trait Primitive where Self: Lower {} 
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for u128 {}
impl Primitive for usize {}

pub struct PodN<T>(T);

impl<T> IsNegative for PodN<T> 
    where T: IsNegative
{
    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

pub trait Unsigned {}

impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}
impl Unsigned for usize {}
impl<T> Unsigned for PodN<T> {}

impl<U, T> Add<U> for PodN<T> where
    T: Lower,
    T: Add<U, Output = T>,
    U: Lower,
    U: Unsigned,
{
    type Output = Self;
    
    fn add(self, other: U) -> Self {
        return Self(self.0.add(other));
    }
}

impl<U, T> Add<PodN<U>> for PodN<T> where
    T: Lower,
    T: Add<U, Output = T>,
    U: Lower,
    U: Unsigned,
{
    type Output = Self;
    
    fn add(self, other: PodN<U>) -> Self {
        return Self(self.0.add(other.0));
    }
}

impl<U, T> AddAssign<U> for PodN<T> where
    T: Lower,
    T: AddAssign<U>, 
    U: Lower,
    U: Unsigned,
{
    fn add_assign(&mut self, other: U) {
        self.0 += other;
    }
}

impl<U, T> AddAssign<PodN<U>> for PodN<T> where
    T: Lower,
    T: AddAssign<U>, 
    U: Lower,
    U: Unsigned,
{
    fn add_assign(&mut self, other: PodN<U>) {
        self.0 += other.0;
    }
}


pub trait Signed {}
impl Signed for Integer {}

// impl<U, T> Add<U> for PodN<T> where
//     T: Add<U, Output = T>,
//     U: Integer,
// {
//     type Output = Self;
//     
//     fn add(self, other: U) -> Self {
//         let newv = self.0.add(other);
//         newv.assert_not_negative();
//         return Self(newv);
//     }
// }
// 
// impl<U, T> AddAssign<U> for PodN<T> where
//     T: AddAssign<U>, 
//     U: Signed,
// {
//     fn add_assign(&mut self, other: U) {
//         self.0 += other;
//         self.0.assert_not_negative();
//     }
// }

impl<T> AddAssign<PodN<Integer>> for PodN<T> where
    T: Lower,
    T: AddAssign<Integer>, 
    T: AssertNotNegative,
{
    fn add_assign(&mut self, other: PodN<Integer>) {
        self.0 += other.0;
        self.0.assert_not_negative();
    }
}

impl<T> AddAssign<Integer> for PodN<T> where
    T: Lower,
    T: AddAssign<Integer>, 
    T: AssertNotNegative,
{
    fn add_assign(&mut self, other: Integer) {
        self.0 += other;
        self.0.assert_not_negative();
    }
}

impl<T> Add<Integer> for PodN<T> where
    T: Add<Integer, Output = T>,
    T: AssertNotNegative,
{
    type Output = Self;
    
    fn add(self, other: Integer) -> Self {
        let newv = self.0.add(other);
        newv.assert_not_negative();
        return Self(newv);
    }
}

impl<T> Add<PodN<Integer>> for PodN<T> where
    T: Add<Integer, Output = T>,
    T: AssertNotNegative,
{
    type Output = Self;
    
    fn add(self, other: PodN<Integer>) -> Self {
        let newv = self.0.add(other.0);
        newv.assert_not_negative();
        return Self(newv);
    }
}

impl<U, T> Sub<U> for PodN<T> where 
    T: Sub<U, Output = T>,
    T: AssertNotNegative,
    U: Lower,
{
    type Output = Self;
    
    fn sub(self, other: U) -> Self {
        let newv = self.0.sub(other);
        newv.assert_not_negative();
        return Self(newv);
    }
}

impl<U, T> Sub<PodN<U>> for PodN<T> where 
    T: Sub<U, Output = T>,
    T: AssertNotNegative,
{
    type Output = Self;
    
    fn sub(self, other: PodN<U>) -> Self {
        let newv = self.0.sub(other.0);
        newv.assert_not_negative();
        return Self(newv);
    }
}

impl<U, T> SubAssign<U> for PodN<T> where
    T: SubAssign<U>,
    T: AssertNotNegative,
    U: Lower,
{
    fn sub_assign(&mut self, other: U) {
        self.0 -= other;
        self.0.assert_not_negative();
    }
}

impl<U, T> SubAssign<PodN<U>> for PodN<T> where
    T: SubAssign<U>,
    T: AssertNotNegative,
{
    fn sub_assign(&mut self, other: PodN<U>) {
        self.0 -= other.0;
        self.0.assert_not_negative();
    }
}


macro_rules! use_with {
    ($U:ty) => {
        impl<T: PartialEq<$U>> PartialEq<$U> for PodN<T> {
            fn eq(&self, other: &$U) -> bool {
                self.0 == *other
            }
        }

        impl<T: PartialOrd<$U>> PartialOrd<$U> for PodN<T> {
            fn partial_cmp(&self, other: &$U) -> Option<Ordering> {
                self.0.partial_cmp(other)
            }
        }
        
        impl<T: From<$U>> From<$U> for PodN<T> where
            T: AssertNotNegative
        {
            fn from(src: $U) -> Self {
                let newv: T = T::from(src);
                newv.assert_not_negative();
                return Self(newv);
            }
        }
    }
}


use_with!(u8);
use_with!(u16);
use_with!(u32);
use_with!(u64);
use_with!(u128);
use_with!(usize);
use_with!(Integer);
// #[cfg(target_pointer_width = "32")]
// use_with! { usize as u32 }
// #[cfg(target_pointer_width = "64")]
// use_with! { usize as u64 }


impl<T: FromStrRadix> FromStrRadix for PodN<T>
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

impl<T: FromStrRadix> FromStr for PodN<T> 
    where T: IsNegative
{
    type Err = ParseBigError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

#[cfg(feature="trace_macros")]
trace_macros!(true);

macro_rules! make_podly_trait_ops {
    ($d:tt $($R:ident),*) => {
        macro_rules! make_podly_trait_types {
            ($d($d T:ty),*) => {
                pub trait Podly: FromStr
                    $(+ $R)*
                    $d(
                        + From<$T>
                        $(+ $R<$T>)*
                    )*
                    where 
                        Self: std::marker::Sized,
                        PodN<Self>: FromStrRadix,
                {}
            }
        }
    }
}

make_podly_trait_ops! {$ Add, AddAssign, Sub, SubAssign}
make_podly_trait_types! {u8, u16, u32, u64, u128, Integer}

// impl SubAssign<usize> for PodN<Integer> {
//     #[cfg(target_pointer_width = "32")]
//     fn sub_assign(&mut self, other: Self) {
//         self.0 -= other as u32;
//     }
//     #[cfg(target_pointer_width = "64")]
//     fn sub_assign(&mut self, other: Self) {
//         self.0 -= other as u64;
//     }
// }

impl Podly for PodN<Integer> {}

