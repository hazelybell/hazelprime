#![warn(rust_2018_idioms)]

macro_rules! use_podn_with {
    () => {
        use std::cmp::Ordering;
        use std::cmp::PartialOrd;
    }
}

macro_rules! use_add_signed {
    () => {
        use std::ops::Add;
        use std::ops::AddAssign;
    }
}

macro_rules! use_ops {
    () => {
        use_podn_with!();
        use std::str::FromStr;
        use_add_signed!();
        use std::ops::Sub;
        use std::ops::SubAssign;
    }
}

use_ops! {}
use super::errors::{*};

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

pub trait FromStrRadix where
    Self: std::marker::Sized,
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseBigError>;
}

pub trait Lower {}
impl Lower for u8 {}
impl Lower for u16 {}
impl Lower for u32 {}
impl Lower for u64 {}
impl Lower for u128 {}
impl Lower for usize {}

trait Primitive where Self: Lower {} 
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for u128 {}
impl Primitive for usize {}

pub struct PodN<T>(pub(super) T);

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

macro_rules! add_signed { ($U:ty) => {

    impl<T> AddAssign<PodN<$U>> for PodN<T> where
        T: Lower,
        T: AddAssign<$U>, 
        T: AssertNotNegative,
    {
        fn add_assign(&mut self, other: PodN<$U>) {
            self.0 += other.0;
            self.0.assert_not_negative();
        }
    }

    impl<T> AddAssign<$U> for PodN<T> where
        T: Lower,
        T: AddAssign<$U>, 
        T: AssertNotNegative,
    {
        fn add_assign(&mut self, other: $U) {
            self.0 += other;
            self.0.assert_not_negative();
        }
    }

    impl<T> Add<$U> for PodN<T> where
        T: Add<$U, Output = T>,
        T: AssertNotNegative,
    {
        type Output = Self;
        
        fn add(self, other: $U) -> Self {
            let newv = self.0.add(other);
            newv.assert_not_negative();
            return Self(newv);
        }
    }

    impl<T> Add<PodN<$U>> for PodN<T> where
        T: Add<$U, Output = T>,
        T: AssertNotNegative,
    {
        type Output = Self;
        
        fn add(self, other: PodN<$U>) -> Self {
            let newv = self.0.add(other.0);
            newv.assert_not_negative();
            return Self(newv);
        }
    }
}}

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

macro_rules! wrapped_arithmetic {
    ($T:ty, $U:ty) => {
        impl SubAssign<$U> for PodN<$T>
        {
            fn sub_assign(&mut self, other: $U) {
                self.0 -= other;
                self.0.assert_not_negative();
            }
        }
        
        impl AddAssign<$U> for PodN<$T>
        {
            fn add_assign(&mut self, other: $U) {
                self.0 += other;
            }
        }
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

macro_rules! podn_with {
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


podn_with!(u8);
podn_with!(u16);
podn_with!(u32);
podn_with!(u64);
podn_with!(u128);
podn_with!(usize);
// #[cfg(target_pointer_width = "32")]
// podn_with! { usize as u32 }
// #[cfg(target_pointer_width = "64")]
// podn_with! { usize as u64 }


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
// log_syntax!(true);

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
make_podly_trait_types! {u8, u16, u32, u64, u128, usize}

macro_rules! make_interpod_trait_ops {
    ($d:tt $($R:ident),*) => {
        macro_rules! make_interpod_trait_primitives {
            ($d2:tt $d($d T:ty),*) => {
                macro_rules! make_interpod_trait_types {
                    ($d2($d2 U:ty),*) => {
                        pub trait Interpod: FromStr
                            $(+ $R)*
                            $d(
                                + From<$T>
                                $(+ $R<$T>)*
                            )*
                            $d2(
                                + From<$U>
                                $(+ $R<$U>)*
                            )*
                            where 
                                Self: std::marker::Sized,
                                PodN<Self>: FromStrRadix,
                        {}
                    }
                }
            }
        }
    }
}

make_interpod_trait_ops!($ Add, AddAssign, Sub, SubAssign);
make_interpod_trait_primitives!($ u8, u16, u32, u64, u128);

