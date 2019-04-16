#![warn(rust_2018_idioms)]

use std::ops::Index;
use std::ops::IndexMut;
use std::cmp::Ordering;
use std::ops::Mul;
use std::ops::Add;
use std::ops::Sub;

use crate::limb::{*};
use crate::pod::{*};
use crate::svast::{*};
use crate::big::{*};

pub struct SBig {
    pub v: Big,
    pub negative: bool
}

impl SBig {
    pub fn new(sz : BigSize) -> SBig {
        return SBig { 
            v: Big::new(sz),
            negative: false,
            };
    }
    pub fn new_one(sz: BigSize) -> SBig {
        return SBig { 
            v: Big::new_one(sz),
            negative: false,
            };
    }
    pub fn length(&self) -> BigSize { self.v.length() as BigSize }
    pub fn zero(&mut self) { self.v.zero(); self.negative = false; }
    pub fn is_negative(&self) -> bool {
        (self.v != 0) && self.negative
    }
    pub fn into_big(self) -> Big {
        return self.v;
    }
    pub fn downsized(&self, sz: BigSize) -> SBig {
        SBig {
            v: self.v.downsized(sz),
            negative: self.negative
        }
    }
}

impl Index<BigSize> for SBig {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i] }
}

impl IndexMut<BigSize> for SBig {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb { &mut self.v[i] }
}

impl Clone for SBig {
    fn clone(&self) -> SBig {
        let c = self.v.clone();
        return SBig { v: c, negative: self.negative };
    }
}

impl PartialEq for SBig {
    fn eq(&self, other: &SBig) -> bool {
        SVast::from(self) == SVast::from(other)
    }
}
impl Eq for SBig {}
impl Ord for SBig {
    fn cmp(&self, other: &SBig) -> Ordering {
        SVast::from(self).cmp(&SVast::from(other))
    }
}
impl PartialOrd for SBig {
    fn partial_cmp(&self, other: &SBig) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub<&SBig> for &SBig {
    type Output = SBig;

    fn sub(self, a: &SBig) -> SBig {
        let mut v: Big;
        let negative: bool;
        if (!self.negative) && (!a.negative) {
            if self.v >= a.v {
                v = self.v.clone();
                v -= &a.v;
                negative = false;
            } else {
                v = a.v.clone();
                v -= &self.v;
                negative = true;
            }
        } else if (!self.negative) && (a.negative) {
            v = self.v.clone();
            v += &a.v;
            negative = false;
        } else if (self.negative) && (!a.negative) {
            v = self.v.clone();
            v += &a.v;
            negative = true;
        } else { // both negative
            if self.v >= a.v { // self <= a
                v = self.v.clone();
                v -= &a.v;
                negative = true;
            } else {
                v = a.v.clone();
                v -= &self.v;
                negative = false;
            }
        }
        return SBig { v: v, negative: negative };
    }
}

impl Add<&SBig> for &SBig {
    type Output = SBig;

    fn add(self, a: &SBig) -> SBig {
        let mut v: Big;
        let negative: bool;
        if (!self.negative) && (!a.negative) {
            negative = false;
            v = self.v.clone();
            v += &a.v;
        } else if (!self.negative) && (a.negative) {
            if self.v >= a.v {
                negative = false;
                v = self.v.clone();
                v -= &a.v;
            } else {
                negative = true;
                v = a.v.clone();
                v -= &self.v;
            }
        } else if (self.negative) && (!a.negative) {
            if self.v >= a.v {
                negative = true;
                v = self.v.clone();
                v -= &a.v;
            } else {
                negative = false;
                v = a.v.clone();
                v -= &self.v;
            }
        } else { // both negative
            negative = true;
            v = self.v.clone();
            v += &a.v;
        }
        return SBig { v: v, negative: negative };
    }
}

impl Add<&Big> for &SBig {
    type Output = SBig;

    fn add(self, a: &Big) -> SBig {
        let mut v: Big;
        let negative: bool;
        if !self.negative {
            negative = false;
            v = self.v.clone();
            v += a;
        } else {
            if &self.v >= a {
                negative = true;
                v = self.v.clone();
                v -= &a;
            } else {
                negative = false;
                v = a.clone();
                v -= &self.v;
            }
        }
        return SBig { v: v, negative: negative };
    }
}

impl Mul<&Big> for &SBig {
    type Output = SBig;
    
    fn mul(self, rhs: &Big) -> SBig {
        let p = &(self.v) * rhs;
        return SBig {
            v: p,
            negative: self.negative
        }
    }
}
impl Mul<&SBig> for &Big {
    type Output = SBig;
    
    fn mul(self, rhs: &SBig) -> SBig {
        rhs * self
    }
}


#[cfg(test)]
mod tests {
    use crate::sbig::{*};
    #[test]
    fn smoke() {
        let a = SBig::new(2);
        assert_eq!(a.length(), 2);
        let mut b = SBig::new_one(2);
        assert_eq!(a.negative, false);
        b.zero();
        assert_eq!(b[0], 0);
        assert!(a.eq(&b));
        assert!(a == b);
        let b = SBig::new_one(2);
        assert!(a < b);
        assert!(b > a);
        assert!(b != a);
    }
}
