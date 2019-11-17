#![warn(rust_2018_idioms)]

use std::mem::size_of;
use std::ops::Add;
use std::ops::AddAssign;

// The architectural word size
pub type Lorg = usize;

// A type that can hold a value at least as large as the number of bits
// in Lorg; this is set to u32 because that's what Rust uses as arguments
// to its various shift routines
// It could be changed at some point for speed or compactness?
pub type Smol = u32;

pub const AP_BITS : Smol = (size_of::<Lorg>() * 8) as Smol;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ApBitIndex {
    v: Smol
}

macro_rules! xvert_as {
    ($T:ty) => {
        impl From<$T> for ApBitIndex {
            fn from(src: $T) -> Self {
                assert!(src < (AP_BITS as $T));
                return ApBitIndex { v: (src as Smol) };
            }
        }
        
        impl Add<$T> for ApBitIndex {
            type Output = Self;
            
            fn add(self, other: $T) -> Self {
                let newv : Smol = self.v + (other as Smol);
                assert!(newv < AP_BITS);
                return ApBitIndex { v: newv };
            }
        }
        
        impl AddAssign<$T> for ApBitIndex {
            fn add_assign(&mut self, other: $T) {
                let newv : Smol = self.v + Smol::from(other as Smol);
                assert!(newv < AP_BITS);
                self.v = newv;
            }
        }
        
        
    }
}

xvert_as! { u8 }
xvert_as! { u16 }
xvert_as! { u32 }
xvert_as! { u64 }
xvert_as! { u128 }
xvert_as! { usize }

pub trait Appendage {
    fn bits() -> Smol;
    
}
