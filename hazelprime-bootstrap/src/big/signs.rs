#![warn(rust_2018_idioms)]

pub trait Signage {
    pub fn is_negative(self) -> bool;
    pub fn is_positive(self) -> bool;
}


